use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    album_artist: Option<String>,
    pub name: String,
    //server_id: String,
    pub id: String,
    //channel_id: Option<String>,
    //is_folder: bool,
    //Type: String,
    //collection_type: Option<String>,
    //image_tags: HashMap<String, String>,
    //backdrop_image_tags: Vec<String>,
    //image_blur_hashes: HashMap<String, String>,
    //location_type: String,
    //media_type: String,

    //#[serde(rename = "Type")]
    //item_type: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Track {
    pub name: String,
    pub id: String,
    pub stream_url: String,
}

impl Track {
    fn new(item: &Item, stream_url: String) -> Self {
        Self {
            name: item.name.clone(),
            id: item.id.clone(),
            stream_url: stream_url.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
struct Items {
    items: Vec<Item>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Album {
    pub name: String,
    pub id: String,
    pub artist_name: String,
    pub tracks: Vec<Track>,
}

impl Album {
    pub fn new(artist_name: &str, album: &Item, tracks: Vec<Track>) -> Self {
        Self {
            name: album.name.clone(),
            id: album.id.clone(),
            artist_name: artist_name.to_string(),
            tracks: tracks,
        }
    }
}

pub struct Client {
    config: Config,
    client: reqwest::Client,
}

impl Client {
    pub fn new(config: Config) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    fn auth(&self) -> String {
        format!("MediaBrowser Token=\"{}\"", self.config.token)
    }

    pub async fn artist_albums(&self) -> Result<HashMap<String, Vec<Album>>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/Items?recursive=true&sortOrder=Ascending&includeItemTypes=MusicAlbum", self.config.server);
        let body = self.client.get(endpoint)
            .header("Authorization", self.auth())
            .send()
            .await?
            .text()
            .await?;

        let items: Items = serde_json::from_str(&body)?;

        let albums = items.items;

        let mut artists: HashMap<String, Vec<Album>> = HashMap::new();

        for album in albums.iter() {
            let artist_name: &str =
                if let Some(an) = album.album_artist.clone() {
                    &an.clone()
                } else {
                    "unknown"
                };

            if !artists.contains_key(artist_name) {
                artists.insert(artist_name.to_string(), vec![]);
            }

            let album = Album::new(artist_name, album, self.album_playlist(&album).await?);

            artists.get_mut(artist_name)
                .expect("artists[artist_name] should be created only a few lines before this")
                .push(album);
        }

        Ok(artists)
    }


    pub async fn album_playlist(&self, album: &Item) -> Result<Vec<Track>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/Items?parentId={}&includeItemTypes=Audio", self.config.server, album.id);
        let body = self.client.get(endpoint)
            .header("Authorization", self.auth())
            .send()
            .await?
            .text()
            .await?;

        let items: Items = serde_json::from_str(&body)?;

        let tracks: Vec<Track> = items.items.iter()
            .map(|item| Track::new(item, self.stream_url(item)))
            .collect();

        Ok(tracks)
    }

    pub fn stream_url(&self, item: &Item) -> String {
        format!("{}/Audio/{}/stream", self.config.server, item.id)
    }
    // Track information for Album: /Items?parentId={album.id}


    // Track information for Album: /Items?parentId={album.id}
    // Stream for track: /Audio/{track.id}/stream
    //

    // Artists
    // GET:
    // /Artists
    // /Artists/{name}
    // /Artists/AlbumArtists
    //
    // Audio
    // GET/HEAD:
    // /Audio/{itemId}/stream
    // /Audio/{itemId}/stream.{container}
    //
    // Audio
    // GET:
    // /Audio/{itemId}/hls1/{playlistId}/{segmentId}.{container}
    // /Audio/{itemId}/main.m3u8
    // /Audio/{itemId}/master.m3u8 (+HEAD)
    //
    // HlsSegment
    // /Audio/{itemId}/hls/{segmentId}/stream.aac
    // /Audio/{itemId}/hls/{segmentId}/stream.mp3
}

