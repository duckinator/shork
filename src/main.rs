#![forbid(unsafe_code)]

mod config;
mod jellyfin;

use config::Config;
use eframe::egui;
use jellyfin::Album;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

fn main() {
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
        Box::new(|cc| Ok(Box::new(ShorkApp::new(cc)))),
    );
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
enum View {
    Fetching,
    Config,
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let artists = HashMap::new();

        let view = View::Home;

        let config = Config::default();

        let mut slf = Self { tx, rx, config, artists, view };

        if let Some(storage) = cc.storage {
            if let Some(config) = eframe::get_value(storage, "shork/config") {
                slf.config = config;
            }

            if let Some(artists) = eframe::get_value(storage, "shork/artists") {
                slf.artists = artists;
            }

            if let Some(view) = eframe::get_value(storage, "shork/view") {
                slf.view = view;
            }
        }

        slf
    }

    fn update_config(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Configuration");
            ui.label("Server");
            ui.text_edit_singleline(&mut self.config.server);
            ui.label("API Token");
            ui.text_edit_singleline(&mut self.config.token);
            if ui.button("Done").clicked() {
                self.fetch_data(ctx);
                self.view = View::Fetching;
            }
        });
    }

    fn fetch_data(&self, ctx: &egui::Context) {
        fetch_info(self.config.clone(), self.tx.clone(), ctx.clone());
    }
}

impl eframe::App for ShorkApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "shork/config", &self.config);
        eframe::set_value(storage, "shork/artists", &self.artists);
        eframe::set_value(storage, "shork/view", &self.view);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(artists) = self.rx.try_recv() {
            self.artists = artists;
            if self.view == View::Fetching {
                self.view = View::Home;
            }
        }

        if self.config.server.is_empty() || self.config.token.is_empty() {
            self.view = View::Config;
        }

        if self.view == View::Config {
            self.update_config(ctx);
            return;
        }

        egui::TopBottomPanel::top("top-panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                if ui.button("Configure").clicked() {
                    self.view = View::Config;
                }

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
                        View::Fetching => self.view_fetching(ctx, ui),
                        View::Config => { /* do nothing */ },
                    }
                });
            });
        });
    }
}

impl ShorkApp {
    fn view_fetching(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Fetching data from server.");
    }

    fn view_home(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
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

