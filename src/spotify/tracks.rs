use crate::db::auth::Auth;
use crate::spotify;
use serde::Deserialize;

struct SpotfiyRequestTrack {
    token: Auth,
    track_id: String,
}

impl spotify::SpotifyRequest for SpotfiyRequestTrack {
    type JSONDataType = ();

    fn endpoint(&self) -> String {
        let short_id = self.track_id.split(":").last().unwrap();
        format!("https://api.spotify.com/v1/tracks/{}", short_id)
    }

    fn method(&self) -> spotify::SpotifyMethod {
        spotify::SpotifyMethod::Get
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
        None
    }

    fn has_result(&self) -> bool {
        true
    }
}

impl spotify::SpotifyInternal {
    pub async fn request_track(&self, token: Auth, track_id: String) -> Track {
        let req = SpotfiyRequestTrack { token, track_id };

        Self::request(&self, req).await.unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct Track {
    pub album: Album,
    pub artists: Vec<Artist>,
    pub available_markets: Vec<String>,
    pub disc_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_ids: ExternalIds,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub is_playable: Option<bool>,
    pub linked_from: Option<Box<Track>>,
    pub restrictions: Option<Restrictions>,
    pub is_local: bool,
    pub name: String,
    pub popularity: u32,
    pub preview_url: Option<String>,
    pub track_number: u32,
    #[serde(rename = "type")]
    pub obj_type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct Album {
    pub album_type: String,
    pub total_tracks: u32,
    pub available_markets: Vec<String>,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub restrictions: Option<Restrictions>,
    #[serde(rename = "type")]
    pub obj_type: String,
    pub uri: String,
    pub album_group: Option<String>,
    pub artists: Vec<Artist>,
}

#[derive(Debug, Deserialize)]
pub struct Artist {
    pub external_urls: ExternalUrls,
    pub followers: Option<Followers>,
    pub genres: Option<Vec<String>>,
    pub href: String,
    pub id: String,
    pub images: Option<Vec<Image>>,
    pub name: String,
    pub popularity: Option<u32>,
    #[serde(rename = "type")]
    pub obj_type: String,
    pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExternalUrls {
    pub spotify: String,
}

#[derive(Debug, Deserialize)]
pub struct Restrictions {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub url: String,
    pub height: u32,
    pub width: u32,
}

#[derive(Debug, Deserialize)]
pub struct Followers {
    pub href: String,
    pub total: u32,
}
