use core::fmt;
use std::{thread, time::Duration};
use bytes::Bytes;
use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};


#[derive(Clone)]
pub struct Track {
    pub id: String,
    pub full_name: String,
}

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Track: {}, {}", self.id, self.full_name)
    }
}

#[derive(Clone)]
pub struct BufferedTrack {
    pub track: Track,
    pub stream: Bytes,
}

impl fmt::Debug for BufferedTrack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BufferedTrack: {}, {}", self.track.id, self.track.full_name)
    }
}

trait BufforableTrack {
    fn load(&self) -> BufferedTrack;
}

impl BufforableTrack for Track {
    fn load(&self) -> BufferedTrack {
        BufferedTrack { track: self.to_owned(), stream: Bytes::new() }
    }
}

trait UnbufforableTrack {
    fn unload(&self) -> Track;
}

impl UnbufforableTrack for BufferedTrack {
    fn unload(&self) -> Track {
        Track { id: self.track.id.to_owned(), full_name: self.track.full_name.to_owned() }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Playlist {
    buffer_limit: usize,
    sender: Sender<Track>, 
    receiver: Receiver<Track>,
    buffered_sender: Sender<BufferedTrack>, 
    buffered_receiver: Receiver<BufferedTrack>,
}

impl Playlist {
    pub fn new() -> Playlist {
        let (sender, receiver): (Sender<Track>, Receiver<Track>) = unbounded();
        let (buffered_sender, buffered_receiver): (Sender<BufferedTrack>, Receiver<BufferedTrack>) = unbounded();

        Playlist{
            buffer_limit: 3,
            sender,
            receiver,
            buffered_sender,
            buffered_receiver,
        }
    }

    pub fn buffer_worker(&self, f: impl Fn(Track) -> BufferedTrack) {
        loop {
            if self.buffered_receiver.len() > self.buffer_limit {
                thread::sleep(Duration::from_secs(3));
                continue;
            }

            match self.receiver.recv() {
                Ok(track) => {
                    info!("[Playlist worker] Buffer track: {:?}", track);
                    let _ = self.buffered_sender.send(f(track));
                },
                Err(_) => thread::sleep(Duration::from_secs(3)),
            }
        }
    }

    pub fn push(&self, track: Track) {
        debug!("[Playlist] Push track: {:?}", track);
        let _ = self.sender.send(track);
    }

    pub fn pop(&self) -> Option<BufferedTrack> {
        match self.buffered_receiver.try_recv() {
            Ok(track) => {
                info!("[Playlist] Pop track: {:?}", track);
                Some(track)
            },
            Err(_) => None,
        }
    }
    
    pub fn push_force(&self, tracks: Vec<Track>) {
        debug!("[Playlist] Force push tracks: {:?}", tracks);
        
        let mut tracks: Vec<Track> = tracks;

        loop {
            match self.receiver.try_recv() {
                Ok(track) => tracks.push(track),
                Err(_) => break,
            }
        }

        loop {
            match self.buffered_receiver.try_recv() {
                Ok(buffered_track) => tracks.push(buffered_track.unload()),
                Err(_) => break,
            }
        }

        tracks.iter()
            .for_each(|t| { let _ = self.sender.send(t.clone()); });
    }
}
