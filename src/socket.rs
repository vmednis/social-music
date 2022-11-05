use crate::db;
use crate::spotify;
use futures_util::{SinkExt, StreamExt};
use warp::ws::WebSocket;

pub async fn connected(
    ws: WebSocket,
    room_id: String,
    user_id: String,
    db: db::Db,
    spotify: spotify::Spotify,
) {
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
                            match message.data {
                                db::message::MessageType::MessageChat(data) => {
                                    let data = data_out::ChatMessage{
                                        id: message.id.unwrap(),
                                        from: data.from,
                                        message: data.message,
                                    };
                                    let message = data_out::Message::ChatMessage(data);
                                    let json = serde_json::to_string(&message).unwrap();
                                    ws_tx.send(warp::ws::Message::text(json)).await.unwrap();
                                },
                                _ => ()
                            }
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
                            let data = data_out::ChatMessage{
                                id: "".to_string(),
                                from: "system".to_string(),
                                message,
                            };
                            let message = data_out::Message::ChatMessage(data);
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
        inner_db.offer_room(room_id.clone()).await;
        std::mem::drop(inner_db);

        //Track user presence, own task in case ws task dies
        let (kill_presence_tx, mut kill_presence_rx) = tokio::sync::mpsc::channel(1);
        let inner_db = db.clone();
        let inner_user_id = user_id.clone();
        let inner_room_id = room_id.clone();
        tokio::task::spawn(async move {
            let mut db = inner_db.lock().await;
            db.add_presence(inner_room_id.clone(), inner_user_id.clone())
                .await;
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
            db.remove_presence(inner_room_id.clone(), inner_user_id.clone())
                .await;
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
                    spotify.clone(),
                    message_ws.to_str().unwrap().to_string(),
                )
                .await;
            }
        }

        kill_presence_tx.send(()).await.unwrap();
    } else {
        system_tx
            .send(format!("Room {} does not exist!", room_id))
            .await
            .unwrap();
    }

    kill_db_tx.send(()).await.unwrap();
}

async fn on_message(
    db: db::Db,
    room_id: String,
    user_id: String,
    _spotify: spotify::Spotify,
    message: String,
) {
    let message: data_in::Message = serde_json::from_str(&message).unwrap();

    match message {
        data_in::Message::ChatMessage(chat_message) => {
            let message = db::message::Message::chat_message(user_id.clone(), chat_message.message);

            let mut db = db.lock().await;
            db.add_message(room_id.clone(), message).await;
        }
        data_in::Message::SetDevice(set_device) => {
            let message = db::message::Message::device_change(user_id.clone());

            let mut db = db.lock().await;
            db.set_device(user_id, set_device.device_id).await;
            db.add_message(room_id, message).await;
        }
        data_in::Message::QueueSong(queue_song) => {
            let mut db = db.lock().await;
            db.push_user_queue(room_id.clone(), user_id.clone(), queue_song.track_id).await;
        }
        data_in::Message::JoinQueue => {
            let mut db = db.lock().await;
            db.push_queue(room_id.clone(), user_id.clone()).await;
        }
    };
}

mod data_out {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ChatMessage {
        pub id: String,
        pub from: String,
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        ChatMessage(ChatMessage),
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
    pub struct QueueSong {
        pub track_id: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Message {
        ChatMessage(ChatMessage),
        SetDevice(SetDevice),
        QueueSong(QueueSong),
        JoinQueue,
    }
}
