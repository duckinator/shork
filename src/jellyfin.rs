use crate::config::Config;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
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
    media_type: String,

    #[serde(flatten)]
    item_type: ItemVariant,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(tag = "Type")]
pub enum ItemVariant {
    Audio,
    Folder,
    ManualPlaylistsFolder,
    MusicAlbum,
    MusicArtist,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Items {
    items: Vec<Item>,
}

#[derive(Debug)]
pub struct Artist {
    pub name: String,
    pub id: String,
    pub albums: Vec<Album>,
}

impl Artist {
    pub fn new(artist: &Item, albums: Vec<Album>) -> Self {
        Self {
            name: artist.name.clone(),
            id: artist.id.clone(),
            albums: albums,
        }
    }
}

#[derive(Debug)]
pub struct Album {
    pub name: String,
    pub id: String,
    pub artist_name: String,
    pub artist_id: String,
}

impl Album {
    pub fn new(artist: &Item, album: &Item) -> Self {
        Self {
            name: album.name.clone(),
            id: album.id.clone(),
            artist_name: artist.name.clone(),
            artist_id: artist.id.clone(),
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

    pub async fn toplevel_item(&self, variant: ItemVariant, name: &str) -> Result<Item, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/Items", self.config.server);
        let body = self.client.get(endpoint)
            .header("Authorization", self.auth())
            .send()
            .await?
            .text()
            .await?;

        let items: Items = serde_json::from_str(&body)?;
        let folders: Vec<&Item> = items.items.iter()
            .filter(|x| x.item_type == variant && x.name == name)
            .collect();

        let no_match: Box<dyn std::error::Error> = format!("No folder named {}", name).into();
        folders.get(0).ok_or(no_match).copied().cloned()
    }

    pub async fn items(&self, parent_id: &str, variant: ItemVariant) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
        let endpoint = format!("{}/Items?parentId={}", self.config.server, parent_id);
        let body = self.client.get(endpoint)
            .header("Authorization", self.auth())
            .send()
            .await?
            .text()
            .await?;

        let items: Items = serde_json::from_str(&body)?;
        Ok(items.items.iter()
            .cloned()
            .filter(|x| x.item_type == variant)
            .collect()
        )
    }

    pub async fn artist_albums(&self) -> Result<Vec<Artist>, Box<dyn std::error::Error>> {
        let parent = self.toplevel_item(ItemVariant::Folder, "Music").await?;

        let artists = self.items(&parent.id, ItemVariant::MusicArtist).await?;

        let mut results: Vec<Artist> = vec![];

        for artist in artists.iter() {
            let mut albums: Vec<Album> = vec![];
            for album in self.items(&artist.id, ItemVariant::MusicAlbum).await? {
                albums.push(Album::new(&artist, &album));
            }
            results.push(Artist::new(&artist, albums));
        }

        Ok(results)
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

