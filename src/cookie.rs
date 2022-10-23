use sodiumoxide::crypto::secretbox;
use warp::Filter;

pub fn with_user() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
  warp::cookie("userid").map(|cookie| decrypt_cookie(cookie))
}

pub fn gen_user(user_id: String) -> String {
  encrypt_cookie(user_id)
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