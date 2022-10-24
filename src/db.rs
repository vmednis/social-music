use redis::{Client, AsyncCommands};
use redis::streams;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use warp::Filter;
use futures_util::future;

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

    Self {
      client
    }
  }

  fn key_auth(user_id: String) -> String {
    format!("{}:auth", user_id)
  }

  pub async fn set_auth(&mut self, user_id: String, token: String) {
    let mut con = self.client.get_async_connection().await.unwrap();
    let _ : () = con.set(Self::key_auth(user_id), token).await.unwrap();
  }

  pub async fn get_auth(&mut self, user_id: String) -> Option<String> {
    let mut con = self.client.get_async_connection().await.unwrap();
    con.get(Self::key_auth(user_id)).await.ok()
  }

  fn key_messages() -> String {
    format!("messages")
  }

  pub async fn add_message(&mut self, message: Message) {
    let mut args: Vec<(String, String)> = Vec::new();
    args.push(("from".to_string(), message.from));
    args.push(("message".to_string(), message.message));

    let mut con = self.client.get_async_connection().await.unwrap();
    let _ : () = con.xadd(Self::key_messages(), "*", &args[..]).await.unwrap();
  }

  //TODO: MAKE SURE IT GETS CLOSED!!!!
  pub async fn subscribe_messages(&mut self) -> mpsc::Receiver<Message> {
    let (tx, rx) = mpsc::channel(10);
    let url = std::env::var("REDIS_URL").unwrap();
    let client = Client::open(url).unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    tokio::task::spawn(async move {
      let options = streams::StreamReadOptions::default()
        .block(250)
        .count(5);
      let mut id = "$".to_string();

      loop {
        if tx.is_closed() {
          log::info!("tx closed");
          break;
        }
        let response: redis::RedisResult<streams::StreamReadReply> = con.xread_options(&[Self::key_messages()], &[id.clone()], &options).await;

        let mut sends = Vec::new();
        match response {
          Ok(reply) => {
            id = reply.keys.iter().map(|stream_key| {
              if stream_key.key == Self::key_messages() {
                stream_key.ids.iter().map(|stream_id| {
                  let mut from = "unknown".to_string();
                  let mut message = "unknown".to_string();

                  if let redis::Value::Data(text) = stream_id.map.get("from").unwrap() {
                    from = String::from_utf8(text.clone()).unwrap();
                  }
                  if let redis::Value::Data(text) = stream_id.map.get("message").unwrap() {
                    message = String::from_utf8(text.clone()).unwrap();
                  }

                  let message = Message {
                    from,
                    message
                  };

                  sends.push(tx.send(message));

                  stream_id.id.clone()
                }).fold("$".to_string(), |_, val| val)
              } else {
                id.clone()
              }
            }).fold(id.clone(), |acc, val| {
              if val.len() >= acc.len() {
                val
              } else {
                acc
              }
            });
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
  pub from: String,
  pub message: String,
}