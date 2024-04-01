use env_logger::Target;
use gui::Gui;
use log::error;
use macroquad::window::Conf;
use session::Session;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use std::thread::{self, JoinHandle};

mod playerbus;
use playerbus::PlayerBus;

mod playlist;
use playlist::Playlist;

mod session;

mod discovery;
use discovery::DiscoveryStore;

mod downloader;
use downloader::Downloader;

mod config;
use config::Config;

mod player;
mod gui;
mod http;

fn session_module(config: Config, player_bus: PlayerBus) {
    thread::spawn(move || {
        Session::setup(&mut config.clone(), player_bus);
    });
}

fn service_module(player_bus: PlayerBus, discovery_store: DiscoveryStore) {
    thread::spawn(move || {
        discovery_store.listen_commands(player_bus);
    });
}

fn discovery_module(player_bus: PlayerBus, discovery_store: DiscoveryStore) {
    thread::spawn(move || {
        let session = player_bus.wait_for_session();
        let _ = discovery_store.discover_mixes(&session);
        let _ = discovery_store.discover_favorities_tracks(&session);
    });
}

fn downloader_module(player_bus: PlayerBus, config: Config, playlist: Playlist) {
    thread::spawn(move || {
        let session = player_bus.wait_for_session();
        let downloader = Downloader::init(&session, &config);

        playlist.buffer_worker(|track| {
            match downloader.download_file(track) {
                Ok(buffered_track) => Some(buffered_track),
                Err(err) => { error!("[Downloader] download file error: {:?}", err); None },
            }
        });
    });
}

fn server_module(player_bus: PlayerBus) {
    thread::Builder::new()
        .name("Server module".to_owned())
        .spawn_with_priority(ThreadPriority::Min, move |_| {
            http::server(&player_bus);
    }).unwrap();
}

fn player_module(playlist: Playlist, player_bus: PlayerBus) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Player module".to_owned())
        .spawn_with_priority(ThreadPriority::Max, move |_| {
            player::player(&playlist, player_bus);
    }).unwrap()
}

async fn gui_module(player_bus: PlayerBus) {
    Gui::init(player_bus.clone())
        .gui_loop()
        .await;
}

fn conf() -> Conf {
    Conf {
      window_title: "Woodaudio".to_string(),
      fullscreen: true,
      window_height: 600,
      window_width: 1024,
      window_resizable: false,
      ..Default::default()
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let config = Config::init_default_path();
    let playlist = Playlist::new();
    let player_bus = PlayerBus::new();
    let discovery_store = DiscoveryStore::new(playlist.clone());

    session_module(config.clone(), player_bus.clone());
    discovery_module(player_bus.clone(), discovery_store.clone());
    service_module(player_bus.clone(), discovery_store.clone());
    downloader_module(player_bus.clone(), config.clone(), playlist.clone());
    server_module(player_bus.clone());

    let player = player_module(playlist.clone(), player_bus.clone());

    if config.gui.enabled {
        macroquad::Window::from_config(conf(), gui_module(player_bus.clone()));
    } else {
        let _ = player.join();
    }
}