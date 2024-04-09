use rodio::{OutputStream, Decoder, Sink};
use std::{io::{BufReader, Cursor}, thread, time::{Duration, Instant}};
use log::{debug, error};

use crate::{playerbus::{Command, Message, PlayerBus}, playlist::{BufferedTrack, Playlist}};

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

fn source(track: BufferedTrack) -> Option<Decoder<BufReader<std::io::Cursor<bytes::Bytes>>>> {
    // let source_result = Decoder::new_flac(BufReader::with_capacity(4_194_304, Cursor::new(track.stream)));
    // let source_result = Decoder::new_vorbis(BufReader::with_capacity(4_194_304, Cursor::new(track.stream)));
    let source_result = Decoder::new(BufReader::with_capacity(4_194_304, Cursor::new(track.stream)));
    match source_result {
        Ok(file) => Some(file),
        Err(err) => {
            error!("[Player] Audio file '{:?}' decode error, try next...", err);
            None
        },
    }
}

pub fn player(playlist: &Playlist, mut player_bus: PlayerBus) {
    let command_channel = player_bus.register_command_channel(vec![Command::Play.as_string(), Command::Pause.as_string(), Command::Next.as_string()]);

    let (_stream, stream_handle) = retry(OutputStream::try_default);
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    let mut playing_time: Option<Duration> = None;
    let mut last_iteration_datetime = Instant::now();

    sink.play();

    loop {
        if sink.empty() {
            if let Some(track) = playlist.pop() {
                let source = source(track.clone());
                if source.is_some() {
                    playing_time = Some(Duration::ZERO);
                    player_bus.publish_message(Message::PlayerPlayingNewTrack(track));
                    
                    sink.append(source.unwrap());
                    sink.play();
                }
            } else {
                playing_time = None;
                player_bus.publish_message(Message::PlayerQueueIsEmpty);
                
                thread::sleep(Duration::from_millis(200));
            }
        } else {
            match command_channel.read_command() {
                Some(Command::Play) => {
                    sink.play();
                    player_bus.publish_message(Message::PlayerPlaying);
                },
                Some(Command::Pause) => {
                    sink.pause();
                    player_bus.publish_message(Message::PlayerToPause);
                },
                Some(Command::Next) => {
                    sink.clear();
                },
                _ => {},
            };

            thread::sleep(Duration::from_millis(50));

            if !sink.is_paused() {
                debug!("[Player] playing time: {:?}, ({:?})", playing_time, last_iteration_datetime);
                player_bus.publish_message(Message::PlayerElapsed(playing_time.unwrap_or(Duration::ZERO)));
                playing_time = Some(playing_time.unwrap_or(Duration::ZERO) + last_iteration_datetime.elapsed());
            }
        }
        last_iteration_datetime = Instant::now();
    }
}