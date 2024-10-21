use crate::config::Config;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    album_artist: Option<String>,
    name: String,
    //server_id: String,
    id: String,
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Items {
    items: Vec<Item>,
}

#[derive(Debug)]
pub struct Album {
    pub name: String,
    pub id: String,
    pub artist_name: String,
}

impl Album {
    pub fn new(artist_name: &str, album: &Item) -> Self {
        Self {
            name: album.name.clone(),
            id: album.id.clone(),
            artist_name: artist_name.to_string(),
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

            let album = Album::new(artist_name, album);

            artists.get_mut(artist_name).expect("artists HashMap should have specified artist").push(album);
        }

        Ok(artists)
    }

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
    //
    // Image
    // /Artists/{name}/Images/{imageType}/{imageIndex}
    // /Items/{itemId}/Images
    // /Items/{itemId}/Images/{imageType}
    // /Items/{itemId}/Images/{imageType}/{imageIndex}
    //
    // /Items
}

