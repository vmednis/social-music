use crate::spotify;
use serde::Serialize;

pub fn shorten_track(track: &spotify::tracks::Track) -> ShortTrack {
    //Extract artists
    let artists: Vec<String> = track
        .artists
        .iter()
        .map(|artist| artist.name.clone())
        .collect();

    //Extract smallest album cover image
    let mut images: Vec<(u32, String)> = track
        .album
        .images
        .iter()
        .map(|image| (image.width * image.height, image.url.clone()))
        .collect();
    images.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
    let (_, image) = &images[0];

    ShortTrack {
        name: track.name.clone(),
        preview_url: track.preview_url.clone().unwrap(),
        uri: track.uri.clone(),
        artists: artists,
        cover: image.clone(),
    }
}

#[derive(Debug, Serialize)]
pub struct ShortTrack {
    pub name: String,
    pub preview_url: String,
    pub uri: String,
    pub artists: Vec<String>,
    pub cover: String,
}
