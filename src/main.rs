use backend::BackendInitialization;
use env_logger::Target;

use log::error;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use std::thread::{self, JoinHandle};
use std::time::Duration;

mod state;
use state::PlayerBus;

mod playlist;
use playlist::Playlist;

mod backend;

mod config;
use config::Config;

mod player;
mod interface;

#[cfg(feature = "gui")]
use interface::gui::Gui;

#[cfg(feature = "gpio")]
use crate::interface::gpio::Gpio;

fn service_module(backend_init: BackendInitialization, playlist: Playlist) {
    thread::spawn(move || {
        backend_init.initialization();
        backend_init.get_initialized().listen_commands(playlist);
    });
}

fn downloader_module(playlist: Playlist, backend_init: BackendInitialization) {
    thread::spawn(move || {
        let backend = backend_init.get_initialized();

        while !backend.is_listener_ready() {
            thread::sleep(Duration::from_millis(10));
        }

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

fn player_module(config: Config, playlist: Playlist, player_bus: PlayerBus) -> JoinHandle<()> {
    thread::Builder::new()
        .name("Player module".to_owned())
        .spawn_with_priority(ThreadPriority::Max, move |_| {
            player::player(&config, &playlist, player_bus);
    }).unwrap()
}

#[cfg(feature = "gui")]
fn gui_module(config: Config, player_bus: PlayerBus) {
    Gui::init(config, player_bus.clone())
        .gui_loop()
}

#[cfg(feature = "gpio")]
fn gpio_module(config: Config, player_bus: PlayerBus) {
    thread::Builder::new()
        .name("GPIO module".to_owned())
        .spawn_with_priority(ThreadPriority::Min, move |_| {
            Gpio::new(config, player_bus.clone())
                .wait().expect("Gpio module error");
        }).unwrap();
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

    #[cfg(feature = "gpio")]
    gpio_module(config.clone(), player_bus.clone());

    let player = player_module(config.clone(), playlist.clone(), player_bus.clone());

    #[cfg(feature = "gui")]
    if config.gui.enabled {
        gui_module(config.clone(), player_bus.clone());
    } else {
        let _ = player.join();
    }

    #[cfg(not(feature = "gui"))]
    let _ = player.join();
}
