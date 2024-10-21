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
    Home,
    ArtistList,
    Album(Album),
    Artist(String),
}

struct ShorkApp {
    // Sender/Receiver for async notifications.
    tx: Sender<HashMap<String, Vec<Album>>>,
    rx: Receiver<HashMap<String, Vec<Album>>>,

    config: Config,

    artists: HashMap<String, Vec<Album>>,

    show_config: bool,
    fetching_data: bool,

    view: View,
}

impl ShorkApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut slf = Self {
            tx: tx,
            rx: rx,
            config: Config::default(),
            artists: HashMap::new(),
            show_config: false,
            fetching_data: false,
            view: View::Home,
        };

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
            self.fetching_data = false;
        }

        if self.config.server.is_empty() || self.config.token.is_empty() {
            self.show_config = true;
        }

        egui::TopBottomPanel::bottom("bottom-panel").show(ctx, |ui| {
            ui.label("bottom");

            if ui.button("Configure").clicked() {
                self.show_config = true;
            }
        });

        egui::SidePanel::left("left-panel").show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                if ui.button("Home").clicked() {
                    self.view = View::Home;
                }

                if ui.button("Artists").clicked() {
                    self.view = View::ArtistList;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.fetching_data {
                self.view_fetching(ctx, ui);
                return;
            }

            if self.show_config {
                self.view_config(ctx, ui);
                return;
            }

            match self.view.clone() {
                View::Album(album) => self.view_album(ctx, ui, &album),
                View::Artist(artist_name) => self.view_artist(ctx, ui, &artist_name),
                View::ArtistList => self.view_artist_list(ctx, ui),
                View::Home => self.view_home(ctx, ui),
            }
        });
    }
}

impl ShorkApp {
    fn view_config(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.label("Server");
        ui.text_edit_singleline(&mut self.config.server);
        ui.label("API Token");
        ui.text_edit_singleline(&mut self.config.token);
        if ui.button("Refresh Data").clicked() {
            self.fetch_data(ctx);
            self.show_config = false;
            self.fetching_data = true;
        }
    }

    fn view_fetching(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("Fetching data from server.");
    }

    fn view_home(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("TODO");
        self.view = View::ArtistList;
    }

    fn view_artist_list(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (artist_name, _albums) in &self.artists {
                    let btn = egui::Button::opt_image_and_text(None, Some(artist_name.into()))
                        .wrap_mode(egui::TextWrapMode::Truncate);

                    if ui.add_sized(egui::vec2(100.0, 100.0), btn).clicked() {
                        self.view = View::Artist(artist_name.clone());
                    }
                }
            });
        });
    }

    fn view_artist(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, artist_name: &str) {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true), |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for album in &self.artists[artist_name] {
                    let btn = egui::Button::opt_image_and_text(None, Some(album.name.clone().into()))
                        .wrap_mode(egui::TextWrapMode::Truncate);
                    if ui.add_sized(egui::vec2(100.0, 100.0), btn).clicked() {
                        self.view = View::Album(album.clone());
                    }
                }
            });
        });
    }

    fn view_album(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui, album: &Album) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(&album.name);
            if ui.button(&album.artist_name).clicked() {
                self.view = View::Artist(album.artist_name.clone());
            }

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                for track in &album.tracks {
                    if ui.button(&track.name).clicked() {
                        println!("{:?}", track.stream_url);
                    }
                }
            });
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

