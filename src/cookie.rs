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

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;

    fn gen_cookie_key() {
        let cookie_key = "0".to_string().repeat(secretbox::KEYBYTES);
        std::env::set_var("COOKIE_KEY", cookie_key);
    }

    #[test]
    fn test_decrypting_encrypted_data_should_return_same_data() {
        gen_cookie_key();
        let input_data = "this:is:an:user:id".to_string();

        let encrypted = encrypt_cookie(input_data.clone());
        let decrypted = decrypt_cookie(encrypted.clone());

        assert_ne!(encrypted, input_data);
        assert_eq!(decrypted, input_data);
    }

    #[tokio::test]
    async fn test_cookie_creation_and_extraction() {
        gen_cookie_key();
        let user_id = "this:is:user:data".to_string();

        let test_filter = warp::get()
            .and(with_user())
            .and_then(|cookie| async {
                let res: Result<String, Infallible>  = Ok(cookie);
                res
            });

        let cookie = gen_user(user_id.clone());
        let res = warp::test::request()
            .header("Cookie", format!("userid={}", cookie))
            .reply(&test_filter)
            .await;

        assert_eq!(res.body(), user_id.as_bytes());
    }
}