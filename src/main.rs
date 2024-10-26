use backend::BackendInitialization;
use env_logger::Target;
use interface::gui::Gui;

#[cfg(target_os = "macos")]
use interface::systray;

use log::error;
use macroquad::window::Conf;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use std::thread::{self, JoinHandle};

mod state;
use state::PlayerBus;

mod playlist;
use playlist::Playlist;

mod backend;

mod config;
use config::Config;

mod player;
mod interface;

use interface::http;

fn service_module(backend_init: BackendInitialization, playlist: Playlist) {
    thread::spawn(move || {
        backend_init.initialization();
        backend_init.get_initialized().listen_commands(playlist);
    });
}

fn downloader_module(playlist: Playlist, backend_init: BackendInitialization) {
    thread::spawn(move || {
        let backend = backend_init.get_initialized();
        backend.discover();

        playlist.buffer_worker(|track| {
            let mut backend = backend_init.get_initialized();
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

fn conf(config: Config) -> Conf {
    Conf {
        window_title: "Woodaudio".to_string(),
        fullscreen: config.gui.fullscreen,
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

    let backend_init = BackendInitialization::new(config.clone(), player_bus.clone());

    service_module(backend_init.clone(), playlist.clone());
    downloader_module(playlist.clone(), backend_init.clone());
    server_module(player_bus.clone());

    let player = player_module(playlist.clone(), player_bus.clone());
    
    #[cfg(target_os = "macos")]
    if config.gui.systray_enabled {
        Systray::init(player_bus.clone()).display();
    }

    if config.gui.enabled {
        macroquad::Window::from_config(conf(config), gui_module(player_bus.clone()));
    } else {
        let _ = player.join();
    }
}
