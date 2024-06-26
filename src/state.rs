use std::{sync::{Arc, Mutex}, time::Duration};
use std::collections::HashMap;

use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

use crate::playlist::{BufferedCover, BufferedTrack, Cover, PlayableItem, Track};

#[derive(Debug)]
#[derive(Clone)]
pub enum Command {
    Play,
    Pause,
    Next,
    Like(String),
    Radio(String),
    PlayTrackForce(String),
    PlayAlbumForce(String),
    PlayArtistForce(String),
    ShowScreen(String),
    AddTracksToPlaylist(Vec<Track>),
    AddTracksToPlaylistForce(Vec<Track>),
    AddBufferedTracksToPlaylist(Vec<BufferedTrack>),
    LoadLikedAlbum,
    LoadCover(String)
}

impl Command {
    pub fn as_string(&self) -> String {
        match self {
            Command::Play => "Play".to_owned(),
            Command::Pause => "Pause".to_owned(),
            Command::Next => "Next".to_owned(),
            Command::Like(_) => "Like".to_owned(),
            Command::Radio(_) => "Radio".to_owned(),
            Command::PlayTrackForce(_) => "PlayTrackForce".to_owned(),
            Command::PlayAlbumForce(_) => "PlayAlbumForce".to_owned(),
            Command::PlayArtistForce(_) => "PlayArtistForce".to_owned(),
            Command::ShowScreen(_) => "ShowScreen".to_owned(),
            Command::AddTracksToPlaylist(_) => "AddTracksToPlaylist".to_owned(),
            Command::AddTracksToPlaylistForce(_) => "AddTracksToPlaylistForce".to_owned(),
            Command::AddBufferedTracksToPlaylist(_) => "AddBufferedTracksToPlaylist".to_owned(),
            Command::LoadLikedAlbum => "LoadLikedAlbum".to_owned(),
            Command::LoadCover(_) => "LoadCover".to_owned(),
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Message {
    PlayerPlayingNewTrack(BufferedTrack),
    PlayerPlaying,
    PlayerToPause,
    PlayerElapsed(Duration),
    PlayerQueueIsEmpty,

    TrackAddedToFavorites,
    TrackDiscovered(Track),
    TracksDiscoveredWithHighPriority(Vec<Track>),
    TrackDiscoveredLocally(BufferedTrack),

    UserPlay,
    UserPause,
    UserPlayNext,
    UserLike,
    UserLoadRadio,
    UserPlayTrack(String),
    UserPlayAlbum(String),
    UserPlayArtist(String),

    UserClickActions,
    UserClickLikedAlbumsButton,
    UserClickBackToPlayer,

    TidalBackendStarted,
    TidalBackendLoginLinkCreated(String),
    TidalBackendInitialized,

    RadioTracksLoaded,
    TrackLoaded,
    AlbumTracksLoaded,
    ArtistTracksLoaded,
    BrowsingPlayableItemsReady(Vec<PlayableItem>),
    CoverLoaded(BufferedCover),
    CoverNeeded(String),
}

#[derive(Debug)]
#[derive(Clone)]
pub struct State {
    pub player: PlayerState,
    pub track: Option<TrackState>,
    pub backends: BackendsState,
    pub browser: Option<BrowserState>,
    pub covers: Covers,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Covers {
    items: HashMap<String, String>
}

impl Covers {
    fn init() -> Self {
        Self { items: HashMap::new() }
    }

    pub fn get(&self, cover_url: String) -> Option<&String> {
        self.items.get(&cover_url)
    }

    fn add_and_build(&self, cover: BufferedCover) -> Self {
        let mut new_covers = HashMap::from(self.items.to_owned());
        new_covers.insert(cover.id(), cover.path);
        Covers { items: new_covers }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum BackendState {
    Off,
    Initialization,
    WaitingForLoginByLink(String),
    Ready,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct BackendsState {
    pub tidal: BackendState,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct BrowserState {
    pub items: Vec<PlayableItem>,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerState {
    pub case: PlayerStateCase,
    pub playing_time: Option<Duration>,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum PlayerStateCase {
    Playing,
    Paused,
    Loading,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TrackStateCover {
    pub foreground: Option<String>,
    pub background: Option<String>,
}

impl From<Cover> for TrackStateCover {
    fn from(cover: Cover) -> Self {
        Self { 
            foreground: cover.foreground,
            background: cover.background,
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TrackState {
    pub id: String,
    pub title: String,
    pub artist_name: String,
    pub album_name: String,
    pub cover: TrackStateCover,
    pub duration: Duration,
}

impl From<BufferedTrack> for TrackState {
    fn from(buffered_track: BufferedTrack) -> Self {
        TrackState {
            id: buffered_track.track.id,
            title: buffered_track.track.title,
            artist_name: buffered_track.track.artist_name,
            album_name: buffered_track.track.album_name,
            cover: TrackStateCover::from(buffered_track.cover),
            duration: buffered_track.track.duration,
        }
    }
}

impl From<Track> for TrackState {
    fn from(track: Track) -> Self {
        TrackState {
            id: track.id,
            title: track.title,
            artist_name: track.artist_name,
            album_name: track.album_name,
            cover: TrackStateCover::from(Cover::empty()),
            duration: track.duration,
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct BroadcastChannel {
    message_sender: Sender<Command>, 
    message_receiver: Receiver<Command>,
    commands: Vec<String>
}

impl BroadcastChannel {
    pub fn read_command(&self) -> Option<Command> {
        let command = self.message_receiver.try_recv();
        debug!("[PlayerBus] Command readed: {:?}", command);

        match command {
            Ok(command) => Some(command),
            Err(_) => None,
        }
    }
    fn send(&self, command: Command) {
        let _ = self.message_sender.send(command);
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Broadcast {
    channels: Arc<Mutex<Vec<BroadcastChannel>>>,
}

impl Broadcast {
    fn init() -> Broadcast {
        Self {
            channels: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn register(&mut self, commands: Vec<String>) -> BroadcastChannel {
        let (message_sender, message_receiver): (Sender<Command>, Receiver<Command>) = unbounded();
        let channel = BroadcastChannel {
            message_receiver,
            message_sender,
            commands: commands.clone(),
        };
        let new_channels = self.channels.clone();
        new_channels.lock().unwrap().push(channel.clone());

        info!("[PlayerBus] new channel on broadcast registred, commands: {:?}", commands);
        channel
    }
    fn send(&self, command: Command) {
        for channel in self.channels.lock().unwrap().iter() {
            if channel.commands.contains(&command.as_string()) {
                channel.send(command.clone());
                info!("[PlayerBus] broadcast event sended, command: {:?}, channel: {:?}", command, channel);
            }
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerBus {
    broadcast: Broadcast,
    state: Arc<Mutex<State>>,
}

impl State {
    pub fn default_state() -> State {
        State {
            player: PlayerState {
                case: PlayerStateCase::Loading,
                playing_time: None,
            },
            track: None,
            backends: BackendsState { 
                tidal: BackendState::Off
            },
            browser: None,
            covers: Covers::init(),
        }
    }
}

impl PlayerBus {
    pub fn new() -> PlayerBus {
        PlayerBus{
            broadcast: Broadcast::init(),
            state: Arc::new(Mutex::new(State::default_state())),
        }
    }

    pub fn publish_message(&self, message: Message) {
        let mut state = self.state.lock().unwrap();

        let prev_state = state.clone();
        let next_state = match message {
            Message::PlayerPlayingNewTrack(track) => State { track: Some(TrackState::from(track)), player: PlayerState { case: PlayerStateCase::Playing, playing_time: Some(Duration::ZERO) }, ..prev_state },
            Message::PlayerPlaying => State { player: PlayerState { case: PlayerStateCase::Playing, ..prev_state.player }, ..prev_state },
            Message::PlayerToPause => State { player: PlayerState { case: PlayerStateCase::Paused, ..prev_state.player }, ..prev_state },
            Message::PlayerElapsed(duration) => State { player: PlayerState { case: prev_state.player.case, playing_time: Some(duration) }, ..prev_state },
            Message::PlayerQueueIsEmpty => State { track: None, player: PlayerState { case: PlayerStateCase::Loading, playing_time: None }, ..prev_state },
            Message::UserPlay => { self.publish_command(Command::Play); prev_state },
            Message::UserPause => { self.publish_command(Command::Pause); prev_state },
            Message::UserPlayNext => { self.publish_command(Command::Next); prev_state },
            Message::UserLike => { self.publish_command(Command::Like(prev_state.track.clone().unwrap().id)); prev_state },
            Message::UserLoadRadio => { self.publish_command(Command::Pause); self.publish_command(Command::Radio(prev_state.track.clone().unwrap().id)); prev_state },
            Message::UserPlayTrack(track) => { self.publish_command(Command::Pause); self.publish_command(Command::PlayTrackForce(track)); prev_state },
            Message::UserPlayAlbum(track) => { self.publish_command(Command::Pause); self.publish_command(Command::PlayAlbumForce(track)); prev_state },
            Message::UserPlayArtist(track) => { self.publish_command(Command::Pause); self.publish_command(Command::PlayArtistForce(track)); prev_state },
            Message::TrackAddedToFavorites => { prev_state },
            Message::TrackDiscovered(track) => { self.publish_command(Command::AddTracksToPlaylist(vec![track])); prev_state },
            Message::TracksDiscoveredWithHighPriority(tracks) => { self.publish_command(Command::AddTracksToPlaylistForce(tracks)); prev_state },
            Message::TrackDiscoveredLocally(buffered_stream) => { self.publish_command(Command::AddBufferedTracksToPlaylist(vec![buffered_stream])); prev_state },
            Message::UserClickActions => { self.publish_command(Command::ShowScreen("/actions".to_string())); prev_state },
            Message::UserClickBackToPlayer => { self.publish_command(Command::ShowScreen("/player".to_string())); prev_state },
            Message::UserClickLikedAlbumsButton => { self.publish_command(Command::ShowScreen("/browse".to_string())); self.publish_command(Command::LoadLikedAlbum); prev_state },
            Message::TidalBackendStarted => State { backends: BackendsState { tidal: BackendState::Initialization, ..prev_state.backends }, ..prev_state },
            Message::TidalBackendLoginLinkCreated(login_link) =>  State { backends: BackendsState { tidal: BackendState::WaitingForLoginByLink(login_link), ..prev_state.backends }, ..prev_state },
            Message::TidalBackendInitialized => { self.publish_command(Command::ShowScreen("/player".to_string())); State { backends: BackendsState { tidal: BackendState::Ready, ..prev_state.backends }, ..prev_state }},
            Message::RadioTracksLoaded => { self.publish_command(Command::Next); prev_state },
            Message::TrackLoaded => { self.publish_command(Command::Next); prev_state },
            Message::AlbumTracksLoaded => { self.publish_command(Command::Next); prev_state },
            Message::ArtistTracksLoaded => { self.publish_command(Command::Next); prev_state },
            Message::BrowsingPlayableItemsReady(items) => State { browser: Some(BrowserState { items }), ..prev_state },
            Message::CoverNeeded(cover_url) => { self.publish_command(Command::LoadCover(cover_url)); prev_state },
            Message::CoverLoaded(cover) => State { covers: prev_state.covers.add_and_build(cover), ..prev_state },
        };

        *state = next_state;
    }

    pub fn register_command_channel(&mut self, commands: Vec<String>) -> BroadcastChannel {
        self.broadcast.register(commands)
    }

    pub fn publish_command(&self, command: Command) {
        self.broadcast.send(command);
    }

    pub fn read_state(&self) -> State {
        self.state.lock().unwrap().clone()
    }
}