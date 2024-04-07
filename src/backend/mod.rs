use std::{error::Error, sync::{Arc, Mutex}, time::Duration};

use crate::{config::Config, playerbus::{self, PlayerBus}, playlist::{BufferedTrack, Playlist, Track}};

use self::{discovery::DiscoveryStore, downloader::Downloader, session::Session};

pub mod discovery;
pub mod session;
pub mod downloader;

#[derive(Clone)]
pub struct TidalBackend {
    discovery: DiscoveryStore,
    session: Session,
    downloader: Downloader,
}

impl TidalBackend {
    pub fn init(config: &mut Config, player_bus: PlayerBus) -> Self {
        let session = Session::setup(config, player_bus.clone());
        Self { 
            discovery: DiscoveryStore::new(),
            session: session.clone(),
            downloader: Downloader::init(&session, config),
        }
    }
    fn discovery(&self, discovery_fn: impl Fn(Track)) {
        let _ = self.discovery.discover_mixes(&self.session, &discovery_fn);
        let _ = self.discovery.discover_favorities_tracks(&self.session, &discovery_fn);
    }
    fn download(&self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        self.downloader.download_file(track)
    }
    fn discovery_radio(&self, id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        let _ = self.discovery.discovery_radio(&self.session, id, &discovery_fn);
    }
    fn discovery_track(&self, id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        let _ = self.discovery.discovery_track(&self.session, id, &discovery_fn);
    }
    fn discovery_album(&self, id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        let _ = self.discovery.discovery_album(&self.session, id, &discovery_fn);
    }
    fn discovery_artist(&self, id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        let _ = self.discovery.discovery_artist(&self.session, id, &discovery_fn);
    }
}

#[derive(Clone)]
pub struct Backend {
    tidal: TidalBackend,
    playerbus: Arc<Mutex<PlayerBus>>,
}

impl Backend {
    pub fn init(tidal: TidalBackend, playerbus: PlayerBus) -> Self {
        Self { tidal, playerbus: Arc::new(Mutex::new(playerbus)) }
    }
    pub fn discover(&self) {
        self.tidal.discovery(move |track| {
            self.playerbus.lock().unwrap().publish_message(playerbus::Message::TrackDiscovered(track));
        });
    }
    pub fn download(&self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        self.tidal.download(track)
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
        let session = self.playerbus.lock().unwrap().wait_for_session();

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
                    let _ = session.add_track_to_favorites(&track_id);
                    self.playerbus.lock().unwrap().publish_message(playerbus::Message::TrackAddedToFavorites);
                },
                _ => {},
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }
}