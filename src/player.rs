use rodio::{OutputStream, Decoder, Sink};
use std::{io::Cursor, thread, time::{Duration, Instant}};
use log::error;

use crate::{playerbus::{self, PlayerBus, PlayerBusAction, PlayerState, State, TrackState}, playlist::{BufferedTrack, Playlist}};

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

pub fn player(playlist: Playlist, player_bus: PlayerBus) {
    let mut state = State::default_state();

    let (_stream, stream_handle) = retry(OutputStream::try_default);
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    sink.play();

    loop {
        match player_bus.read() {
            PlayerBusAction::PausePlay => {
                if sink.is_paused() {
                    sink.play();
                    state = state.build_with_change_player(PlayerState { case: playerbus::PlayerStateCase::Playing, playing_time: state.player.playing_time }).publish(&player_bus);
                } else {
                    sink.pause();
                    state = state.build_with_change_player(PlayerState { case: playerbus::PlayerStateCase::Paused, playing_time: state.player.playing_time }).publish(&player_bus);
                }
            },
            PlayerBusAction::NextSong => {
                sink.clear();
            },
            _ => {},
        };

        match sink.empty() {
            true => {
                match playlist.pop() {
                    Some(track) => {  
                        let source = source(track.clone());
                        if source.is_some() {
                            state = State {
                                player: playerbus::PlayerState { case: playerbus::PlayerStateCase::Playing, playing_time: Some(Instant::now()) },
                                track: Some(TrackState::from(track)),
                            }.publish(&player_bus);

                            sink.append(source.unwrap());
                            sink.play();
                        }
                    }
                    None => {
                        state = State {
                            player: playerbus::PlayerState { case: playerbus::PlayerStateCase::Loading, playing_time: None },
                            track: None,
                        }.publish(&player_bus);

                        thread::sleep(Duration::from_millis(200));
                    }
                }
            },
            false => thread::sleep(Duration::from_millis(200)),
        }
    }
}