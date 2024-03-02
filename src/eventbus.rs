use std::time::Duration;
use std::{thread, time};
use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

#[derive(Clone)]
#[derive(Debug)]
pub struct Track {
    pub id: String,
    pub full_name: String,
    pub file_path: Option<String>,
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

#[derive(Debug)]
#[derive(Clone)]
pub struct Playlist {
    sender: Sender<Track>, 
    receiver: Receiver<Track>,
}

impl Playlist {
    pub fn new() -> Playlist {
        let (sender, receiver): (Sender<Track>, Receiver<Track>) = unbounded();

        Playlist{
            sender,
            receiver,
        }
    }

    pub fn push(&self, track: Track) {
        debug!("[Playlist] Track added: {:?}", track);
        let _ = self.sender.send(track);
    }

    pub fn push_force(&self, tracks: Vec<Track>) {
        info!("[Playlist] Priority track added: {:?}", tracks);
        
        loop {
            match self.receiver.try_recv() {
                Ok(_) => {},
                Err(_) => break,
            }
        }

        tracks.iter()
            .for_each(|t| { let _ = self.sender.send(t.clone()); });
    }

    pub fn pop(&self) -> Option<Track> {
        match self.receiver.try_recv() {
            Ok(track) => {
                info!("[Playlist] Track play: {:?}", track);
                Some(track)
            },
            Err(_) => None,
        }
    }

    pub fn size(&self) -> usize {
        self.receiver.len()
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct EventBus {
    track_discovered_sender: Sender<Track>, 
    track_discovered_receiver: Receiver<Track>,
    playlist: Playlist,
}

impl EventBus {
    pub fn new(playlist: Playlist) -> EventBus {
        let (track_discovered_sender, track_discovered_receiver): (Sender<Track>, Receiver<Track>) = unbounded();

        EventBus{
            track_discovered_sender,
            track_discovered_receiver,
            playlist,
        }
    }

    pub fn track_discovered(&self, track: Track) {
        debug!("[Discoverer] Track discovered: {:?}", track);
        let _ = self.track_discovered_sender.send(track);
    }

    pub fn on_track_discovered(&self, f: impl Fn(&Track)) {
        loop {
            if self.playlist.size() > 3 {
                thread::sleep(time::Duration::from_secs(3));
                continue;
            }

            match self.track_discovered_receiver.recv() {
                Ok(track) => {
                    info!("[Downloader] Track discovered: {:?}", track);
                    f(&track)
                },
                Err(_) => thread::sleep(Duration::from_secs(3)),
            }
        }
    }
    
    pub fn track_downloaded(&self, track: Track) {
        info!("[Downloader] Track downloaded: {:?}", track);
        let _ = self.playlist.push(track);
    }
}
