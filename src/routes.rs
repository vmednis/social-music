use crate::cookie;
use crate::db;
use crate::db::Db;
use crate::endpoint;
use crate::spotify;
use crate::spotify::Spotify;
use std::collections::HashMap;
use warp::Filter;
use warp::filters::BoxedFilter;

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

    login
        .or(authorize)
        .or(logout)
        .or(test)
        .or(token)
        .or(chat)
        .or(routes_api(db.clone()))
        .or(routes_static())
}

fn routes_static() -> BoxedFilter<(impl warp::Reply,)> {
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

    assets.or(robots).or(icon).or(default).boxed()
}

fn routes_api(
    db: Db,
) -> BoxedFilter<(impl warp::Reply,)> {
    warp::path("api")
        .and(warp::path("v1"))
        .and(
            routes_api_room(db.clone())
            .or(warp::path::end().map(|| "api")))
        .boxed()
}

fn routes_api_room(
    db: Db,
) -> BoxedFilter<(impl warp::Reply,)> {
    //POST /api/v1/rooms
    let post_room = warp::path::end()
        .and(warp::post())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and(warp::body::json())
        .and_then(endpoint::create_room);

    //GET /api/v1/rooms
    let get_rooms = warp::path::end()
        .and(warp::get())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and_then(endpoint::list_rooms);

    //GET /api/v1/rooms/{id}
    let get_room = warp::path::param::<String>()
        .and(warp::path::end())
        .and(warp::get())
        .and(cookie::with_user())
        .and(db::with(db.clone()))
        .and_then(endpoint::get_room);

    warp::path("rooms")
        .and(
            post_room.or(get_rooms).or(get_room)
            .or(warp::path::end().map(|| "room")))
        .boxed()
}