use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

use crate::playlist::{BufferedTrack, Cover, Track};

#[derive(Debug)]
#[derive(Clone)]
pub struct State {
    pub player: PlayerState,
    pub track: Option<TrackState>,
}

impl State {
    pub fn build_with_change_track(&self, track_state: TrackState) -> Self {
        Self { player: self.player.clone(), track: Some(track_state) }
    }
    pub fn build_with_change_player(&self, player_state: PlayerState) -> Self {
        Self { player: player_state, track: self.track.clone() }
    }
    pub fn publish(&self, player_bus: &PlayerBus) -> Self {
        player_bus.set_state(self.clone());
        self.clone()
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerState {
    pub case: PlayerStateCase,
    pub playing_time: Option<Instant>,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum PlayerBusAction {
    PausePlay,
    NextSong,
    None,
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
    actions_sender: Sender<PlayerBusAction>, 
    actions_receiver: Receiver<PlayerBusAction>,
    state_sender: Sender<State>, 
    state_receiver: Receiver<State>,
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
        let (actions_sender, actions_receiver): (Sender<PlayerBusAction>, Receiver<PlayerBusAction>) = unbounded();
        let (state_sender, state_receiver): (Sender<State>, Receiver<State>) = unbounded();

        PlayerBus{
            actions_sender,
            actions_receiver,
            state_sender,
            state_receiver,
        }
    }

    pub fn call(&self, action: PlayerBusAction) {
        debug!("[PlayerBus] Action called: {:?}", action);
        let _ = self.actions_sender.send(action);
    }

    pub fn read(&self) -> PlayerBusAction {
        let actions: Vec<_> = self.actions_receiver.try_iter().collect();

        match actions.last() {
            Some(action) => {
                info!("[PlayerBus] Action readed: {:?}", action);
                action.clone()
            },
            None => PlayerBusAction::None,
        }
    }

    pub fn set_state(&self, state: State) {
        debug!("[PlayerBus] Set state: {:?}", state);
        let _ = self.state_sender.send(state);
    }

    pub fn read_state(&self) -> Option<State> {
        let states: Vec<_> = self.state_receiver.try_iter().collect();
        debug!("[PlayerBus] Read states: {:?}", states);

        match states.last() {
            Some(state) => {
                Some(state.clone())
            },
            None => None,
        }
    }
}