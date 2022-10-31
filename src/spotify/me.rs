use crate::spotify;
use serde::{Deserialize, Serialize};

struct SpotifyRequestMe {
    token: String,
}

impl spotify::SpotifyRequest for SpotifyRequestMe {
    type JSONDataType = ();

    fn endpoint(&self) -> String {
        "https://api.spotify.com/v1/me".to_string()
    }

    fn method(&self) -> spotify::SpotifyMethod {
        spotify::SpotifyMethod::Get
    }

    fn basic_auth(&self) -> bool {
        false
    }

    fn token(&self) -> Option<String> {
        Some(self.token.clone())
    }

    fn form_data(&self) -> Option<Vec<(&str, &str)>> {
        None
    }

    fn json_data(&self) -> Option<Self::JSONDataType> {
        None
    }

    fn has_result(&self) -> bool {
        true
    }
}

impl spotify::SpotifyInternal {
    pub async fn request_me(&self, token: String) -> User {
        let req = SpotifyRequestMe { token };

        self.request(req).await.unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserExplicitContent {
    pub filter_enabled: bool,
    pub filter_locked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFollowers {
    pub href: Option<String>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserImages {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub country: Option<String>,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub explicit_content: Option<UserExplicitContent>,
    pub external_urls: UserExternalUrls,
    pub followers: UserFollowers,
    pub href: String,
    pub images: Vec<UserImages>,
    pub product: Option<String>,
    #[serde(rename = "type")]
    pub obj_type: String,
    pub uri: String,
}
