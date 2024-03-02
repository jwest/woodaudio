use std::time::Duration;
use std::{thread, time};
use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};

#[derive(Debug)]
pub struct Track {
    pub id: String,
    pub full_name: String,
    pub file_path: Option<String>,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct EventBus {
    track_discovered_sender: Sender<Track>, 
    track_discovered_receiver: Receiver<Track>,
    track_downloaded_sender: Sender<Track>, 
    track_downloaded_receiver: Receiver<Track>,
}

impl EventBus {
    pub fn new() -> EventBus {
        let (track_discovered_sender, track_discovered_receiver): (Sender<Track>, Receiver<Track>) = unbounded();
        let (track_downloaded_sender, track_downloaded_receiver): (Sender<Track>, Receiver<Track>) = unbounded();

        EventBus{
            track_discovered_sender,
            track_discovered_receiver,
            track_downloaded_sender,
            track_downloaded_receiver,
        }
    }

    pub fn track_discovered(&self, track: Track) {
        debug!("[Discoverer] Track discovered: {:?}", track);
        let _ = self.track_discovered_sender.send(track);
    }

    pub fn on_track_discovered(&self, f: impl Fn(&Track)) {
        loop {
            if self.track_downloaded_sender.len() > 3 {
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
        let _ = self.track_downloaded_sender.send(track);
    }

    pub fn on_track_downloaded(&self, f: impl Fn(&Track)) {
        loop {
            match self.track_downloaded_receiver.recv() {
                Ok(track) => {
                    info!("[Player] Track played: {:?}", track);
                    f(&track)
                },
                Err(_) => thread::sleep(Duration::from_secs(3)),
            }
        }
    }
}
