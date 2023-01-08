use crate::cookie;
use crate::db;
use crate::db::Db;
use crate::socket;
use crate::spotify;
use crate::spotify::Spotify;
use std::collections::HashMap;
use std::convert::Infallible;
use warp::{ws::Ws, Reply};

pub async fn get_test_endpoint(user_id: String, db: Db) -> Result<impl Reply, Infallible> {
    //Used only for testing
    let mut db = db.lock().await;
    let key = db.get_auth(user_id.clone()).await;
    Ok(format!("Hello, {} your token is {:?}!", user_id, key))
}

pub async fn get_token(
    user_id: String,
    spotify: Spotify,
    db: Db,
) -> Result<impl Reply, Infallible> {
    //Get users token from database
    let mut inner_db = db.lock().await;
    let token = inner_db.get_auth(user_id.clone()).await.unwrap();
    std::mem::drop(inner_db);

    //Do a spotify request to see if token is still valid and refresh it if not
    let spotify = spotify.lock().await;
    spotify.request_me(token).await;
    std::mem::drop(spotify);

    let mut inner_db = db.lock().await;
    let token = inner_db.get_auth(user_id.clone()).await.unwrap();
    Ok(format!("{}", token.access_token))
}

pub async fn get_login() -> Result<impl Reply, Infallible> {
    let return_url = std::env::var("SPOTIFY_RETURN_URL").unwrap();
    let client_id = std::env::var("SPOTIFY_CLIENT_ID").unwrap();

    //Redirect user to a spotify authorization page
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

pub async fn get_authorize(
    query: HashMap<String, String>,
    spotify: Spotify,
    db: Db,
) -> Result<impl warp::Reply, Infallible> {
    let return_url = std::env::var("SPOTIFY_RETURN_URL").unwrap();
    let code = query.get("code").unwrap();

    //Acquire users tokens from spotify
    let spotify = spotify.lock().await;
    let token = spotify.request_auth_new(code.clone(), return_url).await;
    let access_token = token.access_token;
    let refresh_token = token.refresh_token.unwrap();
    let user = spotify
        .request_me(db::auth::Auth {
            user_id: None,
            access_token: access_token.clone(),
            refresh_token: refresh_token.clone(),
        })
        .await;
    std::mem::drop(spotify);

    //Save the users tokens to the database
    let mut db = db.lock().await;
    db.set_auth(
        user.uri.clone(),
        access_token.clone(),
        refresh_token.clone(),
    )
    .await;
    std::mem::drop(db);

    //Set the cookie and redirect user to /
    let cookie = cookie::gen_user(user.uri);
    let redirect = warp::redirect::see_other(warp::http::Uri::from_static("/"));
    let reply = warp::reply::with_header(redirect, "Set-Cookie", format!("userid={}", cookie));

    Ok(reply)
}

pub async fn get_logout() -> Result<impl Reply, Infallible> {
    //Set an expiration for the cookie and redirect to /
    let redirect = warp::redirect::see_other(warp::http::Uri::from_static("/"));
    let reply = warp::reply::with_header(
        redirect,
        "Set-Cookie",
        "userid=expired; expires=Thu, 01 Jan 1970 00:00:00 GMT",
    );

    Ok(reply)
}

pub async fn create_room(
    user_id: String,
    db: Db,
    body: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    //Extract room information fromm parameters
    let id = body.get("id").unwrap_or(&"".to_string()).clone();
    let title = body.get("title").unwrap_or(&"".to_string()).clone();

    let room = db::room::Room {
        id,
        title,
        owner: user_id.clone(),
    };

    //Try to insert the room into the database
    let mut db = db.lock().await;
    match db.create_room(room).await {
        Ok(_) => {
            //If successfuly reply with no errors
            let reply: Vec<String> = Vec::new();
            let json = warp::reply::json(&reply);
            Ok(warp::reply::with_status(
                json,
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(errors) => {
            //If failed to insert room (i.e. failing validation) reply with errors
            let json = warp::reply::json(&errors);
            Ok(warp::reply::with_status(
                json,
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    }
}

pub async fn list_rooms(_user_id: String, db: Db) -> Result<warp::reply::Response, Infallible> {
    //Acquire room information from the database
    let mut db = db.lock().await;
    let rooms = db.list_rooms().await;

    Ok(warp::reply::json(&rooms).into_response())
}

pub async fn get_room(
    room_id: String,
    _user_id: String,
    db: Db,
) -> Result<warp::reply::Response, Infallible> {
    //Acquire information about the room from database
    let mut db = db.lock().await;
    let room = db.get_room(room_id).await;

    //Differnet replies based on if the room was found
    match room {
        Some(room) => Ok(warp::reply::json(&room).into_response()),
        None => Ok(warp::reply::with_status(
            "Couldn't find room with this id.",
            warp::http::StatusCode::NOT_FOUND,
        )
        .into_response()),
    }
}

pub async fn ws_chat(
    room_id: String,
    user_id: String,
    db: Db,
    spotify: Spotify,
    ws: Ws,
) -> Result<impl warp::Reply, Infallible> {
    //Finish connecting the websocket
    Ok(ws.on_upgrade(move |websocket| socket::connected(websocket, room_id, user_id, db, spotify)))
}

pub async fn get_search(
    user_id: String,
    db: Db,
    spotify: Spotify,
    query: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    //Extract query from query parameters
    let query = query.get("q");

    match query {
        Some(query) => {
            //Get users token from database
            let mut db = db.lock().await;
            let token = db.get_auth(user_id).await.unwrap();
            std::mem::drop(db);

            //Perform a search on spotify
            let spotify = spotify.lock().await;
            let results = spotify
                .request_search(
                    token,
                    query.clone(),
                    "track".to_string(),
                    None,
                    Some(20),
                    Some(0),
                    None,
                )
                .await;
            std::mem::drop(spotify);

            //Only keep the useful information from search results
            let response: Vec<spotify::util::ShortTrack> = results
                .tracks
                .items
                .iter()
                .map(|track| {
                    spotify::util::shorten_track(track)
                })
                .collect();

            Ok(warp::reply::json(&response).into_response())
        }
        None => Ok(warp::reply::with_status(
            "Missing query parameter",
            warp::http::StatusCode::BAD_REQUEST,
        )
        .into_response()),
    }
}

pub async fn list_user_queue(
    room_id: String,
    user_id: String,
    db: Db,
    spotify: Spotify,
) -> Result<warp::reply::Response, Infallible> {
    //Acquire list of ids from database
    let mut db = db.lock().await;
    let token = db.get_auth(user_id.clone()).await.unwrap();
    let track_ids = db.list_user_queue(room_id, user_id).await;
    std::mem::drop(db);

    let response: Vec<spotify::util::ShortTrack> = if !track_ids.is_empty() {
        //Get information on these ids from spotify
        let spotify = spotify.lock().await;
        let results = spotify.request_tracks(token, track_ids).await;

        //Only return the useful part of the response
        let response: Vec<spotify::util::ShortTrack> = results
            .tracks
            .iter()
            .map(|track| {
                spotify::util::shorten_track(track)
            })
            .collect();

        response
    } else {
        //If is empty or doesn't exist then return an empty list
        Vec::new()
    };

    Ok(warp::reply::json(&response).into_response())
}