use crate::db::auth::Auth;
use crate::spotify;
use serde::Deserialize;

struct SpotifyRequestSearch {
    token: Auth,
    query: String,
    search_type: String,
    market: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    include_external: Option<bool>,
}

impl spotify::SpotifyRequest for SpotifyRequestSearch {
    type JSONDataType = ();

    fn endpoint(&self) -> String {
        let market = match self.market.clone() {
            Some(market) => format!("&market={}", market),
            None => "".to_string(),
        };

        let limit = match self.limit.clone() {
            Some(limit) => format!("&limit={}", limit),
            None => "".to_string(),
        };

        let offset = match self.offset.clone() {
            Some(offset) => format!("&offset={}", offset),
            None => "".to_string(),
        };

        let include_external = match self.include_external {
            Some(include_external) => format!("&include_external={}", include_external),
            None => "".to_string(),
        };

        format!(
            "https://api.spotify.com/v1/search?q={}&type={}{}{}{}{}",
            self.query, self.search_type, market, limit, offset, include_external
        )
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
    pub async fn request_search(
        &self,
        token: Auth,
        query: String,
        search_type: String,
        market: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
        include_external: Option<bool>,
    ) -> SearchResult {
        let req = SpotifyRequestSearch {
            token,
            query,
            search_type,
            market,
            limit,
            offset,
            include_external,
        };

        self.request(req).await.unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchResult {
    //TODO: Add support for other types than "tracks"
    pub tracks: SearchResultTracks,
}

#[derive(Debug, Deserialize)]
pub struct SearchResultTracks {
    pub href: String,
    pub items: Vec<spotify::tracks::Track>,
    pub limit: u32,
    pub offset: u32,
    pub previous: Option<String>,
    pub next: Option<String>,
    pub total: u32,
}
