use env_logger::Target;
use gui::Gui;
use macroquad::window::Conf;
use serde_json::Value;
use std::thread;

use log::info;

use tiny_http::{Server, Response};

mod playerbus;
use playerbus::{PlayerBus, PlayerBusAction};

mod playlist;
use playlist::Playlist;

mod session;
use session::Session;

mod player;

mod discovery;
use discovery::DiscoveryStore;

mod downloader;
mod gui;

fn discovery_module_favorites(discovery_store: DiscoveryStore) {
    thread::spawn(move || {
        let _ = discovery_store.discover_favorities_tracks();
    });
}

fn discovery_module_categories_for_you(discovery_store: DiscoveryStore) {
    thread::spawn(move || {
        let _ = discovery_store.discover_mixes();
    });
}

fn downloader_module(session: Session, playlist: Playlist) {
    thread::spawn(move || {
        playlist.buffer_worker(|track| {
            match downloader::download_file(track, &session) {
                Ok(buffered_track) => return buffered_track,
                Err(err) => panic!("{:?}", err),
            }
        });
    });
}

fn server_module(discovery_store: DiscoveryStore, player_bus: PlayerBus) {
    thread::spawn(move || {
        let server = Server::http("0.0.0.0:8001").unwrap();

        for mut request in server.incoming_requests() {
            if request.method().eq(&tiny_http::Method::Post) {
                info!("[Server control] {}", request.url());

                match request.url() {
                    "/action/next" => player_bus.call(PlayerBusAction::NextSong),
                    "/action/play_pause" => player_bus.call(PlayerBusAction::PausePlay),
                    "/action/play_by_url" => {
                        let mut content = String::new();
                        request.as_reader().read_to_string(&mut content).unwrap();
                        info!("[Server control] detail action play by url {}", content);

                        let result: Value = serde_json::from_str(&content).expect("Json required in body");
                        let tidal_url = result["url"].as_str().expect("Json required url string field");
                        let id = tidal_url.split("/").last().unwrap();

                        if tidal_url.starts_with("https://tidal.com/track/") {
                            let _ = discovery_store.discovery_track(id);
                            player_bus.call(PlayerBusAction::NextSong);
                        }

                        if tidal_url.starts_with("https://tidal.com/album/") {
                            let _ = discovery_store.discovery_album(id);
                            player_bus.call(PlayerBusAction::NextSong);
                        }

                        if tidal_url.starts_with("https://tidal.com/artist/") {
                            let _ = discovery_store.discovery_artist(id);
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

fn player_module(playlist: Playlist, player_bus: PlayerBus) {
    thread::spawn(move || {
        let _ = player::player(playlist, player_bus);
    });
}

fn conf() -> Conf {
    Conf {
      window_title: "Woodaudio".to_string(),
      fullscreen: true,
      window_height: 600,
      window_width: 1024,
      ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    env_logger::Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let playlist = Playlist::new();
    let player_bus = PlayerBus::new();
    let session = Session::init_from_config_file().unwrap();
    let discovery_store = DiscoveryStore::new(session.clone(), playlist.clone());
    
    discovery_module_categories_for_you(discovery_store.clone());
    discovery_module_favorites(discovery_store.clone());
    server_module(discovery_store.clone(), player_bus.clone());

    downloader_module(session.clone(), playlist.clone());

    player_module(playlist.clone(), player_bus.clone());

    let mut gui = Gui::default_state();

    gui.gui_loop(player_bus.clone(), discovery_store.clone()).await;
}