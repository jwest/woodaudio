use std::{error::Error, sync::{Arc, Mutex}, time::Duration};

use bytes::Bytes;

use crate::{config::Config, state::{self, PlayerBus}, playlist::{BufferedTrack, Playlist, Track}};
use crate::backend::cover::CoverProcessor;
use crate::backend::storage::{CacheRandomRead, FileStorage};
use crate::playlist::{BufferedCover, PlayableItem};

use self::{downloader::Downloader, tidal::TidalBackend};

mod tidal;
mod downloader;
mod cover;
mod storage;

pub trait Backend {
    fn init(config: &mut Config, player_bus: PlayerBus) -> Self;
    fn discovery(&self, discovery_fn: impl Fn(Track));
    fn get_track(&mut self, track_id: String) -> Result<Bytes, Box<dyn Error>>;
    fn get_cover(&self, cover_url: String) -> Result<Bytes, Box<dyn Error>>;
    fn discovery_radio(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_track(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_album(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn discovery_artist(&self, id: &str, discovery_fn: impl Fn(Vec<Track>));
    fn add_track_to_favorites(&self, track_id: &str);
    fn get_favorite_albums(&self) -> Vec<PlayableItem>;
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
    discovery_local: bool,
    storage_local: Arc<Mutex<FileStorage>>,
}

impl BackendService {
    fn init(config: &Config, tidal: TidalBackend, playerbus: PlayerBus) -> Self {
        Self { 
            tidal: tidal.clone(),
            playerbus: Arc::new(Mutex::new(playerbus)),
            downloader: Downloader::init(config, tidal),
            discovery_local: config.player.without_cold_start,
            storage_local: Arc::new(Mutex::new(FileStorage::init(config.exporter_file.clone()))),
        }
    }
    pub fn discover(&self) {
        if self.discovery_local {
            for _ in 0..5 {
                let audio_file = self.storage_local.try_lock().unwrap().read_random_file(None).unwrap().unwrap();
                self.playerbus.lock().unwrap().publish_message(state::Message::TrackDiscoveredLocally(audio_file));
            }
        }

        self.tidal.discovery(move |track| {
            self.playerbus.lock().unwrap().publish_message(state::Message::TrackDiscovered(track));
        });
    }
    pub fn download(&mut self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        self.downloader.download_file(track)
    }
    pub fn listen_commands(self, playlist: Playlist) {
        let channel = self.playerbus.lock().unwrap().register_command_channel(
            vec![
                "AddTracksToPlaylist".to_string(), 
                "AddTracksToPlaylistForce".to_string(),
                "AddBufferedTracksToPlaylist".to_string(),
                "Radio".to_string(), 
                "PlayTrackForce".to_string(), 
                "PlayAlbumForce".to_string(), 
                "PlayArtistForce".to_string(), 
                "Like".to_string(),
                "LoadLikedAlbum".to_string(),
                "LoadCover".to_string(),
            ]
        );

        let discovery_fn = |tracks| {
            self.playerbus.lock().unwrap().publish_message(state::Message::TracksDiscoveredWithHighPriority(tracks));
        };

        loop {
            let command = channel.read_command();

            match command {
                Some(state::Command::AddTracksToPlaylist(tracks)) => {
                    playlist.push(tracks);
                },
                Some(state::Command::AddTracksToPlaylistForce(tracks)) => {
                    playlist.push_force(tracks);
                },
                Some(state::Command::AddBufferedTracksToPlaylist(tracks)) => {
                    playlist.push_buffered(tracks);
                },
                Some(state::Command::Radio(track_id)) => {
                    let _ = self.tidal.discovery_radio(&track_id, discovery_fn);
                    self.playerbus.lock().unwrap().publish_message(state::Message::RadioTracksLoaded);
                },
                Some(state::Command::PlayTrackForce(track_id)) => {
                    let _ = self.tidal.discovery_track(&track_id, discovery_fn);
                    self.playerbus.lock().unwrap().publish_message(state::Message::TrackLoaded);
                },
                Some(state::Command::PlayAlbumForce(track_id)) => {
                    let _ = self.tidal.discovery_album(&track_id, discovery_fn);
                    self.playerbus.lock().unwrap().publish_message(state::Message::AlbumTracksLoaded);
                },
                Some(state::Command::PlayArtistForce(track_id)) => {
                    let _ = self.tidal.discovery_artist(&track_id, discovery_fn);
                    self.playerbus.lock().unwrap().publish_message(state::Message::ArtistTracksLoaded);
                },
                Some(state::Command::Like(track_id)) => {
                    let _ = self.tidal.add_track_to_favorites(&track_id);
                    self.playerbus.lock().unwrap().publish_message(state::Message::TrackAddedToFavorites);
                },
                Some(state::Command::LoadLikedAlbum) => {
                    let playable_items = self.tidal.get_favorite_albums();
                    self.playerbus.lock().unwrap().publish_message(state::Message::BrowsingPlayableItemsReady(playable_items));
                },
                Some(state::Command::LoadCover(cover_url)) => {
                    let cover_path = CoverProcessor::new(self.tidal.get_cover(cover_url.clone()).unwrap()).generate_foreground().unwrap();
                    self.playerbus.lock().unwrap().publish_message(state::Message::CoverLoaded(BufferedCover { url: cover_url.clone(), path: cover_path.to_str().unwrap().to_string() }))
                },
                _ => {
                    std::thread::sleep(Duration::from_millis(500));
                },
            }
        }
    }
}