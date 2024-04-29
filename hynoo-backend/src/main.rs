use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use warp::ws::{Message, WebSocket};
use warp::Filter;
#[derive(Serialize)]
struct MessageData {
    username: String,
    content: String,
}
struct UserData {
    username: String,
    tx: tokio::sync::mpsc::UnboundedSender<Result<Message, warp::Error>>,
}
type Users =
    Arc<Mutex<HashMap<String, tokio::sync::mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
type Rooms = Arc<Mutex<HashMap<String, Users>>>;

async fn handle_init(ws: WebSocket, rooms: Rooms) {
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let msg = user_ws_rx
        .next()
        .await
        .unwrap()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let v: Value = serde_json::from_str(&msg).unwrap();
    let room_id = v["room_id"].as_str().unwrap().to_string();
    let username = v["username"].as_str().unwrap().to_string();
    // if room_id is not exist, create it
    // if room_id is exist, add user to it
    if rooms.lock().unwrap().contains_key(&room_id) {
        rooms
            .lock()
            .unwrap()
            .get(&room_id)
            .unwrap()
            .lock()
            .unwrap()
            .insert(username.clone(), tx.clone());
    } else {
        let users = Arc::new(Mutex::new(HashMap::new()));
        users.lock().unwrap().insert(username.clone(), tx.clone());
        rooms.lock().unwrap().insert(room_id.clone(), users);
    }
    //users.lock().unwrap().insert(username.clone(), tx.clone());
    handle_connection(username, room_id, user_ws_tx, user_ws_rx, rooms.clone(), rx).await;
}
async fn handle_connection(
    user_id: String,
    room_id: String,
    mut user_ws_tx: SplitSink<WebSocket, Message>,
    mut user_ws_rx: SplitStream<WebSocket>,
    rooms: Rooms,
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
                let v: Value = serde_json::from_str(&msg.to_str().unwrap()).unwrap();
                let room_id = v["room_id"].as_str().unwrap().to_string();
                if rooms.lock().unwrap().contains_key(&room_id) {
                    for (roomname, users) in rooms.lock().unwrap().iter() {
                        if (roomname != &room_id) {
                            continue;
                        }
                        for (_, user) in users.lock().unwrap().iter() {
                            user.send(Ok(msg.clone())).unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", user_id, e);
                break;
            }
        }
    }
    // remove user from users
    rooms
        .lock()
        .unwrap()
        .get(&room_id)
        .unwrap()
        .lock()
        .unwrap()
        .remove(&user_id);
    // if users is empty, remove room
    if rooms
        .lock()
        .unwrap()
        .get(&room_id)
        .unwrap()
        .lock()
        .unwrap()
        .is_empty()
    {
        rooms.lock().unwrap().remove(&room_id);
    }
}

#[tokio::main]
async fn main() {
    //let users: Users = Arc::new(Mutex::new(HashMap::new()));
    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));
    let chat = warp::path("chat")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let rooms = rooms.clone();
            //let users = users.clone();
            ws.on_upgrade(move |socket| handle_init(socket, rooms))
        });
    warp::serve(chat).run(([127, 0, 0, 1], 3030)).await;
}
