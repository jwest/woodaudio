use std::{error::Error, sync::{Arc, Mutex}};

use log::{debug, error, info};
use secular::normalized_lower_lay_string;

use crate::{backend::cover::CoverProcessor, config::Config, playlist::{BufferedTrack, Cover, Track}};
use super::{spotify::SpotifyBackend, storage::{CacheRead, Exporter, FtpStorage}, tidal::TidalBackend, Backend};

#[derive(Clone)]
pub struct Downloader {
    storage: Arc<Mutex<Option<FtpStorage>>>,
    display_cover_background: bool,
    display_cover_foreground: bool,
    // backend: TidalBackend,
    backend: SpotifyBackend,
}

impl Track {
    fn file_name(&self) -> String {
        normalized_lower_lay_string(format!("{} - {}.flac", self.artist_name, self.title).as_str())
    }
}

impl Downloader {
    pub fn init(config: &Config, backend: SpotifyBackend) -> Self {
        let storage = match config.exporter_ftp.enabled {
            true => Some(FtpStorage::init(config.exporter_ftp.clone())),
            false => None,
        };

        Downloader { 
            storage: Arc::new(Mutex::new(storage)),
            display_cover_background: config.gui.display_cover_background, 
            display_cover_foreground: config.gui.display_cover_foreground,
            backend,
        }
    }
    
    pub fn download_file(&self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        if let Some(ftp_storage) = self.storage.lock().unwrap().as_mut() {
            match ftp_storage.read_file(&track.file_name(), None) {
                Ok(Some(file)) => {
                    info!("[Storage] cache exists {:?}", track);
                    return Ok(BufferedTrack {
                        track: track.clone(),
                        stream: file.clone(),
                        cover: match self.download_album_cover(track.album_image) {
                            Ok(cover) => cover,
                            Err(_) => Cover::empty(),
                        },
                    })
                },
                _ => {
                    info!("[Storage] cache empty or error for {:?}", track);
                },
            }
        }
        for _ in 1..5 {
            let bytes_response = self.backend.get_track(track.id.clone())?;
    
            if let Some(ftp_storage) = self.storage.lock().unwrap().as_mut() {
                let export_bytes = bytes_response.clone();
                match ftp_storage.write_file(export_bytes, &track.file_name(), None) {
                    Ok(()) => {
                        info!("[Storage] cache file wrote, track: {:?}", track);
                    },
                    Err(err) => {
                        error!("[Storage] cache file wrote error, track: {:?}, error: {:?}", track, err);
                    },
                }
            }

            return Ok(BufferedTrack {
                track: track.clone(),
                stream: bytes_response,
                cover: match self.download_album_cover(track.album_image) {
                    Ok(cover) => cover,
                    Err(_) => Cover::empty(),
                },
            })
        }
    
        Err("Track Download fail!".into())
    }

    fn download_album_cover(&self, cover_url: String) -> Result<Cover, Box<dyn Error>> {
        if !self.display_cover_background && !self.display_cover_foreground {
            return Ok(Cover::empty());
        }

        debug!("[Downloader] Prepare cover '{}'...", cover_url);
    
        let bytes_response = self.backend.get_cover(cover_url.clone())?;
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