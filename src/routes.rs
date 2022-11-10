use crate::cookie;
use crate::db;
use crate::db::Db;
use crate::endpoint;
use crate::spotify;
use crate::spotify::Spotify;
use std::collections::HashMap;
use warp::Filter;

pub fn routes(
    db: Db,
    spotify: Spotify,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let login = warp::path("login")
        .and(warp::get())
        .and_then(endpoint::get_login);
    let authorize = warp::path("authorize")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and(spotify::with(spotify.clone()))
        .and(db::with(db.clone()))
        .and_then(endpoint::get_authorize);
    let logout = warp::path("logout")
        .and(warp::get())
        .and_then(endpoint::get_logout);
    let chat = warp::path!("chat" / String)
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(spotify::with(spotify.clone()))
        .and(warp::ws())
        .and_then(endpoint::ws_chat);
    let test = warp::path("test")
        .and(warp::get())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and_then(endpoint::get_test_endpoint);
    let token = warp::path("token")
        .and(warp::get())
        .and(cookie::with_user())
        .and(spotify::with(spotify.clone()))
        .and(db::with(db.clone()))
        .and_then(endpoint::get_token);
    let create_room = warp::path!("room" / "new")
        .and(warp::post())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(warp::body::json())
        .and_then(endpoint::post_room_new);

    login
        .or(authorize)
        .or(logout)
        .or(test)
        .or(token)
        .or(chat)
        .or(create_room)
        .or(routes_static())
}

fn routes_static() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
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

    assets.or(robots).or(icon).or(default)
}
