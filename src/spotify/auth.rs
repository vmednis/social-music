use crate::spotify;
use serde::{Deserialize, Serialize};

struct SpotifyRequestAuthNew {
    code: String,
    return_url: String,
}

impl spotify::SpotifyRequest for SpotifyRequestAuthNew {
    type JSONDataType = ();

    fn endpoint(&self) -> String {
        "https://accounts.spotify.com/api/token".to_string()
    }

    fn method(&self) -> spotify::SpotifyMethod {
        spotify::SpotifyMethod::Post
    }

    fn basic_auth(&self) -> bool {
        true
    }

    fn token(&self) -> Option<String> {
        None
    }

    fn form_data(&self) -> Option<Vec<(&str, &str)>> {
        let mut data = Vec::new();

        data.push(("code", &self.code[..]));
        data.push(("redirect_uri", &self.return_url[..]));
        data.push(("grant_type", "authorization_code"));

        Some(data)
    }

    fn json_data(&self) -> Option<Self::JSONDataType> {
        None
    }

    fn has_result(&self) -> bool {
        true
    }
}

impl spotify::SpotifyInternal {
    pub async fn request_auth_new(&self, code: String, return_url: String) -> AccessToken {
        let req = SpotifyRequestAuthNew { code, return_url };

        self.request(req).await.unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_in: u32,
    pub refresh_token: Option<String>,
}
