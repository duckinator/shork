mod config;
mod jellyfin;

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::load("config.toml")?;
    let client = jellyfin::Client::new(config);

    let artists = client.artist_albums().await?;

    for artist in artists.iter() {
        println!("{}", artist.name);
        for album in artist.albums.iter() {
            println!("- {}", album.name);
        }
    }

    Ok(())
}
