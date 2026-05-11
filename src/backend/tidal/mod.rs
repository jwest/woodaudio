use std::{error::Error, time::Duration};
use bytes::Bytes;
use log::info;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;
use serde_json::Value;

use crate::{config::Config, state::PlayerBus, playlist::Track};
use self::session::Session;
use super::Backend;

mod session;

#[derive(Clone)]
pub struct TidalBackend {
    session: Session,
}

impl Backend for TidalBackend {
    fn init(config: &mut Config, player_bus: PlayerBus) -> Self {
        let session = Session::setup(config, player_bus.clone());
        Self {
            session: session.clone(),
        }
    }
    fn discovery(&self, discovery_fn: impl Fn(Track)) {
        let _ = self.discover_mixes(&self.session, &discovery_fn);
        let _ = self.discover_favorities_tracks(&self.session, &discovery_fn);
    }
    fn get_track_url(&mut self, track_id: String) -> Result<String, Box<dyn Error>> {
        for _ in 1..5 {
            TidalBackend::sleep_for_health();

            match self.session.get_track_url(track_id.clone()) {
                Ok(url) => return Ok(url),
                Err(_) => continue,
            }
        }

        Err("Track URL fetch fail!".into())
    }
    fn get_cover(&self, cover_url: String) -> Result<Bytes, Box<dyn Error>> {
        self.session.get_cover_bytes(cover_url)
    }
    fn discovery_radio(&self, track_id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        self.discovery_track(track_id, discovery_fn);
    }
    fn discovery_track(&self, track_id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        info!("[Discovery] Discover radio for track: {}", track_id);
        TidalBackend::sleep_for_health();
        let radio = self.session.get_track_radio(track_id).unwrap();
        let tracks = Self::parse_tracks(&radio["items"]);

        info!("[Discovery] Discover tracks: {:?}", tracks);
        discovery_fn(tracks);
    }
    fn discovery_album(&self, album_id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        TidalBackend::sleep_for_health();
        let album = self.session.get_album(album_id).unwrap();
        let tracks = Self::parse_tracks(&album["items"]);

        info!("[Discovery] Discover tracks {:?} from album: {}", tracks, album_id);
        discovery_fn(tracks);
    }
    fn discovery_artist(&self, artist_id: &str, discovery_fn: impl Fn(Vec<Track>)) {
        TidalBackend::sleep_for_health();
        let artist = self.session.get_artist(artist_id).unwrap();
        let tracks = Self::parse_tracks(&artist["items"]);

        info!("[Discovery] Discover tracks {:?} from artist: {}", tracks, artist_id);
        discovery_fn(tracks);
    }
    fn add_track_to_favorites(&self, track_id: &str) {
        let _ = self.session.add_track_to_favorites(track_id);
    }
}

impl TidalBackend {
    fn discover_favorities_tracks(&self, session: &Session, discovery_fn: impl Fn(Track)) -> Result<(), Box<dyn Error>> {
        let v = session.get_favorites()?;

        if let Value::Array(items) = &v["items"] {
            let mut rng = thread_rng();
            let mut shuffled_items = items.clone();
            shuffled_items.shuffle(&mut rng);

            for item in shuffled_items {
                if item["item"]["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    discovery_fn(Track::build_from_json(item["item"].clone()));
                }
            }
        }

        Ok(())
    }

    fn discover_mixes(&self, session: &Session, discovery_fn: impl Fn(Track)) -> Result<(), Box<dyn Error>> {
        let v = session.get_page_for_you()?;
        let mut mixes = parse_modules(v)?;
        let mut rng = thread_rng();
        mixes.shuffle(&mut rng);

        let mut all_tracks: Vec<Value> = mixes.into_iter()
            .filter(|mix| mix["mixType"].is_string())
            .filter_map(|mix| session.get_mix(mix["id"].as_str().unwrap()).ok())
            .filter_map(|mix| parse_modules(mix).ok())
            .flat_map(|mut mix_tracks| {
                mix_tracks.shuffle(&mut thread_rng());
                mix_tracks
            })
            .filter(|t| t["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready))
            .collect();

        all_tracks.shuffle(&mut rng);

        for track in all_tracks {
            discovery_fn(Track::build_from_json(track));
        }

        Ok(())
    }

    fn parse_tracks(items: &Value) -> Vec<Track> {
        let mut tracks: Vec<Track> = vec![];

        if let Value::Array(items) = items {
            for item in items {
                if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                    tracks.push(Track::build_from_json(item.to_owned()));
                }
            }
        }

        tracks
    }

    fn sleep_for_health() {
        let mut rng = rand::thread_rng();
        std::thread::sleep(Duration::from_millis(rng.gen_range(800..1200)));
    }
}

impl Track {
    pub fn build_from_json(item: Value) -> Track {
        let cover = if item["album"]["cover"].is_string() { 
            item["album"]["cover"].as_str().unwrap()
        } else { 
            "0dfd3368-3aa1-49a3-935f-10ffb39803c0" 
        }.replace('-', "/");

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
}

fn parse_modules(value: Value) -> Result<Vec<Value>, Box<dyn Error>> {
    let modules = value["rows"].as_array().unwrap().iter()
        .flat_map(|row| row.as_object().unwrap()["modules"].as_array().unwrap())
        .filter(|module| module["pagedList"]["items"].is_array())
        .flat_map(|module| module.as_object().unwrap()["pagedList"]["items"].as_array().unwrap().clone())
        .collect::<Vec<Value>>();

    Ok(modules)
}