use db::Db;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap};
use std::convert::Infallible;
use warp::{ws::Ws, Filter};

mod cookie;
mod db;
mod socket;

#[tokio::main]
async fn main() {
    env_logger::init();

    let db = db::connect_db();

    let default = warp::get().and(warp::fs::file("./www/index.html"));
    let assets = warp::path("assets")
        .and(warp::get())
        .and(warp::fs::dir("./www/assets/"));
    let robots = warp::path("robots.txt")
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::fs::file("./www/robots.txt"));
    let icon = warp::path("icon.svg")
        .and(warp::path::end())
        .and(warp::get())
        .and(warp::fs::file("./www/icon.svg"));
    let login = warp::path("login").and(warp::get()).and_then(handle_login);
    let authorize = warp::path("authorize")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(db::with(db.clone()))
        .and_then(handle_authorize);
    let chat = warp::path!("chat"/ String)
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(warp::ws())
        .and_then(handle_chat);
    let test = warp::path("test")
        .and(warp::get())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and_then(test_endpoint);
    let token = warp::path("token")
        .and(warp::get())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and_then(get_token);
    let create_room = warp::path!("room" / "new")
        .and(warp::post())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(warp::body::json())

        .and_then(create_room);

    let routes = assets
        .or(robots)
        .or(icon)
        .or(login)
        .or(authorize)
        .or(test)
        .or(token)
        .or(chat)
        .or(create_room)
        .or(default);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn test_endpoint(user_id: String, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut db = db.lock().await;
    let key = db.get_auth(user_id.clone()).await;
    Ok(format!("Hello, {} your token is {:?}!", user_id, key))
}

async fn get_token(user_id: String, db: Db) -> Result<impl warp::Reply, Infallible> {
    let mut db = db.lock().await;
    let key = db.get_auth(user_id.clone()).await;
    let token = key.unwrap();
    Ok(format!("{}", token))
}

async fn handle_login() -> Result<impl warp::Reply, Infallible> {
    let return_url = "http://127.0.0.1:3030/authorize";
    let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap();

    let spotify_uri = warp::http::Uri::builder()
        .scheme("https")
        .authority("accounts.spotify.com")
        .path_and_query(format!("/authorize?response_type=code&client_id={client_id}&redirect_uri={redirect_uri}&state=not-used&scope={scope}&show_dialog=true",
            client_id = client_id,
            redirect_uri = return_url,
            scope="user-modify-playback-state+streaming"
        ))
        .build()
        .unwrap();

    Ok(warp::redirect::see_other(spotify_uri))
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessToken {
    access_token: String,
    token_type: String,
    scope: Option<String>,
    expires_in: u32,
    refresh_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserExplicitContent {
    filter_enabled: bool,
    filter_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserExternalUrls {
    spotify: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserFollowers {
    href: Option<String>,
    total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserImages {
    url: String,
    height: Option<u32>,
    width: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    country: Option<String>,
    display_name: Option<String>,
    email: Option<String>,
    explicit_content: Option<UserExplicitContent>,
    external_urls: UserExternalUrls,
    followers: UserFollowers,
    href: String,
    images: Vec<UserImages>,
    product: Option<String>,
    #[serde(rename = "type")]
    obj_type: String,
    uri: String,
}

async fn handle_authorize(
    query: HashMap<String, String>,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    let return_url = "http://127.0.0.1:3030/authorize";

    let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap();
    let client_secret = std::env::var("SPOTIFY_CLIENT_SECRET").unwrap();
    let code = query.get("code").unwrap();

    let client = reqwest::ClientBuilder::new()
        .connection_verbose(true)
        .build()
        .unwrap();

    let response: AccessToken = client
        .post("https://accounts.spotify.com/api/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&[
            ("code", &code[..]),
            ("redirect_uri", return_url),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let access_token = response.access_token;

    let user: User = client
        .get("https://api.spotify.com/v1/me")
        .bearer_auth(access_token.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let mut db = db.lock().await;
    db.set_auth(user.uri.clone(), access_token.clone()).await;

    let cookie = cookie::gen_user(user.uri);
    let redirect = warp::redirect::see_other(warp::http::Uri::from_static("/"));
    let reply = warp::reply::with_header(redirect, "Set-Cookie", format!("userid={}", cookie));
    Ok(reply)
}

async fn create_room(user_id: String, db: Db, body: HashMap<String, String>) -> Result<impl::warp::Reply, Infallible> {
    let id = body.get("id").unwrap_or(&"".to_string()).clone();
    let title = body.get("title").unwrap_or(&"".to_string()).clone();

    let room = db::room::Room {
        id,
        title,
        owner: user_id.clone(),
    };

    let mut db = db.lock().await;
    match db.create_room(room).await {
        Ok(_) => {
            let reply: Vec<String> = Vec::new();
            Ok(warp::reply::json(&reply))
        },
        Err(errors) => {
            Ok(warp::reply::json(&errors))
        }
    }
}

async fn handle_chat(room_id: String, user_id: String, db: Db, ws: Ws) -> Result<impl warp::Reply, Infallible> {
    Ok(ws.on_upgrade(move |websocket| socket::connected(websocket, room_id, user_id, db)))
}
