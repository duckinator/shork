#![forbid(unsafe_code)]

mod config;
mod jellyfin;

use config::Config;
use eframe::egui;
use jellyfin::Album;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

fn main() {
    let config = Config::load("config.toml").expect("Expected config.toml to exist and contain a valid config file");
    let rt = tokio::runtime::Runtime::new().expect("Should have created tokio Runtime.");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        });
    });

    let _ = eframe::run_native(
        "shork - jellyfin music player",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(ShorkApp::new(config)))),
    );
}

struct ShorkApp {
    // Sender/Receiver for async notifications.
    tx: Sender<HashMap<String, Vec<Album>>>,
    rx: Receiver<HashMap<String, Vec<Album>>>,

    config: Config,

    artists: HashMap<String, Vec<Album>>,
}

impl ShorkApp {
    fn new(config: Config) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let artists = HashMap::new();

        Self { tx, rx, config, artists }
    }
}

impl eframe::App for ShorkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(artists) = self.rx.try_recv() {
            self.artists = artists;

            for (name, albums) in &self.artists {
                println!("{}", name);
                for album in albums.iter() {
                    println!("- {}", album.name);
                    /*for track in album.tracks {
                        println!("- - {:?}", track);
                    }*/
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Fetch data").clicked() {
                fetch_info(self.config.clone(), self.tx.clone(), ctx.clone());
            }
        });
    }
}

fn fetch_info(config: Config, tx: Sender<HashMap<String, Vec<Album>>>, ctx: egui::Context) {
    // This gets run in the thread set up in main().
    tokio::spawn(async move {
        let client = jellyfin::Client::new(config);

        // Fetch artist + album information from the Jellyfin server.
        let artists = client.artist_albums().await
            .expect("Artist + album information should be available from the server");

        // Notify the GUI thread of the fetched data.
        let _ = tx.send(artists);

        ctx.request_repaint();
    });
}

