use crate::db;
use futures_util::{SinkExt, StreamExt};
use warp::ws::WebSocket;

pub async fn connected(ws: WebSocket, room_id: String, user_id: String, db: db::Db) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    //Send out messages to the client
    let inner_db = db.clone();
    let inner_room_id = room_id.clone();
    let (kill_db_tx, mut kill_db_rx) = tokio::sync::mpsc::channel(1);
    let (system_tx, mut system_rx) = tokio::sync::mpsc::channel(1);
    tokio::task::spawn(async move {
        let mut db = inner_db.lock().await;
        let mut db_rx = db.subscribe_messages(inner_room_id).await;
        std::mem::drop(db);

        loop {
            tokio::select! {
                msg = db_rx.recv() => {
                    match msg {
                        Some(message) => {
                            let json = serde_json::to_string(&data_out::Message::ChatMessage(message)).unwrap();
                            ws_tx.send(warp::ws::Message::text(json)).await.unwrap();
                        },
                        _ => {
                            log::info!("Database subscription in handle_chat_connected died");
                            break;
                        }
                    }
                },
                msg = system_rx.recv() => {
                    match msg {
                        Some(message) => {
                            let message = data_out::Message::ChatMessage(db::Message{
                                id: None,
                                from: "system".to_string(),
                                message,
                            });

                            let json = serde_json::to_string(&message).unwrap();
                            ws_tx.send(warp::ws::Message::text(json)).await.unwrap();
                        },
                        _ => {
                            break;
                        }
                    }
                }
                _ = kill_db_rx.recv() => {
                    break;
                },
            };
        }
    });

    let mut inner_db = db.lock().await;
    if inner_db.exists_room(room_id.clone()).await {
        std::mem::drop(inner_db);

        //Track user presence, own task in case ws task dies
        let (kill_presence_tx, mut kill_presence_rx) = tokio::sync::mpsc::channel(1);
        let inner_db = db.clone();
        let inner_user_id = user_id.clone();
        let inner_room_id = room_id.clone();
        tokio::task::spawn(async move {
            let mut db = inner_db.lock().await;
            db.add_presence(inner_room_id.clone(), inner_user_id.clone()).await;
            std::mem::drop(db);

            loop {
                let duration = tokio::time::Duration::from_secs(3);
                tokio::select! {
                    exit = kill_presence_rx.recv() => {
                        match exit {
                            None => {
                                log::info!("User presence removed because task exited unexepctedly");
                            },
                            _ => (),
                        }
                        break;
                    },
                    _ = tokio::time::sleep(duration) => {
                        let mut db = inner_db.lock().await;
                        db.keep_alive_presence(inner_room_id.clone(), inner_user_id.clone()).await;
                    }
                }
            }

            let mut db = inner_db.lock().await;
            db.remove_presence(inner_room_id.clone(), inner_user_id.clone()).await;
        });

        //Receive messages from the client
        while let Some(result) = ws_rx.next().await {
            let message_ws = match result {
                Ok(msg) => msg,
                Err(e) => {
                    panic!("Websocket Error {}", e);
                }
            };
            if message_ws.is_text() {
                on_message(
                    db.clone(),
                    room_id.clone(),
                    user_id.clone(),
                    message_ws.to_str().unwrap().to_string(),
                )
                .await;
            }
        }

        kill_presence_tx.send(()).await.unwrap();
    } else {
        system_tx.send(format!("Room {} does not exist!", room_id)).await.unwrap();
    }

    kill_db_tx.send(()).await.unwrap();
}

async fn on_message(db: db::Db, room_id: String, user_id: String, message: String) {
    let message: data_in::Message = serde_json::from_str(&message).unwrap();

    match message {
        data_in::Message::ChatMessage(chat_message) => {
            let message = db::Message {
                id: None,
                from: user_id.clone(),
                message: chat_message.message,
            };

            let mut db = db.lock().await;
            db.add_message(room_id.clone(), message).await;
        }
        data_in::Message::SetDevice(set_device) => {
            let mut db = db.lock().await;
            db.set_device(user_id, set_device.device_id).await;
        }
        data_in::Message::PlaySong(play_song) => {
            let mut db = db.lock().await;
            let token = db.get_auth(user_id.clone()).await.unwrap();
            let device_id = db.get_device(user_id.clone()).await.unwrap();
            std::mem::drop(db);

            let client = reqwest::ClientBuilder::new()
                .connection_verbose(true)
                .build()
                .unwrap();

            //spotify:track:3ZzxtumoIENCi16HAKuiLU
            //spotify:track:2qOfyyZc41LfQ63U9p3GVV
            let track_data = PlayerPlayData {
                context_uri: None,
                uris: Some(vec![play_song.track_id]),
                position_ms: 0,
            };

            client
                .put(format!(
                    "https://api.spotify.com/v1/me/player/play?device_id={}",
                    device_id
                ))
                .bearer_auth(token)
                .json(&track_data)
                .send()
                .await
                .unwrap();
        }
    };
}

use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
struct PlayerPlayData {
    #[serde(skip_serializing_if = "Option::is_none")]
    context_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uris: Option<Vec<String>>,
    position_ms: u32,
}

mod data_out {
    use crate::db;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        ChatMessage(db::Message),
    }
}

mod data_in {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SetDevice {
        pub device_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ChatMessage {
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlaySong {
        pub track_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        ChatMessage(ChatMessage),
        SetDevice(SetDevice),
        PlaySong(PlaySong),
    }
}
