use db::Db;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{ws::Ws, Filter};

mod cookie;
mod db;
mod socket;
mod spotify;

#[tokio::main]
async fn main() {
    env_logger::init();

    let db = db::connect_db();
    let spotify = spotify::init();

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
        .and(spotify::with(spotify.clone()))
        .and(db::with(db.clone()))
        .and_then(handle_authorize);
    let chat = warp::path!("chat" / String)
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(spotify::with(spotify.clone()))
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

    tokio::task::spawn(async move {
        loop {
            let mut inner_db = db.lock().await;
            let rx = inner_db.claim_room().await;
            std::mem::drop(inner_db);

            if let Some(room_id) = rx.await.unwrap() {
                let inner_db = db.clone();
                tokio::task::spawn(async move {
                    log::info!("Claimed room {}", room_id.clone());

                    loop {
                        let mut inner_db = inner_db.lock().await;
                        inner_db.keep_alive_room_claim(room_id.clone()).await;
                        std::mem::drop(inner_db);

                        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                    }
                });
            }
        }
    });

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
        .path_and_query(format!("/authorize?response_type=code&client_id={}&redirect_uri={}&state=not-used&scope={}&show_dialog=true",
            client_id,
            return_url,
            "user-modify-playback-state+streaming"
        ))
        .build()
        .unwrap();

    Ok(warp::redirect::see_other(spotify_uri))
}

async fn handle_authorize(
    query: HashMap<String, String>,
    spotify: spotify::Spotify,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    let return_url = "http://127.0.0.1:3030/authorize".to_string();
    let code = query.get("code").unwrap();

    let spotify = spotify.lock().await;
    let token = spotify.request_auth_new(code.clone(), return_url).await;
    let user = spotify.request_me(token.access_token.clone()).await;
    std::mem::drop(spotify);

    let mut db = db.lock().await;
    db.set_auth(user.uri.clone(), token.access_token.clone())
        .await;
    std::mem::drop(db);

    let cookie = cookie::gen_user(user.uri);
    let redirect = warp::redirect::see_other(warp::http::Uri::from_static("/"));
    let reply = warp::reply::with_header(redirect, "Set-Cookie", format!("userid={}", cookie));

    Ok(reply)
}

async fn create_room(
    user_id: String,
    db: Db,
    body: HashMap<String, String>,
) -> Result<impl ::warp::Reply, Infallible> {
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
        }
        Err(errors) => Ok(warp::reply::json(&errors)),
    }
}

async fn handle_chat(
    room_id: String,
    user_id: String,
    db: Db,
    spotify: spotify::Spotify,
    ws: Ws,
) -> Result<impl warp::Reply, Infallible> {
    Ok(ws.on_upgrade(move |websocket| socket::connected(websocket, room_id, user_id, db, spotify)))
}
