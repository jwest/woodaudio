use core::fmt;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use bytes::Bytes;
use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, info};
use serde_json::Value;

#[derive(Clone)]
pub struct Track {
    pub id: String,
    pub title: String,
    pub artist_name: String,
    pub album_name: String,
    pub album_image: String, // Valid resolutions: 80x80, 160x160, 320x320, 640x640, 1280x1280
    pub duration: Duration,
}

impl Track {
    pub fn build_from_json(item: Value) -> Track {
        let cover = if item["album"]["cover"].is_string() { 
            item["album"]["cover"].as_str().unwrap()
        } else { 
            "0dfd3368-3aa1-49a3-935f-10ffb39803c0" 
        }.replace("-", "/");

        let artist_name = item["artists"].as_array()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|item| item["name"].as_str().unwrap())
            .collect::<Vec<&str>>()
            .join(", ");

        Track {
            id: item["id"].as_i64().unwrap().to_string(),
            title: item["title"].as_str().unwrap_or_default().to_string(),
            artist_name,
            album_name: item["album"]["title"].as_str().unwrap_or_default().to_string(),
            album_image: format!("https://resources.tidal.com/images/{}/{}x{}.jpg", cover, 320, 320),
            duration: Duration::from_secs(item["duration"].as_u64().unwrap_or_default()),
        }
    }

    pub fn duration_formated(&self) -> String {
        let seconds = self.duration.as_secs() % 60;
        let minutes = (self.duration.as_secs() / 60) % 60;
        format!("{}:{:0>2}", minutes, seconds)
    }

    pub fn full_name(&self) -> String {
        format!("{} - {}: {} ({})", self.artist_name, self.album_name, self.title, self.duration_formated())
    }
}

impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Track: {}, {}", self.id, self.full_name())
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Cover {
    pub foreground: Option<String>, 
    pub background: Option<String>,
}

impl Cover {
    pub fn empty() -> Self {
        Self { foreground: None, background: None }
    }
}

#[derive(Clone)]
pub struct BufferedTrack {
    pub track: Track,
    pub stream: Bytes,
    pub cover: Cover,
}

impl fmt::Debug for BufferedTrack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BufferedTrack: {}, {}, stream: {}, cover: {:?}", self.track.id, self.track.full_name(), !self.stream.is_empty(), self.cover)
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Playlist {
    buffer_limit: usize,
    sender: Arc<Mutex<Sender<Track>>>,
    receiver: Receiver<Track>,
    buffered_sender: Sender<BufferedTrack>, 
    buffered_receiver: Receiver<BufferedTrack>,
    force_lock: Arc<Mutex<bool>>,
}

impl Playlist {
    pub fn new() -> Playlist {
        let (sender, receiver): (Sender<Track>, Receiver<Track>) = unbounded();
        let (buffered_sender, buffered_receiver): (Sender<BufferedTrack>, Receiver<BufferedTrack>) = unbounded();

        Playlist{
            buffer_limit: 3,
            sender: Arc::new(Mutex::new(sender)),
            receiver,
            buffered_sender,
            buffered_receiver,
            force_lock: Arc::new(Mutex::new(false)),
        }
    }

    pub fn buffer_worker(&self, f: impl Fn(Track) -> Option<BufferedTrack>) {
        loop {
            if self.buffered_receiver.len() > self.buffer_limit {
                thread::sleep(Duration::from_secs(3));
                continue;
            }

            match self.receiver.recv() {
                Ok(track) => {
                    let old_force_lock = self.force_lock.lock().unwrap().clone();
                    
                    info!("[Playlist worker] Buffer track: {:?}", track);
                    match f(track.clone()) {
                        Some(buffered_track) => {
                            let mut force_lock = self.force_lock.lock().unwrap();
                            if old_force_lock == false && force_lock.eq(&true) {
                                info!("[Playlist worker] force lock ignore trakc {:?}", track);
                                *force_lock = false;
                            } else {
                                let _ = self.buffered_sender.send(buffered_track);
                            }
                        },
                        None => todo!(),
                    }
                },
                Err(_) => thread::sleep(Duration::from_secs(3)),
            }
        }
    }

    pub fn push(&self, tracks: Vec<Track>) {
        debug!("[Playlist] Push tracks: {:?}", tracks);
        let _ = self.sender.lock().map(|sender| {
            tracks.iter()
                .for_each(|t| { let _ = sender.send(t.clone()); });
        });
    }

    pub fn pop(&self) -> Option<BufferedTrack> {
        self.sender.lock().map(|_| {
            match self.buffered_receiver.try_recv() {
                Ok(track) => {
                    info!("[Playlist] Pop track: {:?}", track);
                    Some(track)
                },
                Err(_) => None,
            }
        }).unwrap()
    }
    
    pub fn push_force(&self, tracks: Vec<Track>) {
        debug!("[Playlist] Force push tracks: {:?}", tracks);
        
        let _ = self.sender.lock().map(|sender| {
            let mut force_lock = self.force_lock.lock().unwrap();
            *force_lock = true;

            let mut existing_tracks: Vec<Track> = tracks;
    
            loop {
                match self.buffered_receiver.recv_timeout(Duration::from_millis(300)) {
                    Ok(buffered_track) => existing_tracks.push(buffered_track.track),
                    Err(_) => break,
                }
            }

            loop {
                match self.receiver.recv_timeout(Duration::from_millis(300)) {
                    Ok(track) => existing_tracks.push(track),
                    Err(_) => break,
                }
            }

            existing_tracks.iter()
                .for_each(|track| { let _ = sender.send(track.clone()); });

            debug!("[Playlist] playlist after push force: {:?}", existing_tracks);
        }).unwrap();
    }
}
