use std::fs::File;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{debug, error, info};
use reqwest::blocking::Client;
use ringbuf::HeapRb;

use crate::playlist::BufferedTrack;
use super::Player;
use super::symphonia_decoder;

/// 2 seconds of audio at 48kHz stereo i16
const RING_BUFFER_SAMPLES: usize = 48_000 * 2 * 2;

pub struct CpalPlayer {
    empty: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
    http_client: Arc<Client>,
}

impl CpalPlayer {
    pub fn new() -> Self {
        let http_client = Arc::new(
            Client::builder()
                .timeout(Duration::from_secs(600))
                .build()
                .expect("Failed to build HTTP client"),
        );
        Self {
            empty: Arc::new(AtomicBool::new(true)),
            paused: Arc::new(AtomicBool::new(false)),
            stopped: Arc::new(AtomicBool::new(false)),
            worker: None,
            http_client,
        }
    }

    fn stop_worker(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Some(handle) = self.worker.take() {
            let _ = handle.join();
        }
    }
}

fn open_stream(client: &Client, url: &str) -> Result<Box<dyn Read + Send + Sync + 'static>, Box<dyn std::error::Error>> {
    if let Some(path) = url.strip_prefix("file://") {
        let file = File::open(path)?;
        Ok(Box::new(BufReader::with_capacity(256 * 1024, file)))
    } else {
        let response = client.get(url).send()?;
        Ok(Box::new(response))
    }
}

impl Player for CpalPlayer {
    fn play_track(&mut self, track: BufferedTrack) -> bool {
        match &self.worker {
            Some(handle) if !handle.is_finished() => self.stop_worker(),
            Some(_) => { self.worker.take(); }
            None => {}
        }

        self.stopped.store(false, Ordering::SeqCst);
        self.paused.store(false, Ordering::SeqCst);
        self.empty.store(false, Ordering::SeqCst);

        let stream_url = track.stream_url.clone();
        let empty = Arc::clone(&self.empty);
        let paused = Arc::clone(&self.paused);
        let stopped = Arc::clone(&self.stopped);
        let http_client = Arc::clone(&self.http_client);

        let handle = thread::spawn(move || {
            info!("[CpalPlayer] Opening stream: {}", stream_url);

            let reader = match open_stream(&http_client, &stream_url) {
                Ok(r) => r,
                Err(e) => {
                    error!("[CpalPlayer] Failed to open stream: {}", e);
                    empty.store(true, Ordering::SeqCst);
                    return;
                }
            };

            let host = cpal::default_host();
            let device = match host.default_output_device() {
                Some(d) => d,
                None => {
                    error!("[CpalPlayer] No output device");
                    empty.store(true, Ordering::SeqCst);
                    return;
                }
            };

            let supported = match device.default_output_config() {
                Ok(c) => c,
                Err(e) => {
                    error!("[CpalPlayer] No output config: {}", e);
                    empty.store(true, Ordering::SeqCst);
                    return;
                }
            };

            info!("[CpalPlayer] Output: {} {}Hz {}ch",
                device.name().unwrap_or_default(),
                supported.sample_rate().0,
                supported.channels());

            let ring = HeapRb::<i16>::new(RING_BUFFER_SAMPLES);
            let (mut producer, mut consumer) = ring.split();

            let paused_cb = Arc::clone(&paused);
            let stopped_cb = Arc::clone(&stopped);

            let stream = match device.build_output_stream(
                &supported.config(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    if stopped_cb.load(Ordering::Relaxed) || paused_cb.load(Ordering::Relaxed) {
                        for s in data.iter_mut() { *s = 0.0; }
                        return;
                    }
                    for sample in data.iter_mut() {
                        let s = consumer.pop().unwrap_or(0i16);
                        *sample = s as f32 / i16::MAX as f32;
                    }
                },
                |err| error!("[CpalPlayer] Stream error: {}", err),
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    error!("[CpalPlayer] Build stream failed: {}", e);
                    empty.store(true, Ordering::SeqCst);
                    return;
                }
            };

            if let Err(e) = stream.play() {
                error!("[CpalPlayer] Play failed: {}", e);
                empty.store(true, Ordering::SeqCst);
                return;
            }

            let result = symphonia_decoder::decode_streaming(reader, |samples| {
                if stopped.load(Ordering::SeqCst) {
                    return false;
                }
                while paused.load(Ordering::SeqCst) && !stopped.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(10));
                }
                if stopped.load(Ordering::SeqCst) {
                    return false;
                }
                let mut offset = 0;
                while offset < samples.len() {
                    if stopped.load(Ordering::Relaxed) {
                        return false;
                    }
                    match producer.push(samples[offset]) {
                        Ok(_) => offset += 1,
                        Err(_) => thread::sleep(Duration::from_millis(1)),
                    }
                }
                true
            });

            // Drain remaining samples
            while producer.len() > 0 && !stopped.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(50));
            }

            drop(stream);

            match result {
                Ok(info) => info!("[CpalPlayer] Done, {}Hz {}ch", info.sample_rate, info.channels),
                Err(e) => error!("[CpalPlayer] Decode error: {}", e),
            }

            empty.store(true, Ordering::SeqCst);
            debug!("[CpalPlayer] Worker done");
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
        self.stop_worker();
        self.empty.store(true, Ordering::SeqCst);
    }

    fn is_empty(&self) -> bool {
        self.empty.load(Ordering::SeqCst)
    }

    fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }
}
