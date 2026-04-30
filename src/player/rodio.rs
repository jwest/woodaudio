use rodio::{OutputStream, Decoder, Sink, Source};
use std::io::{BufReader, Cursor};
use std::thread;
use std::time::Duration;
use log::{debug, error};

use crate::playlist::BufferedTrack;
use super::Player;

fn retry<T, E>(function: fn() -> Result<T, E>) -> T where E: std::fmt::Display {
    match function() {
        Ok(output) => output,
        Err(err) => {
            error!("[RodioPlayer] Load audio output fail, retry... ({:?})", err.to_string());
            thread::sleep(Duration::from_secs(3));
            retry(function)
        },
    }
}

pub struct RodioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

impl RodioPlayer {
    pub fn new() -> Self {
        let (_stream, stream_handle) = retry(OutputStream::try_default);
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.play();
        Self { _stream, sink }
    }
}

impl Player for RodioPlayer {
    fn play_track(&mut self, track: BufferedTrack) -> bool {
        let source_result = Decoder::new_flac(BufReader::with_capacity(4_194_304, Cursor::new(track.stream)));

        match source_result {
            Ok(file) => {
                debug!("[RodioPlayer] Track {:?}, channels: {:?}, sample rate: {:?}, duration: {:?}",
                    track.track, file.channels(), file.sample_rate(), file.total_duration());
                self.sink.append(file);
                self.sink.play();
                true
            },
            Err(err) => {
                error!("[RodioPlayer] Audio file '{:?}' decode error, try next...", err);
                false
            },
        }
    }

    fn pause(&mut self) {
        self.sink.pause();
    }

    fn resume(&mut self) {
        self.sink.play();
    }

    fn stop(&mut self) {
        self.sink.clear();
    }

    fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
}
