use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

#[derive(Debug)]
#[derive(Clone)]
pub enum PlayerBusAction {
    PausePlay,
    NextSong,
    None,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum PlayerState {
    Playing,
    Paused,
    Loading,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerTrackState {
    pub player_state: PlayerState,
    pub id: String,
    pub title: String,
    pub artist_name: String,
    pub album_name: String,
    pub cover: Option<String>,
    pub cover_background: Option<String>,
    pub duration: Duration,
    pub playing_time: Instant,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayerBus {
    actions_sender: Sender<PlayerBusAction>, 
    actions_receiver: Receiver<PlayerBusAction>,
    state_sender: Sender<PlayerTrackState>, 
    state_receiver: Receiver<PlayerTrackState>,
}

impl PlayerTrackState {
    pub fn default_state() -> PlayerTrackState {
        PlayerTrackState { 
            player_state: PlayerState::Loading, 
            id: "".to_string(), 
            title: "".to_string(), 
            artist_name: "".to_string(), 
            album_name: "".to_string(), 
            cover: None,
            cover_background: None,
            duration: Duration::ZERO, 
            playing_time: Instant::now(),
        }
    }
}

impl PlayerBus {
    pub fn new() -> PlayerBus {
        let (actions_sender, actions_receiver): (Sender<PlayerBusAction>, Receiver<PlayerBusAction>) = unbounded();
        let (state_sender, state_receiver): (Sender<PlayerTrackState>, Receiver<PlayerTrackState>) = unbounded();

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

    pub fn set_state(&self, state: PlayerTrackState) {
        debug!("[PlayerBus] Set state: {:?}", state);
        let _ = self.state_sender.send(state);
    }

    pub fn read_state(&self) -> Option<PlayerTrackState> {
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