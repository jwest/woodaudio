use std::{error::Error, sync::{Arc, Mutex}, time::Duration};

use bytes::Bytes;

use crate::{config::Config, playerbus::{self, PlayerBus}, playlist::{BufferedTrack, Playlist, Track}};

use self::{downloader::Downloader, tidal::TidalBackend};

mod tidal;
mod downloader;
mod cover;
mod storage;

pub trait Backend {
    fn init(config: &mut Config, player_bus: PlayerBus) -> Self;
    fn discovery(&self, discovery_fn: impl Fn(Track));
    fn get_track(&self, track_id: String) -> Result<Bytes, Box<dyn Error>>;
    fn get_cover(&self, cover_url: String) -> Result<Bytes, Box<dyn Error>>;
    fn discovery_radio(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_track(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_album(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_artist(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn add_track_to_favorites(&self, track_id: &str);
}

#[derive(Clone)]
pub struct BackendInitialization {
    backend: Arc<Mutex<Option<BackendService>>>,
    config: Config,
    playerbus: PlayerBus,
}

impl BackendInitialization {
    pub fn new(config: Config, playerbus: PlayerBus) -> Self {
        Self { 
            backend: Arc::new(Mutex::new(None)),
            config,
            playerbus,
        }
    }
    pub fn initialization(&self) {
        let tidal = TidalBackend::init(&mut self.config.clone(), self.playerbus.clone());
        let mut backend = self.backend.lock().unwrap();

        *backend = Some(BackendService::init(&self.config, tidal, self.playerbus.clone()));
    }
    pub fn get_initialized(&self) -> BackendService {
        loop {
            if self.is_initialized() {
                break;
            }
        }
        self.backend.lock().unwrap().clone().unwrap()
    }
    fn is_initialized(&self) -> bool {
        self.backend.lock().unwrap().is_some()
    }
}

#[derive(Clone)]
pub struct BackendService {
    tidal: TidalBackend,
    downloader: Downloader,
    playerbus: Arc<Mutex<PlayerBus>>,
}

impl BackendService {
    fn init(config: &Config, tidal: TidalBackend, playerbus: PlayerBus) -> Self {
        Self { 
            tidal: tidal.clone(),
            playerbus: Arc::new(Mutex::new(playerbus)),
            downloader: Downloader::init(config, tidal),
        }
    }
    pub fn discover(&self) {
        self.tidal.discovery(move |track| {
            self.playerbus.lock().unwrap().publish_message(playerbus::Message::TrackDiscovered(track));
        });
    }
    pub fn download(&self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        self.downloader.download_file(track)
    }
    pub fn listen_commands(self, playlist: Playlist) {
        let channel = self.playerbus.lock().unwrap().register_command_channel(
            vec![
                "AddTracksToPlaylist".to_string(), 
                "AddTracksToPlaylistForce".to_string(), 
                "Radio".to_string(), 
                "PlayTrackForce".to_string(), 
                "PlayAlbumForce".to_string(), 
                "PlayArtistForce".to_string(), 
                "Like".to_string(),
            ]
        );

        let discovery_fn = |tracks| {
            self.playerbus.lock().unwrap().publish_message(playerbus::Message::TracksDiscoveredWithHighPriority(tracks));
        };

        loop {
            let command = channel.read_command();

            match command {
                Some(playerbus::Command::AddTracksToPlaylist(tracks)) => {
                    playlist.push(tracks);
                },
                Some(playerbus::Command::AddTracksToPlaylistForce(tracks)) => {
                    playlist.push_force(tracks);
                },
                Some(playerbus::Command::Radio(track_id)) => {
                    let _ = self.tidal.discovery_radio(&track_id, discovery_fn);
                },
                Some(playerbus::Command::PlayTrackForce(track_id)) => {
                    let _ = self.tidal.discovery_track(&track_id, discovery_fn);
                },
                Some(playerbus::Command::PlayAlbumForce(track_id)) => {
                    let _ = self.tidal.discovery_album(&track_id, discovery_fn);
                },
                Some(playerbus::Command::PlayArtistForce(track_id)) => {
                    let _ = self.tidal.discovery_artist(&track_id, discovery_fn);
                },
                Some(playerbus::Command::Like(track_id)) => {
                    let _ = self.tidal.add_track_to_favorites(&track_id);
                    self.playerbus.lock().unwrap().publish_message(playerbus::Message::TrackAddedToFavorites);
                },
                _ => {},
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }
}