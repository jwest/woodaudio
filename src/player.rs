use rodio::{OutputStream, Decoder, Sink};
use std::{io::Cursor, thread, time::{Duration, Instant}};
use log::error;

use crate::{playerbus::{self, PlayerBus, PlayerBusAction, PlayerTrackState}, playlist::{BufferedTrack, Playlist}};

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
    let (_stream, stream_handle) = retry(OutputStream::try_default);
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    sink.play();

    loop {
        match player_bus.read() {
            PlayerBusAction::PausePlay => {
                if sink.is_paused() {
                    sink.play();
                } else {
                    sink.pause();
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
                            let playing_track = track.track;
                            player_bus.set_state(PlayerTrackState {
                                player_state: playerbus::PlayerState::Playing,
                                id: playing_track.id,
                                title: playing_track.title,
                                artist_name: playing_track.artist_name,
                                album_name: playing_track.album_name,
                                cover: track.cover.as_ref().map(|c| c.foreground.to_string()),
                                cover_background: track.cover.as_ref().map(|c| c.background.to_string()),
                                duration: playing_track.duration,
                                playing_time: Instant::now(),
                            });
                            sink.append(source.unwrap());
                            sink.play();
                        }
                    }
                    None => {
                        player_bus.set_state(PlayerTrackState::default_state());
                        thread::sleep(Duration::from_millis(200));
                    }
                }
            },
            false => thread::sleep(Duration::from_millis(200)),
        }
    }
}