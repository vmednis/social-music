use redis::{Client, Connection, Commands};
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

pub type Db = Arc<Mutex<DbInternal>>;

pub fn connect_db() -> Db {
  Arc::new(Mutex::new(DbInternal::init()))
}

pub fn with(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || db.clone())
}

pub struct DbInternal {
  con: Connection,
}

impl DbInternal {
  fn init() -> Self {
    let url = std::env::var("REDIS_URL").unwrap();
    let client = Client::open(url).unwrap();
    let con = client.get_connection().unwrap();

    Self {
      con
    }
  }

  fn key_auth(user_id: String) -> String {
    format!("{}:auth", user_id)
  }

  pub fn set_auth(&mut self, user_id: String, token: String) {
    let _ : () = self.con.set(Self::key_auth(user_id), token).unwrap();
  }

  pub fn get_auth(&mut self, user_id: String) -> Option<String> {
    self.con.get(Self::key_auth(user_id)).ok()
  }
}