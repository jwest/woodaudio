use std::{error::Error, sync::{Arc, Mutex}};

use log::{debug, error, info};

use crate::{backend::cover::CoverProcessor, config::Config, playlist::{BufferedTrack, Cover, Track}};
use super::{tidal::TidalBackend, Backend};

#[derive(Clone)]
pub struct Downloader {
    display_cover_background: bool,
    display_cover_foreground: bool,
    backend: Arc<Mutex<TidalBackend>>,
}

impl Downloader {
    pub fn init(config: &Config, backend: TidalBackend) -> Self {
        Downloader {
            display_cover_background: config.gui.display_cover_background,
            display_cover_foreground: config.gui.display_cover_foreground,
            backend: Arc::new(Mutex::new(backend)),
        }
    }

    pub fn download_file(&mut self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        let stream_url = self.backend.lock().unwrap().get_track_url(track.id.clone())?;
        info!("[Downloader] Got stream URL for {:?}", track);

        let cover = self.download_album_cover(track.album_image.clone()).unwrap_or_else(|e| {
            error!("[Downloader] Cover error: {}", e);
            Cover::empty()
        });

        Ok(BufferedTrack {
            track,
            stream_url,
            cover,
        })
    }

    fn download_album_cover(&self, cover_url: String) -> Result<Cover, Box<dyn Error>> {
        if !self.display_cover_background && !self.display_cover_foreground {
            return Ok(Cover::empty());
        }

        debug!("[Downloader] Prepare cover '{}'...", cover_url);

        let bytes_response = self.backend.lock().unwrap().get_cover(cover_url.clone())?;
        let cover = CoverProcessor::new(bytes_response);

        let foreground = if self.display_cover_foreground {
            Some(cover.generate_foreground()?.to_string_lossy().to_string())
        } else {
            None
        };

        let background = if self.display_cover_background {
            Some(cover.generate_background()?.to_string_lossy().to_string())
        } else {
            None
        };

        debug!("[Downloader] Cover prepared '{}', foreground: {:?}, background: {:?}", cover_url, foreground, background);

        Ok(Cover {
            foreground,
            background,
        })
    }
}
