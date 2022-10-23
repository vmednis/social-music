use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use warp::Filter;
use sodiumoxide::crypto::secretbox;

#[tokio::main]
async fn main() {
    env_logger::init();

    let default = warp::get()
        .and(warp::fs::file("./www/index.html"));
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
        .and_then(handle_authorize);

    let test = warp::path("test")
        .and(warp::get())
        .and(warp::cookie("userid"))
        .and_then(test_endpoint);

    let routes = assets.or(robots).or(icon).or(login).or(authorize).or(test).or(default);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn test_endpoint(cookie: String) -> Result<impl warp::Reply, Infallible> {
    Ok(format!("Hello, {}!", decrypt_cookie(cookie)))
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

    log::info!("{:?}", user);

    let track_data = PlayerPlayData {
        context_uri: None,
        uris: Some(vec!["spotify:track:3ZzxtumoIENCi16HAKuiLU".to_string()]),
        position_ms: 0,
    };

    client
        .put("https://api.spotify.com/v1/me/player/play?device_id=75a716edd5637746e7dc90293bab9fe7eeaa699c")
        .bearer_auth(access_token.clone())
        .json(&track_data)
        .send()
        .await
        .unwrap();

    let cookie = encrypt_cookie(user.uri);
    let redirect = warp::redirect::see_other(warp::http::Uri::from_static("/"));
    let reply = warp::reply::with_header(redirect, "Set-Cookie", format!("userid={}", cookie));
    Ok(reply)
}

fn encrypt_cookie(data: String) -> String {
    let key_raw = std::env::var("COOKIE_KEY").unwrap();
    let key = secretbox::Key::from_slice(key_raw.as_bytes()).unwrap();
    let nonce = secretbox::gen_nonce();

    let chypertext = secretbox::seal(data.as_bytes(), &nonce, &key);

    let nonce_out = base64::encode(nonce);
    let chypertext_out = base64::encode(chypertext);

    format!("{}:{}", nonce_out, chypertext_out)
}

fn decrypt_cookie(cookie: String) -> String {
    let parts: Vec<&str> = cookie.split(":").collect();
    let nonce_in = base64::decode(parts[0]).unwrap();
    let chypertext_in = base64::decode(parts[1]).unwrap();

    let key_raw = std::env::var("COOKIE_KEY").unwrap();
    let key = secretbox::Key::from_slice(key_raw.as_bytes()).unwrap();
    let nonce = secretbox::Nonce::from_slice(&nonce_in).unwrap();

    String::from_utf8(secretbox::open(&chypertext_in, &nonce, &key).unwrap()).unwrap()
}