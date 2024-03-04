use env_logger::Target;
use rodio::{OutputStream, Decoder, Sink};
use serde_json::Value;

use std::error::Error;
use std::io::Cursor;
use std::time::Duration;
use std::thread;

use rand::thread_rng;
use rand::seq::SliceRandom;
use log::{error, info};

mod playerbus;
use playerbus::{PlayerBus, PlayerBusAction};

mod playlist;
use playlist::{BufferedTrack, Playlist, Track};

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

fn get_categories_from_for_you_page(session: &Session, playlist: &Playlist) -> Result<(), Box<dyn Error>> {
    let v = session.get_page_for_you()?;
    let mixes = parse_modules(v)?;

    shuffle_vec(mixes).iter()
        .filter(|mix| mix["mixType"].is_string())
        .map(|mix| session.get_mix(mix["id"].as_str().unwrap()).unwrap())
        .map(|mix| parse_modules(mix).unwrap())
        .flat_map(|mix_tracks| shuffle_vec(mix_tracks.clone()))
        .filter(|mix_track| mix_track["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready))
        .for_each(|track| {
            playlist.push(Track::build_from_json(track));
        });

    Ok(())
}

fn get_favorites_tracks(session: &Session, playlist: &Playlist) -> Result<(), Box<dyn Error>> {
    let v = session.get_favorites()?;

    if let serde_json::Value::Array(items) = &v["items"] {

        let mut rng = thread_rng();
        let mut shuffled_items = items.clone();
        shuffled_items.shuffle(&mut rng);
        
        for item in shuffled_items {
            if item["item"]["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                playlist.push(Track::build_from_json(item["item"].to_owned()));
            }
        }
    }

    Ok(())
}

fn download_file(track: Track, session: &Session) -> Result<BufferedTrack, Box<dyn Error>> {
    for _ in 1..5 {
        let url = session.get_track_url(track.id.clone())?;
        
        let file_response = reqwest::blocking::get(url)?;
        if !file_response.status().is_success() {
            continue;
        }

        return Ok(BufferedTrack {
            track: track,
            stream: file_response.bytes()?,
        })
    }
    panic!("Track Download fail!");
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

fn source(track: BufferedTrack) -> Option<Decoder<std::io::Cursor<bytes::Bytes>>> {
    let source_result = Decoder::new_flac(Cursor::new(track.stream));

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
            PlayerBusAction::PausePlay => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
                }
            },
            PlayerBusAction::NextSong => {
                sink.clear();
            },
            PlayerBusAction::None => {},
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

fn discovery_module_favorites(session: Session, playlist: Playlist) {
    thread::spawn(move || {
        let _ = get_favorites_tracks(&session, &playlist);
    });
}

fn discovery_module_categories_for_you(session: Session, playlist: Playlist) {
    thread::spawn(move || {
       let _ = get_categories_from_for_you_page(&session, &playlist);
    });
}

fn downloader_module(session: Session, playlist: Playlist) {
    thread::spawn(move || {
        playlist.buffer_worker(|track| {
            match download_file(track, &session) {
                Ok(buffered_track) => return buffered_track,
                Err(err) => panic!("{:?}", err),
            }
        });
    });
}

fn player_bus_server_module(session: Session, playlist: Playlist, player_bus: PlayerBus) {
    thread::spawn(move || {
        let server = Server::http("0.0.0.0:8001").unwrap();

        for mut request in server.incoming_requests() {
            if request.method().eq(&tiny_http::Method::Post) {
                info!("[Server control] {}", request.url());

                match request.url() {
                    "/action/next" => player_bus.call(PlayerBusAction::NextSong),
                    "/action/play_pause" => player_bus.call(PlayerBusAction::PausePlay),
                    "/action/track_radio" => {
                        let mut content = String::new();
                        request.as_reader().read_to_string(&mut content).unwrap();
                        info!("[Server control] detail action play radio by track {}", content);

                        let result: Value = serde_json::from_str(&content).expect("Json required in body");
                        let tidal_url = result["url"].as_str().expect("Json required url string field");

                        if tidal_url.starts_with("https://tidal.com/track/") {
                            let track_id = tidal_url.split("/").last();
                            let radio = session.get_track_radio(track_id.unwrap()).unwrap();

                            let mut tracks: Vec<Track> = vec![];

                            if let serde_json::Value::Array(items) = &radio["items"] {
                                for item in items {
                                    if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                                        tracks.push(Track::build_from_json(item.to_owned()));
                                    }
                                }
                            }

                            playlist.push_force(tracks);
                            player_bus.call(PlayerBusAction::NextSong);
                        }
                    },
                    "/action/play_by_url" => {
                        let mut content = String::new();
                        request.as_reader().read_to_string(&mut content).unwrap();
                        info!("[Server control] detail action play by url {}", content);

                        let result: Value = serde_json::from_str(&content).expect("Json required in body");
                        let tidal_url = result["url"].as_str().expect("Json required url string field");

                        if tidal_url.starts_with("https://tidal.com/track/") {
                            let track_id = tidal_url.split("/").last();

                            if track_id.is_some() {
                                playlist.push_force(vec![Track::unnamed_track(track_id.unwrap().to_string())]);
                                player_bus.call(PlayerBusAction::NextSong);
                            }
                        }

                        if tidal_url.starts_with("https://tidal.com/album/") {
                            let album_id = tidal_url.split("/").last();
                            let album = session.get_album(album_id.unwrap()).unwrap();
                            let mut tracks: Vec<Track> = vec![];

                            if let serde_json::Value::Array(items) = &album["items"] {
                                for item in items {
                                    if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                                        tracks.push(Track::build_from_json(item.to_owned()));
                                    }
                                }
                            }

                            playlist.push_force(tracks);
                            player_bus.call(PlayerBusAction::NextSong);
                        }

                        if tidal_url.starts_with("https://tidal.com/artist/") {
                            let artist_id = tidal_url.split("/").last();
                            let artist = session.get_artist(artist_id.unwrap()).unwrap();
                            let mut tracks: Vec<Track> = vec![];

                            if let serde_json::Value::Array(items) = &artist["items"] {
                                for item in items {
                                    if item["adSupportedStreamReady"].as_bool().is_some_and(|ready| ready) {
                                        tracks.push(Track::build_from_json(item.to_owned()));
                                    }
                                }
                            }

                            playlist.push_force(tracks);
                            player_bus.call(PlayerBusAction::NextSong);
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
    let session = Session::init_from_config_file().unwrap();
    
    discovery_module_categories_for_you(session.clone(), playlist.clone());
    discovery_module_favorites(session.clone(), playlist.clone());

    downloader_module(session.clone(), playlist.clone());

    player_bus_server_module(session.clone(), playlist.clone(), player_bus.clone());

    player_module(session.clone(), playlist.clone(), player_bus.clone());
}
