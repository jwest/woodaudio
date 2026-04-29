use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::{error, info, warn};

use crate::playlist::BufferedTrack;
use super::Player;
use super::symphonia_decoder;

pub struct SnapcastPlayer {
    host: String,
    port: u16,
    stream: Arc<Mutex<Option<TcpStream>>>,
    empty: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl SnapcastPlayer {
    pub fn new(host: &str, port: u16) -> Self {
        let stream = Self::connect(host, port);

        Self {
            host: host.to_string(),
            port,
            stream: Arc::new(Mutex::new(Some(stream))),
            empty: Arc::new(AtomicBool::new(true)),
            paused: Arc::new(AtomicBool::new(false)),
            stopped: Arc::new(AtomicBool::new(false)),
            worker: None,
        }
    }

    fn connect(host: &str, port: u16) -> TcpStream {
        let addr = format!("{}:{}", host, port);
        info!("[SnapcastPlayer] Connecting to snapcast TCP at {} ...", addr);

        loop {
            match TcpStream::connect(&addr) {
                Ok(s) => {
                    s.set_nodelay(true).unwrap_or_else(|e| warn!("[SnapcastPlayer] Failed to set TCP_NODELAY: {:?}", e));
                    info!("[SnapcastPlayer] Connected to snapcast TCP at {}", addr);
                    return s;
                }
                Err(e) => {
                    error!("[SnapcastPlayer] Failed to connect to {}, retrying in 3s... ({:?})", addr, e);
                    thread::sleep(Duration::from_secs(3));
                }
            }
        }
    }

    fn stop_and_wait(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);

        // Shutdown the TCP stream to unblock any write_all in the worker thread
        if let Some(stream) = self.stream.lock().unwrap().take() {
            let _ = stream.shutdown(Shutdown::Write);
        }

        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

impl Player for SnapcastPlayer {
    fn play_track(&mut self, track: BufferedTrack) -> bool {
        match &self.worker {
            Some(handle) if !handle.is_finished() => {
                self.stop_and_wait();
            },
            Some(_) => {
                self.worker.take();
            },
            None => {},
        };

        // Reconnect if stream was closed (after stop or shutdown)
        if self.stream.lock().unwrap().is_none() {
            let new_stream = Self::connect(&self.host, self.port);
            *self.stream.lock().unwrap() = Some(new_stream);
        }

        self.stopped.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        self.empty.store(false, Ordering::SeqCst);

        let empty = Arc::clone(&self.empty);
        let paused = Arc::clone(&self.paused);
        let stopped = Arc::clone(&self.stopped);
        let stream = Arc::clone(&self.stream);

        let handle = thread::spawn(move || {
            info!("[SnapcastPlayer] Starting decode_streaming...");
            let mut total_bytes: usize = 0;

            let result = symphonia_decoder::decode_streaming(track.stream, |samples| {
                if stopped.load(Ordering::SeqCst) {
                    return false;
                }

                while paused.load(Ordering::SeqCst) && !stopped.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(50));
                }

                if stopped.load(Ordering::SeqCst) {
                    return false;
                }

                let byte_slice = unsafe {
                    std::slice::from_raw_parts(
                        samples.as_ptr() as *const u8,
                        samples.len() * 2,
                    )
                };

                let mut guard = stream.lock().unwrap();
                let tcp = match guard.as_mut() {
                    Some(tcp) => tcp,
                    None => return false,
                };

                match tcp.write_all(byte_slice) {
                    Ok(_) => {
                        total_bytes += byte_slice.len();
                        if total_bytes % (1024 * 1024) < byte_slice.len() {
                            info!("[SnapcastPlayer] Written {} MB so far", total_bytes / (1024 * 1024));
                        }
                        true
                    },
                    Err(e) => {
                        error!("[SnapcastPlayer] Write error: {:?}", e);
                        false
                    }
                }
            });

            match result {
                Ok(info) => info!("[SnapcastPlayer] Track finished, {}Hz {}ch, total {} bytes", info.sample_rate, info.channels, total_bytes),
                Err(e) => error!("[SnapcastPlayer] Decode error: {}", e),
            }

            empty.store(true, Ordering::SeqCst);
        });

        self.worker = Some(handle);
        true
    }

    fn pause(&mut self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    fn resume(&mut self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    fn stop(&mut self) {
        self.stop_and_wait();
        self.empty.store(true, Ordering::SeqCst);
    }

    fn is_empty(&self) -> bool {
        self.empty.load(Ordering::SeqCst)
    }

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}
