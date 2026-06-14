use std::{error::Error, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Mutex}, time::Duration};

use bytes::Bytes;

use crate::{config::Config, state::{self, PlayerBus}, playlist::{BufferedTrack, Playlist, Track}};
use crate::backend::cover::CoverProcessor;
use crate::playlist::BufferedCover;

use self::{downloader::Downloader, tidal::TidalBackend};
use log::info;

mod tidal;
mod downloader;
mod cover;

pub trait Backend {
    fn init(config: &mut Config, player_bus: PlayerBus) -> Self;
    fn discovery(&self, discovery_fn: impl Fn(Track));
    fn get_track_url(&mut self, track_id: String) -> Result<String, Box<dyn Error>>;
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
    config: Config,
    tidal: TidalBackend,
    downloader: Downloader,
    playerbus: Arc<Mutex<PlayerBus>>,
    listener_ready: Arc<AtomicBool>,
    discovery_generation: Arc<AtomicU64>,
}

impl BackendService {
    fn init(config: &Config, tidal: TidalBackend, playerbus: PlayerBus) -> Self {
        Self { 
            config: config.clone(),
            tidal: tidal.clone(),
            playerbus: Arc::new(Mutex::new(playerbus)),
            downloader: Downloader::init(config, tidal),
            listener_ready: Arc::new(AtomicBool::new(false)),
            discovery_generation: Arc::new(AtomicU64::new(0)),
        }
    }
    pub fn discover(&self) {
        let gen = self.discovery_generation.load(Ordering::SeqCst);
        info!("[Backend] discover() started, gen={}", gen);
        let count = std::sync::atomic::AtomicU64::new(0);
        let count_ref = &count;
        self.tidal.discovery(move |track| {
            let current = self.discovery_generation.load(Ordering::SeqCst);
            if current != gen {
                info!("[Backend] discover callback dropped: callback gen={}, current gen={}", gen, current);
                return;
            }
            count_ref.fetch_add(1, Ordering::SeqCst);
            self.playerbus.lock().unwrap().publish_message(state::Message::TrackDiscovered(track));
        });
        info!("[Backend] discover() finished, gen={}, published {} tracks", gen, count.load(Ordering::SeqCst));
    }
    pub fn is_listener_ready(&self) -> bool {
        self.listener_ready.load(Ordering::SeqCst)
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
                "Suspend".to_string(),
                "Resume".to_string(),
            ]
        );

        self.listener_ready.store(true, Ordering::SeqCst);

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

                    if !self.config.cmd_events.on_like.is_empty() {
                        match std::process::Command::new("sh").arg("-c").arg(&self.config.cmd_events.on_like).spawn() {
                            Ok(_) => {},
                            Err(e) => log::error!("[Backend] Failed to execute on-like command '{}': {}", self.config.cmd_events.on_like, e),
                        }
                    }
                },
                Some(state::Command::LoadCover(cover_url)) => {
                    let cover_path = CoverProcessor::new(self.tidal.get_cover(cover_url.clone()).unwrap()).generate_foreground().unwrap();
                    self.playerbus.lock().unwrap().publish_message(state::Message::CoverLoaded(BufferedCover { url: cover_url.clone(), path: cover_path.to_str().unwrap().to_string() }))
                },
                Some(state::Command::Suspend) => {
                    let new_gen = self.discovery_generation.fetch_add(1, Ordering::SeqCst) + 1;
                    info!("[Backend] Suspend: gen incremented to {}, clearing playlist", new_gen);
                    playlist.clear();
                    if !self.config.cmd_events.on_suspend.is_empty() {
                        match std::process::Command::new("sh").arg("-c").arg(&self.config.cmd_events.on_suspend).spawn() {
                            Ok(_) => {},
                            Err(e) => log::error!("[Backend] Failed to execute on-suspend command '{}': {}", self.config.cmd_events.on_suspend, e),
                        }
                    }
                    self.playerbus.lock().unwrap().publish_message(state::Message::Suspending);
                    info!("[Backend] Suspend: Suspending message published");
                },
                Some(state::Command::Resume) => {
                    let new_gen = self.discovery_generation.fetch_add(1, Ordering::SeqCst) + 1;
                    info!("[Backend] Resume: gen incremented to {}", new_gen);
                    if !self.config.cmd_events.on_resume.is_empty() {
                        match std::process::Command::new("sh").arg("-c").arg(&self.config.cmd_events.on_resume).spawn() {
                            Ok(_) => {},
                            Err(e) => log::error!("[Backend] Failed to execute on-resume command '{}': {}", self.config.cmd_events.on_resume, e),
                        }
                    }
                    let backend = self.clone();
                    info!("[Backend] Resume: spawning discover thread...");
                    std::thread::spawn(move || {
                        info!("[Backend] Resume: discover thread started");
                        backend.discover();
                        info!("[Backend] Resume: discover thread finished");
                    });
                    self.playerbus.lock().unwrap().publish_message(state::Message::Resuming);
                    info!("[Backend] Resume: Resuming message published");
                },
                _ => {
                    std::thread::sleep(Duration::from_millis(500));
                },
            }
        }
    }
}