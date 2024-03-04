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
pub struct PlayerBus {
    sender: Sender<PlayerBusAction>, 
    receiver: Receiver<PlayerBusAction>,
}

impl PlayerBus {
    pub fn new() -> PlayerBus {
        let (sender, receiver): (Sender<PlayerBusAction>, Receiver<PlayerBusAction>) = unbounded();

        PlayerBus{
            sender,
            receiver,
        }
    }

    pub fn call(&self, action: PlayerBusAction) {
        debug!("[PlayerBus] Action called: {:?}", action);
        let _ = self.sender.send(action);
    }

    pub fn read(&self) -> PlayerBusAction {
        let actions: Vec<_> = self.receiver.try_iter().collect();

        match actions.last() {
            Some(action) => {
                info!("[PlayerBus] Action readed: {:?}", action);
                action.clone()
            },
            None => PlayerBusAction::None,
        }
    }
}