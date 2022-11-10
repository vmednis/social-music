use crate::db::{Db, auth::Auth};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

mod auth;
mod me;
mod play;
mod tracks;

pub type Spotify = Arc<Mutex<SpotifyInternal>>;

pub fn init(db: Db) -> Spotify {
    Arc::new(Mutex::new(SpotifyInternal::init(db)))
}

pub fn with(
    spotify: Spotify,
) -> impl Filter<Extract = (Spotify,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || spotify.clone())
}

pub struct SpotifyInternal {
    client_id: String,
    client_secret: String,
    http_client: Client,
    db: Db,
}

impl SpotifyInternal {
    fn init(db: Db) -> Self {
        let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap();
        let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET").unwrap();

        let http_client = reqwest::ClientBuilder::new()
            .connection_verbose(true)
            .build()
            .unwrap();

        Self {
            client_id,
            client_secret,
            http_client,
            db
        }
    }

    async fn request<T: DeserializeOwned>(&self, request: impl SpotifyRequest) -> Option<T> {
        let response = self.perform_request(&request).await;

        let response = if request.token().is_some() && response.status() == 401 {
            //Bad or expired token, try refreshing it
            log::info!("Refreshing token!!!!!");
            self.refresh_token(request.token().unwrap()).await;
            self.perform_request(&request).await
        } else {
            response
        };

        if request.has_result() {
            Some(response.json().await.unwrap())
        } else {
            None
        }
    }

    async fn perform_request(&self, request: &impl SpotifyRequest) -> reqwest::Response {
        let http_client = &self.http_client;

        let endpoint = request.endpoint();
        let mut builder = match request.method() {
            SpotifyMethod::Get => http_client.get(endpoint),
            SpotifyMethod::Post => http_client.post(endpoint),
            SpotifyMethod::Put => http_client.put(endpoint),
        };

        if request.basic_auth() {
            builder = builder.basic_auth(self.client_id.clone(), Some(self.client_secret.clone()))
        }

        if let Some(token) = request.token() {
            builder = builder.bearer_auth(token.access_token);
        }

        if let Some(data) = request.form_data() {
            builder = builder.form(&data[..]);
        }

        if let Some(data) = request.json_data() {
            builder = builder.json(&data);
        }

        builder.send().await.unwrap()
    }

    async fn refresh_token(&self, token: Auth) {
        let data = Self::request_auth_refresh(&self, token.refresh_token.clone()).await;

        let access_token = data.access_token;
        let refresh_token = data.refresh_token.unwrap_or(token.refresh_token);

        let mut db = self.db.lock().await;
        db.set_auth(token.user_id.unwrap(), access_token, refresh_token).await;
    }
}

enum SpotifyMethod {
    Get,
    Post,
    Put,
}

trait SpotifyRequest {
    type JSONDataType: Serialize;

    fn endpoint(&self) -> String;
    fn method(&self) -> SpotifyMethod;
    fn basic_auth(&self) -> bool;
    fn token(&self) -> Option<Auth>;
    fn form_data(&self) -> Option<Vec<(&str, &str)>>;
    fn json_data(&self) -> Option<Self::JSONDataType>;
    fn has_result(&self) -> bool;
}
