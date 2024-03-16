use std::{sync::{Arc, Mutex}, time::Duration};

use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

use crate::playlist::{BufferedTrack, Cover, Track};


#[derive(Debug)]
#[derive(Clone)]
pub enum Command {
    Play,
    Pause,
    Next,
    Like(String),
    Radio(String),
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Message {
    PlayerPlayingNewTrack(BufferedTrack),
    PlayerPlaying,
    PlayerToPause,
    PlayerElapsed(Duration),
    PlayerQueueIsEmpty,

    UserPlay,
    UserPause,
    UserPlayNext,
    UserLike(String),
    UserLoadRadio(String),
}

#[derive(Debug)]
#[derive(Clone)]
pub struct State {
    pub player: PlayerState,
    pub track: Option<TrackState>,
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
    pub foreground: String,
    pub background: String,
}

impl From<Cover> for TrackStateCover {
    fn from(cover: Cover) -> Self {
        Self { foreground: cover.foreground, background: cover.background }
    }
}

impl From<Option<Cover>> for TrackStateCover {
    fn from(cover: Option<Cover>) -> Self {
        Self { 
            foreground: cover.clone().map(|cover| cover.foreground).unwrap_or("../static/sample_cover.jpg-foreground.png".to_string()),
            background: cover.clone().map(|cover| cover.background).unwrap_or("../static/sample_cover.jpg-background.png".to_string()),
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
    pub cover: Option<TrackStateCover>,
    pub duration: Duration,
}

impl From<BufferedTrack> for TrackState {
    fn from(buffered_track: BufferedTrack) -> Self {
        TrackState {
            id: buffered_track.track.id,
            title: buffered_track.track.title,
            artist_name: buffered_track.track.artist_name,
            album_name: buffered_track.track.album_name,
            cover: buffered_track.cover.map(|c| Some(TrackStateCover::from(c))).unwrap_or_default(),
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
            cover: None,
            duration: track.duration,
        }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerBus {
    message_sender: Sender<Command>, 
    message_receiver: Receiver<Command>,
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
        }
    }
}

impl PlayerBus {
    pub fn new() -> PlayerBus {
        let (message_sender, message_receiver): (Sender<Command>, Receiver<Command>) = unbounded();

        PlayerBus{
            message_sender,
            message_receiver,
            state: Arc::new(Mutex::new(State::default_state())),
        }
    }

    pub fn publish_message(&self, message: Message) {
        let mut state = self.state.lock().unwrap();

        let prev_state = state.clone();
        let next_state = match message {
            Message::PlayerPlayingNewTrack(track) => State { track: Some(TrackState::from(track)), player: PlayerState { case: PlayerStateCase::Playing, playing_time: Some(Duration::ZERO) } },
            Message::PlayerPlaying => State { track: prev_state.track, player: PlayerState { case: PlayerStateCase::Playing, playing_time: prev_state.player.playing_time } },
            Message::PlayerToPause => State { track: prev_state.track, player: PlayerState { case: PlayerStateCase::Paused, playing_time: prev_state.player.playing_time } },
            Message::PlayerElapsed(duration) => State { track: prev_state.track, player: PlayerState { case: prev_state.player.case, playing_time: Some(duration) } },
            Message::PlayerQueueIsEmpty => State { track: None, player: PlayerState { case: PlayerStateCase::Loading, playing_time: None } },
            Message::UserPlay => { self.publish_command(Command::Play); prev_state },
            Message::UserPause => { self.publish_command(Command::Pause); prev_state },
            Message::UserPlayNext => { self.publish_command(Command::Next); prev_state },
            Message::UserLike(track) => { self.publish_command(Command::Like(track)); prev_state },
            Message::UserLoadRadio(track) => { self.publish_command(Command::Radio(track)); prev_state },
        };

        *state = next_state;
    }

    pub fn publish_command(&self, command: Command) {
        let _ = self.message_sender.send(command);
    }

    pub fn read_command(&self) -> Option<Command> {
        let command = self.message_receiver.try_recv();
        debug!("[PlayerBus] Command readed: {:?}", command);

        match command {
            Ok(command) => Some(command),
            Err(_) => None,
        }
    }

    pub fn read_state(&self) -> State {
        self.state.lock().unwrap().clone()
    }
}