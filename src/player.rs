use rodio::{OutputStream, Decoder, Sink};
use std::{io::Cursor, thread, time::{Duration, Instant}};
use log::{debug, error, info};

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
    
    let mut playing_time: Option<Duration> = None;
    let mut last_iteration_datetime = Instant::now();

    sink.play();

    loop {
        match sink.empty() {
            true => {
                match playlist.pop() {
                    Some(track) => {  
                        let source = source(track.clone());
                        if source.is_some() {
                            playing_time = Some(Duration::ZERO);
                            state = State {
                                player: playerbus::PlayerState { case: playerbus::PlayerStateCase::Playing, playing_time },
                                track: Some(TrackState::from(track)),
                            }.publish(&player_bus);

                            sink.append(source.unwrap());
                            sink.play();
                        }
                    }
                    None => {
                        playing_time = None;
                        state = State {
                            player: playerbus::PlayerState { case: playerbus::PlayerStateCase::Loading, playing_time },
                            track: None,
                        }.publish(&player_bus);

                        thread::sleep(Duration::from_millis(200));
                    }
                }
            },
            false => {
                match player_bus.read() {
                    PlayerBusAction::PausePlay => {
                        if sink.is_paused() {
                            sink.play();
                            state = state.build_with_change_player(PlayerState { case: playerbus::PlayerStateCase::Playing, playing_time }).publish(&player_bus);
                        } else {
                            sink.pause();
                            state = state.build_with_change_player(PlayerState { case: playerbus::PlayerStateCase::Paused, playing_time }).publish(&player_bus);
                        }
                    },
                    PlayerBusAction::NextSong => {
                        sink.clear();
                    },
                    _ => {},
                };

                thread::sleep(Duration::from_millis(50));

                if !sink.is_paused() {
                    debug!("[Player] playing time: {:?}, ({:?})", playing_time, last_iteration_datetime);
                    state = state.build_with_change_player(PlayerState { case: playerbus::PlayerStateCase::Playing, playing_time }).publish(&player_bus);
                    playing_time = Some(playing_time.unwrap_or(Duration::ZERO) + (Instant::now() - last_iteration_datetime));
                }
            },
        }
        last_iteration_datetime = Instant::now();
    }
}