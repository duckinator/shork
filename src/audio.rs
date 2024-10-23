use crate::config::Config;
use crate::jellyfin::{Album, Track};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use rodio::source::Source;
use std::collections::VecDeque;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug)]
pub enum AudioCommand {
    PlayTrack(Track),
    PlayAlbum(Album),
    QueueTrack(Track),
    QueueAlbum(Album),
    Pause,
    Resume,
    ClearQueue,
    Stop,
}

#[derive(Clone, Debug)]
pub enum AudioState {
    Playing(Track),
    Paused,
    Stopped,
}

pub struct Audio {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    command_rx: Receiver<AudioCommand>,
    state: AudioState,
    state_tx: Sender<AudioState>,
    config: Config,
    queue: VecDeque<Track>,
}

impl Audio {
    pub fn new(config: Config, command_rx: Receiver<AudioCommand>, state_tx: Sender<AudioState>) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let queue: VecDeque<Track> = VecDeque::new();

        let state = AudioState::Stopped;

        Self { stream, stream_handle, sink, command_rx, state, state_tx, config, queue }
    }

    fn process_command(&mut self, command: AudioCommand) {
        println!("{:?}", command);
        match command {
            AudioCommand::PlayTrack(track) => self.play_track(track),
            AudioCommand::PlayAlbum(album) => self.play_album(album),
            AudioCommand::QueueTrack(track) => self.queue_track(track),
            AudioCommand::QueueAlbum(album) => self.queue_album(album),
            AudioCommand::Pause => self.pause(),
            AudioCommand::Resume => self.resume(),
            AudioCommand::Stop => self.stop(),
            AudioCommand::ClearQueue => self.clear_queue(),
        }
    }

    fn update_state(&mut self, state: AudioState) {
        self.state = state;
        let _ = self.state_tx.send(self.state.clone());
    }

    fn pause(&mut self) {
        println!("pause");
    }

    fn resume(&mut self) {
        println!("resume");
    }

    fn stop(&mut self) {
        println!("stop");
    }

    fn clear_queue(&mut self) {
        self.stop();
        self.queue.clear();
    }

    fn queue_track(&mut self, track: Track) {
        self.queue.push_back(track.clone());
    }

    fn queue_album(&mut self, album: Album) {
        for track in album.tracks {
            self.queue_track(track);
        }
    }

    fn play_track(&mut self, track: Track) {
        self.clear_queue();
        self.queue_track(track.clone());

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Authorization", reqwest::header::HeaderValue::from_str(&self.config.auth()).unwrap());

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        let response = client.get(track.stream_url)
            .send()
            .unwrap();

        // Use Cursor to get Read and Seek traits.
        let cursor = std::io::Cursor::new(response.bytes().unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(cursor).unwrap();
        // Play the sound directly on the device
        self.stream_handle.play_raw(source.convert_samples()).unwrap();
    }

    fn play_album(&mut self, album: Album) {
        self.clear_queue();
        self.queue_album(album);
    }

    pub fn main_loop(&mut self) {
        self.update_state(self.state.clone());

        loop {
            if let Ok(command) = self.command_rx.try_recv() {
                self.process_command(command);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
