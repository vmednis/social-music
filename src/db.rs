use futures_util::future;
use redis::streams;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::Filter;

pub mod room;

pub type Db = Arc<Mutex<DbInternal>>;

pub fn connect_db() -> Db {
    Arc::new(Mutex::new(DbInternal::init()))
}

pub fn with(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub struct DbInternal {
    client: Client,
}

impl DbInternal {
    fn init() -> Self {
        let url = std::env::var("REDIS_URL").unwrap();
        let client = Client::open(url).unwrap();

        Self { client }
    }

    fn key_auth(user_id: String) -> String {
        format!("{}:auth", user_id)
    }

    pub async fn set_auth(&mut self, user_id: String, token: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(Self::key_auth(user_id), token).await.unwrap();
    }

    pub async fn get_auth(&mut self, user_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.get(Self::key_auth(user_id)).await.ok()
    }

    fn key_device(user_id: String) -> String {
        format!("{}:device", user_id)
    }

    pub async fn set_device(&mut self, user_id: String, device_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(Self::key_device(user_id), device_id).await.unwrap();
    }

    pub async fn get_device(&mut self, user_id: String) -> Option<String> {
        let mut con = self.client.get_async_connection().await.unwrap();
        con.get(Self::key_device(user_id)).await.ok()
    }

    fn key_presence(room_id: String, user_id: String) -> String {
        format!("room:{}:presence:{}", room_id, user_id)
    }

    pub async fn add_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.set(Self::key_presence(room_id.clone(), user_id.clone()), "").await.unwrap();
        self.keep_alive_presence(room_id, user_id).await;
    }

    pub async fn remove_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.del(Self::key_presence(room_id, user_id)).await.unwrap();
    }

    pub async fn keep_alive_presence(&mut self, room_id: String, user_id: String) {
        let mut con = self.client.get_async_connection().await.unwrap();
        let _: () = con.expire(Self::key_presence(room_id, user_id), 5).await.unwrap();
    }

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
        let url = std::env::var("REDIS_URL").unwrap();
        let client = Client::open(url).unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        tokio::task::spawn(async move {
            let options = streams::StreamReadOptions::default().block(250).count(5);
            let mut id = "$".to_string();

            loop {
                if tx.is_closed() {
                    break;
                }
                let response: redis::RedisResult<streams::StreamReadReply> = con
                    .xread_options(&[Self::key_messages(room_id.clone())], &[id.clone()], &options)
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
pub struct Message {
    pub id: Option<String>,
    pub from: String,
    pub message: String,
}

impl Into<Vec<(String, String)>> for Message {
    fn into(self) -> Vec<(String, String)> {
        let mut args: Vec<(String, String)> = Vec::new();
        args.push(("from".to_string(), self.from));
        args.push(("message".to_string(), self.message));

        args
    }
}

impl TryFrom<&redis::streams::StreamId> for Message {
    type Error = &'static str;

    fn try_from(stream_id: &redis::streams::StreamId) -> Result<Self, Self::Error> {
        let id = Some(stream_id.id.clone());

        let from = util::read_redis_stream_data(stream_id, "from")?;
        let message = util::read_redis_stream_data(stream_id, "message")?;

        Ok(Message { id, from, message })
    }
}

mod util {
    type Error = &'static str;

    pub fn read_redis_stream_data(
        stream_id: &redis::streams::StreamId,
        field: &str,
    ) -> Result<String, Error> {
        match stream_id.map.get(field).ok_or("Missing mandatory field")? {
            redis::Value::Data(bytes) => {
                let string =
                    String::from_utf8(bytes.clone()).or(Err("Failed utf8 conversion on field"))?;
                Ok(string)
            }
            _ => Err("Wrong type for field"),
        }
    }
}