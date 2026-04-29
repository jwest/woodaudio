use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use log::{error, info};

use crate::playlist::BufferedTrack;
use super::Player;
use super::symphonia_decoder;

pub struct SnapcastPlayer {
    pipe_path: String,
    empty: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
}

impl SnapcastPlayer {
    pub fn new(pipe_path: &str) -> Self {
        let path = Path::new(pipe_path);
        if !path.exists() {
            info!("[SnapcastPlayer] Creating FIFO at {}", pipe_path);
            unsafe {
                let c_path = std::ffi::CString::new(pipe_path).unwrap();
                libc::mkfifo(c_path.as_ptr(), 0o644);
            }
        }

        Self {
            pipe_path: pipe_path.to_string(),
            empty: Arc::new(AtomicBool::new(true)),
            paused: Arc::new(AtomicBool::new(false)),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn write_pcm_to_pipe(pipe_path: &str, samples: &[i16], paused: &AtomicBool, stopped: &AtomicBool) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(pipe_path)
        .map_err(|e| format!("Failed to open FIFO {}: {:?}", pipe_path, e))?;

    let byte_slice = unsafe {
        std::slice::from_raw_parts(
            samples.as_ptr() as *const u8,
            samples.len() * 2,
        )
    };

    // Write in chunks to allow checking paused/stopped between writes
    let chunk_size = 8192;
    for chunk in byte_slice.chunks(chunk_size) {
        if stopped.load(Ordering::SeqCst) {
            return Ok(());
        }

        while paused.load(Ordering::SeqCst) && !stopped.load(Ordering::SeqCst) {
            thread::sleep(std::time::Duration::from_millis(50));
        }

        if stopped.load(Ordering::SeqCst) {
            return Ok(());
        }

        file.write_all(chunk)
            .map_err(|e| format!("Failed to write to FIFO: {:?}", e))?;
    }

    Ok(())
}

impl Player for SnapcastPlayer {
    fn play_track(&mut self, track: BufferedTrack) -> bool {
        self.stopped.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);

        let decoded = match symphonia_decoder::decode(track.stream) {
            Ok(decoded) => {
                info!("[SnapcastPlayer] Decoded track: {} samples, {}Hz, {}ch",
                    decoded.samples.len(), decoded.sample_rate, decoded.channels);
                decoded
            },
            Err(e) => {
                error!("[SnapcastPlayer] Decode error: {}", e);
                return false;
            }
        };

        self.empty.store(false, Ordering::SeqCst);

        let empty = Arc::clone(&self.empty);
        let paused = Arc::clone(&self.paused);
        let stopped = Arc::clone(&self.stopped);
        let pipe_path = self.pipe_path.clone();

        thread::spawn(move || {
            match write_pcm_to_pipe(&pipe_path, &decoded.samples, &paused, &stopped) {
                Ok(_) => info!("[SnapcastPlayer] Track written to pipe successfully"),
                Err(e) => error!("[SnapcastPlayer] Write error: {}", e),
            }
            empty.store(true, Ordering::SeqCst);
        });

        true
    }

    fn pause(&mut self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    fn resume(&mut self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    fn stop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.empty.store(true, Ordering::SeqCst);
    }

    fn is_empty(&self) -> bool {
        self.empty.load(Ordering::SeqCst)
    }

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}
