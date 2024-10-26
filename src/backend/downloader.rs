use std::{error::Error, fs, sync::{Arc, Mutex}};

use log::{debug, error, info};
use secular::normalized_lower_lay_string;

use crate::{backend::cover::CoverProcessor, config::Config, playlist::{BufferedTrack, Cover, Track}};
use crate::backend::storage::FileStorage;
use super::{storage::{CacheRead, Exporter, FtpStorage}, tidal::TidalBackend, Backend};

#[derive(Clone)]
pub struct Downloader {
    storage_file: Arc<Mutex<Option<FileStorage>>>,
    storage_ftp: Arc<Mutex<Option<FtpStorage>>>,
    display_cover_background: bool,
    display_cover_foreground: bool,
    backend: TidalBackend,
}

impl Track {
    fn file_name(&self) -> String {
        normalized_lower_lay_string(format!("{} - {}.flac", self.artist_name, self.title).as_str())
    }
}

impl Downloader {
    pub fn init(config: &Config, backend: TidalBackend) -> Self {
        let storage_file = match config.exporter_file.enabled {
            true => Some(FileStorage::init(config.exporter_file.clone())),
            false => None,
        };

        let storage_ftp = match config.exporter_ftp.enabled {
            true => Some(FtpStorage::init(config.exporter_ftp.clone())),
            false => None,
        };

        Downloader {
            storage_file: Arc::new(Mutex::new(storage_file)),
            storage_ftp: Arc::new(Mutex::new(storage_ftp)),
            display_cover_background: config.gui.display_cover_background, 
            display_cover_foreground: config.gui.display_cover_foreground,
            backend,
        }
    }

    pub fn download_file(&mut self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        if let Some(storage_file) = self.storage_file.lock().unwrap().as_mut() {
            match storage_file.read_file(&track.file_name(), None) {
                Ok(Some(file)) => {
                    info!("[Storage] cache exists {:?}", track);
                    return Ok(BufferedTrack {
                        track: track.clone(),
                        stream: file.clone(),
                        cover: self.download_album_cover(track.album_image).unwrap_or_else(|_| Cover::empty()),
                    })
                },
                _ => {
                    info!("[Storage] cache empty or error for {:?}", track);
                },
            }
        }
        if let Some(ftp_storage) = self.storage_ftp.lock().unwrap().as_mut() {
            match ftp_storage.read_file(&track.file_name(), None) {
                Ok(Some(file)) => {
                    info!("[Storage] cache exists {:?}", track);
                    return Ok(BufferedTrack {
                        track: track.clone(),
                        stream: file.clone(),
                        cover: self.download_album_cover(track.album_image).unwrap_or_else(|_| Cover::empty()),
                    })
                },
                _ => {
                    info!("[Storage] cache empty or error for {:?}", track);
                },
            }
        }
        for _ in 1..5 {
            let bytes_response = self.backend.get_track(track.id.clone())?;

            let cover = self.download_album_cover(track.album_image.clone()).unwrap_or_else(|_| Cover::empty());

            if let Some(storage_file) = self.storage_file.lock().unwrap().as_mut() {
                let export_bytes = bytes_response.clone();
                let cover_image = if let Some(image_url) = cover.clone().foreground { Some(fs::read(image_url)?) } else { None };
                match storage_file.write_file(track.clone(), export_bytes, &track.file_name(), None, cover_image) {
                    Ok(()) => {
                        info!("[Storage File] cache file wrote, track: {:?}", track);
                    },
                    Err(err) => {
                        error!("[Storage File] cache file wrote error, track: {:?}, error: {:?}", track, err);
                    },
                }
            }

            if let Some(storage_ftp) = self.storage_ftp.lock().unwrap().as_mut() {
                let export_bytes = bytes_response.clone();
                match storage_ftp.write_file(track.clone(), export_bytes, &track.file_name(), None, None) {
                    Ok(()) => {
                        info!("[Storage FTP] cache file wrote, track: {:?}", track);
                    },
                    Err(err) => {
                        error!("[Storage FTP] cache file wrote error, track: {:?}, error: {:?}", track, err);
                    },
                }
            }

            return Ok(BufferedTrack {
                track: track.clone(),
                stream: bytes_response,
                cover,
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