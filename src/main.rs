use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

    let root = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./www/index.html"));
    let assets = warp::path("assets")
        .and(warp::get())
        .and(warp::fs::dir("./www/assets/"));
    let login = warp::path("login").and(warp::get()).and_then(handle_login);
    let authorize = warp::path("authorize")
        .and(warp::get())
        .and(warp::query::<HashMap<String, String>>())
        .and_then(handle_authorize);

    let routes = root.or(assets).or(login).or(authorize);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
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
            scope="user-modify-playback-state"
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
struct PlayerPlayData {
    #[serde(skip_serializing_if = "Option::is_none")]
    context_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uris: Option<Vec<String>>,
    position_ms: u32,
}

async fn handle_authorize(query: HashMap<String, String>) -> Result<impl warp::Reply, Infallible> {
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

    let track_data = PlayerPlayData {
        context_uri: None,
        uris: Some(vec!["spotify:track:3ZzxtumoIENCi16HAKuiLU".to_string()]),
        position_ms: 0,
    };

    let response = client
        .put("https://api.spotify.com/v1/me/player/play?device_id=75a716edd5637746e7dc90293bab9fe7eeaa699c")
        .bearer_auth(response.access_token)
        .json(&track_data)
        .send()
        .await
        .unwrap();

    println!("{:?}", response);

    Ok(warp::redirect::see_other(warp::http::Uri::from_static("/")))
}
