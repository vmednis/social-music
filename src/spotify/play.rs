use crate::db::auth::Auth;
use crate::spotify;
use serde::Serialize;

struct SpotifyRequestPlay {
    token: Auth,
    device_id: String,
    uri: String,
    position: u32,
}

impl spotify::SpotifyRequest for SpotifyRequestPlay {
    type JSONDataType = PlayerPlayData;

    fn endpoint(&self) -> String {
        format!(
            "https://api.spotify.com/v1/me/player/play?device_id={}",
            self.device_id.clone()
        )
    }

    fn method(&self) -> spotify::SpotifyMethod {
        spotify::SpotifyMethod::Put
    }

    fn basic_auth(&self) -> bool {
        false
    }

    fn token(&self) -> Option<Auth> {
        Some(self.token.clone())
    }

    fn form_data(&self) -> Option<Vec<(&str, &str)>> {
        None
    }

    fn json_data(&self) -> Option<Self::JSONDataType> {
        Some(PlayerPlayData {
            context_uri: None,
            uris: Some(vec![self.uri.clone()]),
            position_ms: self.position.clone(),
        })
    }

    fn has_result(&self) -> bool {
        false
    }
}

impl spotify::SpotifyInternal {
    pub async fn request_play(&self, token: Auth, device_id: String, uri: String, position: u32) {
        let req = SpotifyRequestPlay {
            token,
            device_id,
            uri,
            position,
        };

        self.request(req).await.unwrap_or(());
    }
}

#[derive(Debug, Serialize)]
struct PlayerPlayData {
    #[serde(skip_serializing_if = "Option::is_none")]
    context_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uris: Option<Vec<String>>,
    position_ms: u32,
}
