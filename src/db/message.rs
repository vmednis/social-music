use crate::db;
use futures_util::future;
use redis::streams;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

impl db::DbInternal {
    fn key_messages(room_id: String) -> String {
        format!("room:{}:messages", room_id)
    }

    pub async fn add_message(&mut self, room_id: String, message: Message) {
        let args: Vec<(String, String)> = message.into();
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con
            .xadd(Self::key_messages(room_id), "*", &args[..])
            .await
            .unwrap();
    }

    pub async fn subscribe_messages(&mut self, room_id: String) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel(10);
        let client = self.blockable_client();
        let mut con = client.get_async_connection().await.unwrap();

        tokio::task::spawn(async move {
            let options = streams::StreamReadOptions::default().block(250).count(5);
            let mut id = "$".to_string();

            loop {
                if tx.is_closed() {
                    break;
                }
                let response: redis::RedisResult<streams::StreamReadReply> = con
                    .xread_options(
                        &[Self::key_messages(room_id.clone())],
                        &[id.clone()],
                        &options,
                    )
                    .await;

                let mut sends = Vec::new();
                match response {
                    Ok(reply) => {
                        id = reply
                            .keys
                            .iter()
                            .map(|stream_key| {
                                if stream_key.key == Self::key_messages(room_id.clone()) {
                                    stream_key
                                        .ids
                                        .iter()
                                        .map(|stream_id| {
                                            let message = Message::try_from(stream_id).unwrap();
                                            sends.push(tx.send(message));
                                            stream_id.id.clone()
                                        })
                                        .fold("$".to_string(), |_, val| val)
                                } else {
                                    id.clone()
                                }
                            })
                            .fold(
                                id.clone(),
                                |acc, val| {
                                    if val.len() >= acc.len() {
                                        val
                                    } else {
                                        acc
                                    }
                                },
                            );
                    }
                    Err(err) => {
                        panic!("RedisError: {}", err);
                    }
                }

                future::join_all(sends).await;
            }
        });

        rx
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageChat {
    pub from: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageDeviceChange {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MessageType {
    MessageChat(MessageChat),
    MessageDeviceChange(MessageDeviceChange),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<String>,
    pub data: MessageType,
}

impl Message {
    pub fn chat_message(from: String, message: String) -> Self {
        Self {
            id: None,
            data: MessageType::MessageChat(MessageChat{
                from,
                message,
            })
        }
    }

    pub fn device_change(user_id: String) -> Self {
        Self {
            id: None,
            data: MessageType::MessageDeviceChange(MessageDeviceChange{
                user_id
            })
        }
    }
}

impl Into<Vec<(String, String)>> for Message {
    fn into(self) -> Vec<(String, String)> {
        let mut args: Vec<(String, String)> = Vec::new();

        match self.data {
            MessageType::MessageChat(data) => {
                args.push(("type".to_string(), "MessageChat".to_string()));
                args.push(("from".to_string(), data.from));
                args.push(("message".to_string(), data.message));
            },
            MessageType::MessageDeviceChange(data) => {
                args.push(("type".to_string(), "MessageDeviceChange".to_string()));
                args.push(("user_id".to_string(), data.user_id));
            }
        }

        args
    }
}

impl TryFrom<&redis::streams::StreamId> for Message {
    type Error = &'static str;

    fn try_from(stream_id: &redis::streams::StreamId) -> Result<Self, Self::Error> {
        let id = Some(stream_id.id.clone());

        let message_type = db::util::read_redis_stream_data(stream_id, "type")?;

        let data = match message_type.as_str() {
            "MessageChat" => {
                let from = db::util::read_redis_stream_data(stream_id, "from")?;
                let message = db::util::read_redis_stream_data(stream_id, "message")?;

                Ok(MessageType::MessageChat(MessageChat{
                    from,
                    message,
                }))
            },
            "MessageDeviceChange" => {
                let user_id = db::util::read_redis_stream_data(stream_id, "user_id")?;

                Ok(MessageType::MessageDeviceChange(MessageDeviceChange{
                    user_id,
                }))
            },
            _ => {
                Err("Tried to read non existing message data type")
            }
        }?;

        Ok(Message { id, data })
    }
}
