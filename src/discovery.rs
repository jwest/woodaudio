use std::error::Error;

use log::info;
use rand::thread_rng;
use rand::seq::SliceRandom;
use serde_json::Value;

use crate::{playlist::{Playlist, Track}, session::Session};

#[derive(Debug)]
enum DiscoverablePriority {
    Low,
    High,
}

#[derive(Debug)]
#[derive(Clone)]
struct DiscoveryQueue {
    playlist: Playlist,
}

impl DiscoveryQueue {
    fn push(&self, priority: DiscoverablePriority, track: Track) {
        match priority {
            DiscoverablePriority::Low => self.playlist.push(vec![track]),
            DiscoverablePriority::High => self.playlist.push_force(vec![track]),
        }
    }
    fn push_tracks(&self, priority: DiscoverablePriority, tracks: Vec<Track>) {
        match priority {
            DiscoverablePriority::Low => self.playlist.push(tracks),
            DiscoverablePriority::High => self.playlist.push_force(tracks),
        }
    }
}

fn shuffle_vec(items: Vec<Value>) -> Vec<Value> {
    let mut rng_items = thread_rng();
    let mut items_clone = items.clone();
    items_clone.shuffle(&mut rng_items);
    items_clone
}

fn parse_modules(value: Value) -> Result<Vec<Value>, Box<dyn Error>> {
    let modules = value["rows"].as_array().unwrap().iter()
        .flat_map(|row| row.as_object().unwrap()["modules"].as_array().unwrap())
        .filter(|module| module["pagedList"]["items"].is_array())
        .flat_map(|module| module.as_object().unwrap()["pagedList"]["items"].as_array().unwrap().clone())
        .collect::<Vec<Value>>();

    Ok(modules)
}

#[derive(Debug)]
#[derive(Clone)]
pub struct DiscoveryStore {
    session: Session, 
    queue: DiscoveryQueue,
}

impl DiscoveryStore  {
    pub fn new(session: Session, playlist: Playlist) -> Self {
        DiscoveryStore {
            session,
            queue: DiscoveryQueue { playlist },
        }
    }

    pub fn discover_favorities_tracks(&self) -> Result<(), Box<dyn Error>> {
        let v = self.session.get_favorites()?;

        if let serde_json::Value::Array(items) = &v["items"] {
            let mut rng = thread_rng();
            let mut shuffled_items = items.clone();
            shuffled_items.shuffle(&mut rng);
            
            for item in shuffled_items {
                if item["item"]["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    self.queue.push(DiscoverablePriority::Low, Track::build_from_json(item["item"].clone()));
                }
            }
        }

        Ok(())
    }

    pub fn discover_mixes(&self) -> Result<(), Box<dyn Error>> {
        let v = self.session.get_page_for_you()?;
        let mixes = parse_modules(v)?;

        for track in &shuffle_vec(
            shuffle_vec(mixes).iter()
                .filter(|mix| mix["mixType"].is_string())
                .map(|mix| self.session.get_mix(mix["id"].as_str().unwrap()).unwrap())
                .map(|mix| parse_modules(mix).unwrap())
                .flat_map(|mix_tracks| shuffle_vec(mix_tracks.clone()))
                .filter(|mix_track| mix_track["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready))
                .collect()
        ) {
                self.queue.push(DiscoverablePriority::Low, Track::build_from_json(track.clone()));
            }

        Ok(())
    }

    pub fn discovery_radio(&self, track_id: &str) -> Result<(), Box<dyn Error>> {
        info!("[Discovery] Discover radio for track: {}", track_id);
        let radio = self.session.get_track_radio(track_id).unwrap();
        let mut tracks: Vec<Track> = vec![];

        if let serde_json::Value::Array(items) = &radio["items"] {
            for item in items {
                if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    tracks.push(Track::build_from_json(item.to_owned()));
                }
            }
        }

        info!("[Discovery] Discover tracks: {:?}", tracks);

        self.queue.push_tracks(DiscoverablePriority::High, tracks);

        Ok(())
    }

    pub fn discovery_track(&self, track_id: &str) -> Result<(), Box<dyn Error>> {
        self.discovery_radio(track_id)
    }

    pub fn discovery_album(&self, album_id: &str) -> Result<(), Box<dyn Error>> {
        let album = self.session.get_album(album_id).unwrap();
        let mut tracks: Vec<Track> = vec![];

        if let serde_json::Value::Array(items) = &album["items"] {
            for item in items {
                if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    tracks.push(Track::build_from_json(item.to_owned()));
                }
            }
        }

        info!("[Discovery] Discover tracks {:?} from album: {}", tracks, album_id);

        self.queue.push_tracks(DiscoverablePriority::High, tracks);

        Ok(())
    }

    pub fn discovery_artist(&self, artist_id: &str) -> Result<(), Box<dyn Error>> {
        let artist = self.session.get_artist(artist_id).unwrap();
        let mut tracks: Vec<Track> = vec![];

        if let serde_json::Value::Array(items) = &artist["items"] {
            for item in items {
                if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    tracks.push(Track::build_from_json(item.to_owned()));
                }
            }
        }

        info!("[Discovery] Discover tracks {:?} from artist: {}", tracks, artist_id);

        self.queue.push_tracks(DiscoverablePriority::High, tracks);
        Ok(())
    }
}