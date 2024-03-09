use log::info;
use serde_json::Value;
use tiny_http::{Response, Server};

use crate::{discovery::DiscoveryStore, playerbus::{self, PlayerBus, PlayerBusAction}};

pub fn server(discovery_store: DiscoveryStore, player_bus: PlayerBus) {
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
                        player_bus.call(playerbus::PlayerBusAction::Waiting);
                        let _ = discovery_store.discovery_track(id);
                        player_bus.call(PlayerBusAction::NextSong);
                    }

                    if tidal_url.starts_with("https://tidal.com/album/") {
                        player_bus.call(playerbus::PlayerBusAction::Waiting);
                        let _ = discovery_store.discovery_album(id);
                        player_bus.call(PlayerBusAction::NextSong);
                    }

                    if tidal_url.starts_with("https://tidal.com/artist/") {
                        player_bus.call(playerbus::PlayerBusAction::Waiting);
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
}