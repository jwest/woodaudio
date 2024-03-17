use env_logger::Target;
use gui::Gui;
use macroquad::window::Conf;
use thread_priority::{ThreadBuilderExt, ThreadPriority};
use std::{thread, time::Duration};

mod playerbus;
use playerbus::PlayerBus;

mod playlist;
use playlist::Playlist;

mod session;
use session::Session;

mod sessiongui;
use sessiongui::SessionGui;

mod discovery;
use discovery::DiscoveryStore;

mod player;
mod downloader;
mod gui;
mod http;

fn service_module(discovery_store: DiscoveryStore, mut player_bus: PlayerBus, session: Session) {
    thread::spawn(move || {
        let channel = player_bus.register_command_channel(vec!["Radio".to_string(), "PlayTrackForce".to_string(), "PlayAlbumForce".to_string(), "PlayArtistForce".to_string(), "Like".to_string()]);

        loop {
            let command = channel.read_command();

            match command {
                Some(playerbus::Command::Radio(track_id)) => {
                    let _ = discovery_store.discovery_radio(&track_id);
                    player_bus.publish_message(playerbus::Message::ForcePlay);
                },
                Some(playerbus::Command::PlayTrackForce(track_id)) => {
                    let _ = discovery_store.discovery_track(&track_id);
                    player_bus.publish_message(playerbus::Message::ForcePlay);
                },
                Some(playerbus::Command::PlayAlbumForce(track_id)) => {
                    let _ = discovery_store.discovery_album(&track_id);
                    player_bus.publish_message(playerbus::Message::ForcePlay);
                },
                Some(playerbus::Command::PlayArtistForce(track_id)) => {
                    let _ = discovery_store.discovery_artist(&track_id);
                    player_bus.publish_message(playerbus::Message::ForcePlay);
                },
                Some(playerbus::Command::Like(track_id)) => {
                    let _ = session.add_track_to_favorites(&track_id);
                    player_bus.publish_message(playerbus::Message::TrackAddedToFavorites);
                },
                _ => {},
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    });
}

fn discovery_module(discovery_store: DiscoveryStore) {
    thread::spawn(move || {
        let _ = discovery_store.discover_mixes();
        let _ = discovery_store.discover_favorities_tracks();
    });
}

fn downloader_module(session: Session, playlist: Playlist) {
    thread::spawn(move || {
        playlist.buffer_worker(|track| {
            match downloader::download_file(track, &session) {
                Ok(buffered_track) => buffered_track,
                Err(err) => panic!("{:?}", err),
            }
        });
    });
}

fn server_module(player_bus: PlayerBus) {
    thread::Builder::new()
        .name("Server module".to_owned())
        .spawn_with_priority(ThreadPriority::Min, |_| {
            http::server(player_bus);
    }).unwrap();
}

fn player_module(playlist: Playlist, player_bus: PlayerBus) {
    thread::Builder::new()
        .name("Player module".to_owned())
        .spawn_with_priority(ThreadPriority::Max, |_| {
            let _ = player::player(playlist, player_bus);
    }).unwrap();
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

    let config_path = home::home_dir().unwrap().join("config.ini");
    let session = SessionGui::init(config_path).gui_loop().await;

    let playlist = Playlist::new();
    let player_bus = PlayerBus::new();
    let discovery_store = DiscoveryStore::new(session.clone(), playlist.clone());
    
    discovery_module(discovery_store.clone());
    service_module(discovery_store.clone(), player_bus.clone(), session.clone());
    server_module(player_bus.clone());
    downloader_module(session.clone(), playlist.clone());
    player_module(playlist.clone(), player_bus.clone());

    Gui::init(player_bus.clone())
        .gui_loop()
        .await;
}