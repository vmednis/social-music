use redis::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

pub mod auth;
pub mod device;
pub mod message;
pub mod playing;
pub mod presence;
pub mod queue;
pub mod room;

pub type Db = Arc<Mutex<DbInternal>>;

pub fn connect_db() -> Db {
    Arc::new(Mutex::new(DbInternal::init()))
}

pub fn with(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub struct DbInternal {
    url: String,
    client: Client,
}

impl DbInternal {
    fn init() -> Self {
        let url = std::env::var("REDIS_URL").unwrap();
        let client = Client::open(url.clone()).unwrap();

        let internal = Self { url, client };
        internal.enable_keyspace_events();

        internal
    }

    fn blockable_client(&self) -> Client {
        Client::open(self.url.clone()).unwrap()
    }

    fn enable_keyspace_events(&self) {
        let mut con = self.client.get_connection().unwrap();
        let _: () = redis::cmd("CONFIG")
            .arg("set")
            .arg("notify-keyspace-events")
            .arg("Knxg")
            .query(&mut con)
            .unwrap();
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
