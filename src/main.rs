use env_logger::Target;
use rodio::{OutputStream, Decoder, Sink};
use serde_json::Value;
use tempfile::NamedTempFile;

use std::error::Error;
use std::fs::File;
use std::io::{Cursor, BufReader};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use std::thread;

use rand::thread_rng;
use rand::seq::SliceRandom;
use log::{error, info};

mod eventbus;
use eventbus::{EventBus, PlayerBus, Playlist, Track};

mod session;
use session::Session;

use tiny_http::{Server, Response};

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

fn get_categories_from_for_you_page(session: &Session, bus: &EventBus) -> Result<(), Box<dyn Error>> {
    let v = session.get_page_for_you()?;
    let mixes = parse_modules(v)?;

    shuffle_vec(mixes).iter()
        .filter(|mix| mix["mixType"].is_string())
        .map(|mix| session.get_mix(mix["id"].as_str().unwrap()).unwrap())
        .map(|mix| parse_modules(mix).unwrap())
        .flat_map(|mix_tracks| shuffle_vec(mix_tracks.clone()))
        .filter(|mix_track| mix_track["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready))
        .for_each(|track| {
            let _ = bus.track_discovered(Track { 
                id: track["id"].as_i64().unwrap().to_string(),
                full_name: format!("{} - {}", track["artists"][0]["name"], track["title"]), 
                file_path: None,
            });
        });

    Ok(())
}

fn get_favorites_tracks(session: &Session, bus: &EventBus) -> Result<(), Box<dyn Error>> {
    let v = session.get_favorites()?;

    if let serde_json::Value::Array(items) = &v["items"] {

        let mut rng = thread_rng();
        let mut shuffled_items = items.clone();
        shuffled_items.shuffle(&mut rng);
        
        for item in shuffled_items {
            if item["item"]["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                let _ = bus.track_discovered(Track { 
                    id: item["item"]["id"].as_i64().unwrap().to_string(),
                    full_name: format!("{} - {}", item["item"]["artist"]["name"], item["item"]["title"]), 
                    file_path: None,
                });
            }
        }
    }

    Ok(())
}

fn download_file(track: &Track, session: &Session) -> Result<Option<Track>, Box<dyn Error>> {
    for _ in 1..5 {
        let url = session.get_track_url(track.id.clone())?;
        
        let file_response = reqwest::blocking::get(url)?;
        if !file_response.status().is_success() {
            continue;
        }

        let (mut tmp_file, tmp_path) = NamedTempFile::new()?.keep()?;
        let mut content =  Cursor::new(file_response.bytes()?);

        std::io::copy(&mut content, &mut tmp_file)?;

        return Ok(Some(Track {
            full_name: track.full_name.clone(),
            id: track.id.clone(),
            file_path: Some(String::from_str(tmp_path.to_str().unwrap())?),
        }))
    }
    Ok(None)
}

fn retry<T, E>(function: fn() -> Result<T, E>) -> T where E: std::fmt::Display {
    match function() {
        Ok(output) => output,
        Err(err) => {
            error!("[Player] Load audio output fail, retry... ({:?})", err.to_string());
            thread::sleep(Duration::from_secs(3));
            retry(function)
        },
    }
}

fn source(track: Track) -> Option<Decoder<BufReader<File>>> {
    let afp = track.file_path.as_ref().unwrap();
    let audio_file_path = Path::new(afp.as_str());
    let audio_file = match File::open(audio_file_path) {
        Ok(it) => it,
        Err(err) => {
            error!("[Player] Audio file '{:?}' not exists, try next...", err);
            return None
        },
    };
    let file = BufReader::new(audio_file);
    let source_result = Decoder::new_flac(file);

    match source_result {
        Ok(file) => Some(file),
        Err(err) => {
            error!("[Player] Audio file '{:?}' decode error, try next...", err);
            return None
        },
    }
}

fn player(playlist: Playlist, player_bus: PlayerBus) {
    let (_stream, stream_handle) = retry(OutputStream::try_default);
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    sink.play();

    loop {
        match player_bus.read() {
            eventbus::PlayerBusAction::PausePlay => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            },
            eventbus::PlayerBusAction::NextSong => {
                sink.clear();
            },
            eventbus::PlayerBusAction::None => {},
        };
        match sink.empty() {
            true => {
                match playlist.pop() {
                    Some(track) => {  
                        let source = source(track);
                        if source.is_some() {
                            sink.append(source.unwrap());
                            sink.play();
                        }
                    }
                    None => thread::sleep(Duration::from_millis(200)),
                }
            },
            false => thread::sleep(Duration::from_millis(200)),
        }
    }
}

fn downloader(session: Session, bus: EventBus) {
    bus.on_track_discovered(|track| {
        match download_file(track, &session) {
            Ok(file) => if file.is_some() { 
                bus.track_downloaded(file.unwrap());
            },
            Err(err) => println!("{:?}", err),
        }
    });
}

fn discovery_module_favorites(session: Session, bus: EventBus) {
    thread::spawn(move || {
        let _ = get_favorites_tracks(&session, &bus);
    });
}

fn discovery_module_categories_for_you(session: Session, bus: EventBus) {
    thread::spawn(move || {
       let _ = get_categories_from_for_you_page(&session, &bus);
    });
}

fn downloader_module(session: Session, bus: EventBus) {
    thread::spawn(move || {
        let _ = downloader(session, bus);
    });
}

fn player_bus_server_module(session: Session, playlist: Playlist, player_bus: PlayerBus) {
    thread::spawn(move || {
        let server = Server::http("0.0.0.0:8001").unwrap();

        for mut request in server.incoming_requests() {
            if request.method().eq(&tiny_http::Method::Post) {
                info!("[Server control] {}", request.url());

                match request.url() {
                    "/action/next" => player_bus.call(eventbus::PlayerBusAction::NextSong),
                    "/action/play_pause" => player_bus.call(eventbus::PlayerBusAction::PausePlay),
                    "/action/play_by_url" => {
                        let mut content = String::new();
                        request.as_reader().read_to_string(&mut content).unwrap();
                        info!("[Server control] detail action play by url {}", content);

                        if content.starts_with("https://tidal.com/track/") {
                            let track_id = content.split("/").last();

                            if track_id.is_some() {
                                let track = Track { 
                                    id: track_id.unwrap().to_string(),
                                    full_name: format!("{}", track_id.unwrap().to_string()),
                                    file_path: None,
                                 };
                                let download_file = download_file(&track, &session).unwrap().unwrap();
                                info!("[Server control] force {:?}", download_file);
                                
                                playlist.push_force([download_file].to_vec());
                                player_bus.call(eventbus::PlayerBusAction::NextSong);
                            }
                        }

                        if content.starts_with("https://tidal.com/album/") {
                            let album_id = content.split("/").last();
                            let album = session.get_album(album_id.unwrap()).unwrap();
                            let mut tracks: Vec<Track> = vec![];

                            info!("[Server control] force album {:?}", album);
                            if let serde_json::Value::Array(items) = &album["items"] {
                                info!("[Server control] force ITEMS");
                                for item in items {
                                    info!("[Server control] force ITEM");
                                    if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                                        info!("[Server control] force adSupportedStreamReady");
                                        let track = Track { 
                                            id: item["id"].as_i64().unwrap().to_string(),
                                            full_name: format!("{} - {}", item["artist"]["name"], item["title"]), 
                                            file_path: None,
                                        };
                                        let download_file = download_file(&track, &session).unwrap().unwrap();
                                        tracks.push(download_file);
                                        info!("[Server control] force {:?}", track);
                                    }
                                }
                            }

                            playlist.push_force(tracks);
                            player_bus.call(eventbus::PlayerBusAction::NextSong);
                        }

                        if content.starts_with("https://tidal.com/artist/") {
                            let artist_id = content.split("/").last();
                            let artist = session.get_artist(artist_id.unwrap()).unwrap();
                            let mut tracks: Vec<Track> = vec![];

                            info!("[Server control] force artist {:?}", artist);
                            if let serde_json::Value::Array(items) = &artist["items"] {
                                info!("[Server control] force ITEMS");
                                for item in items {
                                    info!("[Server control] force ITEM");
                                    if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                                        info!("[Server control] force adSupportedStreamReady");
                                        let track = Track { 
                                            id: item["id"].as_i64().unwrap().to_string(),
                                            full_name: format!("{} - {}", item["artist"]["name"], item["title"]), 
                                            file_path: None,
                                        };
                                        let download_file = download_file(&track, &session).unwrap().unwrap();
                                        tracks.push(download_file);
                                        info!("[Server control] force {:?}", track);
                                    }
                                }
                            }

                            playlist.push_force(tracks);
                            player_bus.call(eventbus::PlayerBusAction::NextSong);
                        }
                    },
                    _ => {}
                }

                let _ = request.respond(Response::empty(200));
            } else {
                let _ = request.respond(Response::empty(404));
            }
        }
    });
}

fn player_module(_: Session, playlist: Playlist, player_bus: PlayerBus) {
    let player_thread = thread::spawn(|| {
        let _ = player(playlist, player_bus);
    });

    player_thread.join().expect("oops! the [player] thread panicked");
}

fn main() {
    env_logger::Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let playlist = Playlist::new();
    let player_bus = PlayerBus::new();
    let bus = EventBus::new(playlist.clone());
    let session = Session::init_from_config_file().unwrap();
    
    discovery_module_favorites(session.clone(), bus.clone());
    discovery_module_categories_for_you(session.clone(), bus.clone());

    downloader_module(session.clone(), bus.clone());

    player_bus_server_module(session.clone(), playlist.clone(), player_bus.clone());

    player_module(session.clone(), playlist.clone(), player_bus.clone());
}
