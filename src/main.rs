use backend::{session::Session, Backend, TidalBackend};
use env_logger::Target;
use gui::{systray::Systray, Gui};
use log::error;
use macroquad::window::Conf;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use std::thread::{self, JoinHandle};

mod playerbus;
use playerbus::PlayerBus;

mod playlist;
use playlist::Playlist;

mod backend;

mod config;
use config::Config;

mod player;
mod http;
mod gui;

fn session_module(config: Config, player_bus: PlayerBus) {
    thread::spawn(move || {
        Session::setup(&mut config.clone(), player_bus);
    });
}

fn service_module(backend: Backend, playlist: Playlist) {
    thread::spawn(move || {
        backend.listen_commands(playlist);
    });
}

fn discovery_module(backend: Backend) {
    thread::spawn(move || {
        backend.discover();
    });
}

fn downloader_module(player_bus: PlayerBus, playlist: Playlist, backend: Backend) {
    thread::spawn(move || {
        player_bus.wait_for_session();

        playlist.buffer_worker(|track| {
            match backend.download(track) {
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

    let mut config = Config::init_default_path();
    let playlist = Playlist::new();
    let player_bus = PlayerBus::new();
    let backend = Backend::init(
        TidalBackend::init(&mut config, player_bus.clone()), player_bus.clone()
    );

    session_module(config.clone(), player_bus.clone());
    discovery_module(backend.clone());
    service_module(backend.clone(), playlist.clone());
    downloader_module(player_bus.clone(), playlist.clone(), backend);
    server_module(player_bus.clone());

    let player = player_module(playlist.clone(), player_bus.clone());
    
    #[cfg(target_os = "macos")]
    if config.gui.systray_enabled {
        Systray::init(player_bus.clone()).display();
    }

    if config.gui.enabled {
        macroquad::Window::from_config(conf(), gui_module(player_bus.clone()));
    } else {
        let _ = player.join();
    }
}