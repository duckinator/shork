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

#[derive(Clone)]
enum View {
    Home,
    Album(Album),
    Artist(String),
}

struct ShorkApp {
    // Sender/Receiver for async notifications.
    tx: Sender<HashMap<String, Vec<Album>>>,
    rx: Receiver<HashMap<String, Vec<Album>>>,

    config: Config,

    artists: HashMap<String, Vec<Album>>,

    view: View,
}

impl ShorkApp {
    fn new(config: Config) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let artists = HashMap::new();

        let view = View::Home;

        Self { tx, rx, config, artists, view }
    }
}

impl eframe::App for ShorkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(artists) = self.rx.try_recv() {
            self.artists = artists;
        }

        egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                if ui.button("Refresh").clicked() {
                    fetch_info(self.config.clone(), self.tx.clone(), ctx.clone());
                }

                if ui.button("Artists").clicked() {
                    self.view = View::Home;
                }

                if let View::Artist(artist_name) = &self.view {
                    if ui.button(artist_name).clicked() {
                        self.view = View::Artist(artist_name.clone());
                    }
                }

                if let View::Album(album) = &self.view {
                    let name = album.name.clone();
                    if ui.button(&album.artist_name).clicked() {
                        self.view = View::Artist(album.artist_name.clone());
                    }

                    let _ = ui.button(&name);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.label("bottom");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true), |ui| {
                    match self.view.clone() {
                        View::Album(album) => self.view_album(ctx, ui, &album),
                        View::Artist(artist_name) => self.view_artist(ctx, ui, &artist_name),
                        View::Home => self.view_home(ctx, ui),
                    }
                });
            });
        });
    }
}

impl ShorkApp {
    fn view_home(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        for (artist_name, _albums) in &self.artists {
            let btn = egui::Button::opt_image_and_text(None, Some(artist_name.into()))
                .wrap_mode(egui::TextWrapMode::Truncate);

            if ui.add_sized(egui::vec2(100.0, 100.0), btn).clicked() {
                self.view = View::Artist(artist_name.clone());
            }
        }
    }

    fn view_artist(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, artist_name: &str) {
        for album in &self.artists[artist_name] {
            let btn = egui::Button::opt_image_and_text(None, Some(album.name.clone().into()))
                .wrap_mode(egui::TextWrapMode::Truncate);
            if ui.add_sized(egui::vec2(100.0, 100.0), btn).clicked() {
                self.view = View::Album(album.clone());
            }
        }
    }

    fn view_album(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, album: &Album) {
        for track in &album.tracks {
            if ui.button(&track.name).clicked() {
                println!("{:?}", track.stream_url);
            }
        }
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

