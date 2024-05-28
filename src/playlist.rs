use core::fmt;
use std::{sync::{Arc, Mutex}, thread, time::Duration};
use bytes::Bytes;
use crossbeam_channel::{unbounded, Receiver, Sender};

use log::{debug, error, info};

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
    pub fn dummy() -> Self {
        Self {
            id: "".to_owned(),
            album_name: "".to_owned(),
            artist_name: "".to_owned(),
            duration: Duration::from_secs(1),
            album_image: "".to_owned(),
            title: "".to_owned(),
        }
    }
    pub fn duration_formated(&self) -> String {
        let seconds = self.duration.as_secs() % 60;
        let minutes = (self.duration.as_secs() / 60) % 60;
        format!("{minutes}:{seconds:0>2}")
    }

    pub fn full_name(&self) -> String {
        format!("{} - {}: {} ({})", self.artist_name, self.album_name, self.title, self.duration_formated())
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum PlayableItemMediaType {
    Album,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayableItemId {
    id: String,
    media_type: PlayableItemMediaType
}

impl PlayableItemId {
    pub fn album(id: String) -> Self {
        Self { id, media_type: PlayableItemMediaType::Album }
    }

    pub fn get_id(&self) -> String { self.id.clone() }
    pub fn get_media_type(&self) -> PlayableItemMediaType { self.media_type.clone() }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct PlayableItem {
    id: PlayableItemId,
    artist: String,
    title: String,
    cover: Option<Cover>,
}

impl PlayableItem {
    pub fn init(id: PlayableItemId, artist: String, title: String, cover: Option<Cover>) -> Self {
        Self {
            id, artist, title, cover,
        }
    }

    pub fn get_id(&self) -> PlayableItemId { self.id.clone() }
    pub fn get_artist(&self) -> String { self.artist.clone() }
    pub fn get_title(&self) -> String { self.title.clone() }
    pub fn get_cover(&self) -> Option<Cover> { self.cover.clone() }
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

#[derive(Debug)]
#[derive(Clone)]
pub struct BufferedCover {
    pub url: String,
    pub path: String,
}

impl BufferedCover {
    pub fn id(&self) -> String {
        self.url.clone()
    }
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
                    let old_force_lock = *self.force_lock.lock().unwrap();
                    
                    info!("[Playlist worker] Buffer track: {:?}", track);
                    match f(track.clone()) {
                        Some(buffered_track) => {
                            let mut force_lock = self.force_lock.lock().unwrap();
                            if !old_force_lock && force_lock.eq(&true) {
                                info!("[Playlist worker] force lock ignore trakc {:?}", track);
                                *force_lock = false;
                            } else {
                                let _ = self.buffered_sender.send(buffered_track);
                            }
                        },
                        None => {
                            error!("[Playlist worker] Buffered track is empty {:?}", track);
                        },
                    }
                },
                Err(_) => thread::sleep(Duration::from_secs(3)),
            }
        }
    }

    pub fn push(&self, tracks: Vec<Track>) {
        debug!("[Playlist] Push tracks: {:?}", tracks);
        let _ = self.sender.lock().map(|sender| {
            for t in &tracks { let _ = sender.send(t.clone()); }
        });
    }

    pub fn push_buffered(&self, tracks: Vec<BufferedTrack>) {
        debug!("[Playlist] Push tracks: {:?}", tracks);
        for track in &tracks { let _ = self.buffered_sender.send(track.clone()); }
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
        
        self.sender.lock().map(|sender| {
            let mut force_lock = self.force_lock.lock().unwrap();
            *force_lock = true;

            let mut existing_tracks: Vec<Track> = tracks;
    
            let mut next_buffered_exist = true;
            while next_buffered_exist {
                match self.buffered_receiver.recv_timeout(Duration::from_millis(300)) {
                    Ok(buffered_track) => existing_tracks.push(buffered_track.track),
                    Err(_) => next_buffered_exist = false,
                }
            }

            let mut next_exist = true;
            while next_exist {
                match self.receiver.recv_timeout(Duration::from_millis(300)) {
                    Ok(track) => existing_tracks.push(track),
                    Err(_) => next_exist = false,
                }
            }

            for track in &existing_tracks { let _ = sender.send(track.clone()); }

            debug!("[Playlist] playlist after push force: {:?}", existing_tracks);
        }).unwrap();
    }
}
