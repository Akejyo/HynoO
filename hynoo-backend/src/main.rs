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

async fn handle_connection(user_id: String, ws: WebSocket, users: Users) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    users.lock().unwrap().insert(user_id.clone(), tx);

    //when rx recive a message, combine it with usename and send json text {username,content} to client
    tokio::task::spawn(async move {
        while let Some(result) = rx.recv().await {
            let message = match result {
                Ok(msg) => msg.to_str().unwrap().to_string(),
                Err(e) => {
                    eprintln!("error {}", e);
                    break;
                }
            };

            let msg = MessageData {
                username: "User1".to_string(),
                content: message,
            };
            let json_msg = serde_json::to_string(&msg).unwrap();
            let ws_msg = warp::ws::Message::text(json_msg);
            user_ws_tx.send(ws_msg).await.unwrap();
        }
    });

    //when ws recive a message, send it to all tx in users
    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                for (username, user) in users.lock().unwrap().iter() {
                    if username != &user_id {
                        user.send(Ok(msg.clone())).unwrap();
                    }
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
    let user_id_counter = Arc::new(AtomicUsize::new(0));

    let chat = warp::path("chat")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let users = users.clone();
            let user_id_counter = user_id_counter.clone();
            ws.on_upgrade(move |socket| {
                let user_id = user_id_counter.fetch_add(1, Ordering::Relaxed).to_string();
                handle_connection(user_id, socket, users)
            })
        });

    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}
