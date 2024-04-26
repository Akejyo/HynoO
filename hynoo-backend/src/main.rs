use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[derive(Serialize)]
struct MessageData {
    username: String,
    content: String,
}
type Users =
    Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;

async fn handle_init(ws: WebSocket, users: Users) {
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let username = user_ws_rx
        .next()
        .await
        .unwrap()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    users.lock().unwrap().insert(username.clone(), tx.clone());
    handle_connection(username, user_ws_tx, user_ws_rx, users.clone(), rx).await;
}
async fn handle_connection(
    user_id: String,
    mut user_ws_tx: SplitSink<WebSocket, Message>,
    mut user_ws_rx: SplitStream<WebSocket>,
    users: Users,
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Result<Message, warp::Error>>,
) {
    tokio::task::spawn(async move {
        while let Some(result) = rx.recv().await {
            let message = match result {
                Ok(msg) => msg.to_str().unwrap().to_string(),
                Err(e) => {
                    eprintln!("error {}", e);
                    break;
                }
            };
            let ws_msg = warp::ws::Message::text(message);
            user_ws_tx.send(ws_msg).await.unwrap();
        }
    });

    //when ws recive a message, send it to all tx in users
    while let Some(result) = user_ws_rx.next().await {
        //println!("received message from {}", user_id);
        match result {
            Ok(msg) => {
                for (_, user) in users.lock().unwrap().iter() {
                    //println!("send message to {}", user_id);
                    user.send(Ok(msg.clone())).unwrap();
                }
            }
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", user_id, e);
                break;
            }
        }
    }

    users.lock().unwrap().remove(&user_id);
}

#[tokio::main]
async fn main() {
    let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let chat = warp::path("chat")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let users = users.clone();
            ws.on_upgrade(move |socket| handle_init(socket, users))
        });
    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}
