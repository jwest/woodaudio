use std::{thread, time::{Duration, Instant}};
use log::info;

use crate::{config::Config, state::{Command, Message, PlayerBus}, playlist::Playlist};
use crate::playlist::BufferedTrack;

mod rodio;
mod snapcast;
mod symphonia_decoder;

pub trait Player {
    fn play_track(&mut self, track: BufferedTrack) -> bool;
    fn pause(&mut self);
    fn resume(&mut self);
    fn stop(&mut self);
    fn is_empty(&self) -> bool;
    fn is_paused(&self) -> bool;
}

fn create_player(config: &Config) -> Box<dyn Player> {
    match config.player.output.as_str() {
        "snapcast" => {
            info!("[Player] Using SnapcastPlayer output, pipe: {}", config.player.snapcast_pipe);
            Box::new(snapcast::SnapcastPlayer::new(&config.player.snapcast_pipe))
        },
        _ => {
            info!("[Player] Using RodioPlayer output");
            Box::new(rodio::RodioPlayer::new())
        },
    }
}

pub fn player(config: &Config, playlist: &Playlist, mut player_bus: PlayerBus) {
    let command_channel = player_bus.register_command_channel(vec![Command::Play.as_string(), Command::Pause.as_string(), Command::Next.as_string()]);

    let mut backend = create_player(config);

    let mut playing_time: Option<Duration> = None;
    let mut last_iteration_datetime = Instant::now();

    loop {
        if backend.is_empty() {
            if let Some(track) = playlist.pop() {
                if backend.play_track(track.clone()) {
                    playing_time = Some(Duration::ZERO);
                    player_bus.publish_message(Message::PlayerPlayingNewTrack(track));
                }
            } else {
                playing_time = None;
                player_bus.publish_message(Message::PlayerQueueIsEmpty);
                thread::sleep(Duration::from_millis(200));
            }
        } else {
            match command_channel.read_command() {
                Some(Command::Play) => {
                    backend.resume();
                    player_bus.publish_message(Message::PlayerPlaying);
                },
                Some(Command::Pause) => {
                    backend.pause();
                    player_bus.publish_message(Message::PlayerToPause);
                },
                Some(Command::Next) => {
                    backend.stop();
                },
                _ => {},
            };

            thread::sleep(Duration::from_millis(50));

            if !backend.is_paused() {
                player_bus.publish_message(Message::PlayerElapsed(playing_time.unwrap_or(Duration::ZERO)));
                playing_time = Some(playing_time.unwrap_or(Duration::ZERO) + last_iteration_datetime.elapsed());
            }
        }
        last_iteration_datetime = Instant::now();
    }
}
