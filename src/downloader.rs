use std::{error::Error, io::{Cursor, Read}, sync::{Arc, Mutex}, time::Duration};

use bytes::Buf;
use image::io::Reader as ImageReader;
use log::{debug, error, info};
use reqwest::blocking::Client;
use secular::normalized_lower_lay_string;
use suppaftp::{types::FileType, FtpStream};
use tempfile::NamedTempFile;

use crate::{config::{Config, ExporterFTP}, playlist::{BufferedTrack, Cover, Track}, session::Session};

trait CacheRead {
    fn read_file(&mut self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<bytes::Bytes>, Box<dyn Error>>;
}

trait Exporter {
    fn write_file(&mut self, source: bytes::Bytes, output_file_name: &str, output_dir: Option<&str>) -> Result<(), Box<dyn Error>>;
}

struct FtpStorage {
    client: FtpStream,
    cache_read: bool,
    output_path: String,
}

impl FtpStorage {
    fn init(config: ExporterFTP) -> Self {
        let mut client = FtpStream::connect(config.server).unwrap();
        client.login(config.username, config.password).unwrap();
        client.transfer_type(FileType::Binary).unwrap();
        client.set_mode(suppaftp::Mode::ExtendedPassive);

        Self { client, cache_read: config.cache_read, output_path: config.share }
    }
    fn file_name_with_create_dir(&mut self, output_file_name: &str, output_dir: Option<&str>) -> Result<String, Box<dyn Error>> {
        if let Some(dir) = output_dir {
            self.client.mkdir(dir)?;
            Ok(format!("/{dir}/{output_file_name}"))
        } else { 
            Ok(format!("{}{}", self.output_path, output_file_name))
        }
    }
}

impl Exporter for FtpStorage {
    fn write_file(&mut self, source: bytes::Bytes, output_file_name: &str, output_dir: Option<&str>) -> Result<(), Box<dyn Error>> {
        let file_name = self.file_name_with_create_dir(output_file_name, output_dir)?;

        self.client.put_file(
            file_name.clone(),
            &mut source.reader()
        )?;

        info!("[Exporter] file saved on ftp: {:?}", file_name);
        Ok(())
    }
}

impl CacheRead for FtpStorage {
    fn read_file(&mut self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<bytes::Bytes>, Box<dyn Error>> {
        if !self.cache_read {
            return Ok(None);
        }

        let file_name = self.file_name_with_create_dir(output_file_name, output_dir)?;
        let mut reader = self.client.retr_as_stream(file_name.clone())?;

        let mut output = bytes::BytesMut::new();
        reader.read_exact(&mut output)?;
        drop(reader);

        if output.is_empty() {
            return Ok(None);
        }

        info!("[Cache] file readed from ftp: {:?} {:?}", file_name, output);

        Ok(Some(output.into()))
    }
}

pub struct Downloader {
    storage: Arc<Mutex<Option<FtpStorage>>>,
    session: Session,
    display_cover_background: bool,
    display_cover_foreground: bool,
}

impl Track {
    fn file_name(&self) -> String {
        normalized_lower_lay_string(format!("{} - {}.flac", self.artist_name, self.title).as_str())
    }
}

impl Downloader {
    pub fn init(session: Session, config: &Config) -> Self {
        let storage = match config.exporter_ftp.enabled {
            true => Some(FtpStorage::init(config.exporter_ftp.clone())),
            false => None,
        };

        Downloader { 
            storage: Arc::new(Mutex::new(storage)),
            session, 
            display_cover_background: config.gui.display_cover_background, 
            display_cover_foreground: config.gui.display_cover_foreground,
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
            let url = self.session.get_track_url(track.id.clone())?;
            
            let file_response = Client::builder()
                .timeout(Duration::from_secs(300))
                .build()?.get(url).send()?;
    
            if !file_response.status().is_success() {
                continue;
            }

            let bytes_response = file_response.bytes()?;
    
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
    
        let file_response = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?.get(&cover_url).send()?;
    
        let cover = ImageReader::new(Cursor::new(file_response.bytes()?)).with_guessed_format()?.decode()?;

        let path_str = match self.display_cover_foreground {
            true => {
                let file = NamedTempFile::new()?;
                let path = file.into_temp_path();
                let image_str = path.keep()?.to_str().unwrap().to_string();
                cover
                    .resize(320, 320, image::imageops::FilterType::Nearest)
                    .save_with_format(&image_str, image::ImageFormat::Png)
                    .unwrap();
                Some(image_str)
            },
            false => None,
        };

        let background_path_str = match self.display_cover_background {
            true => {
                let background = cover.clone();
                let background_file = NamedTempFile::new()?;
                let background_path = background_file.into_temp_path();
                let background_path_str = background_path.keep()?.to_str().unwrap().to_string();
                
                background
                    .brighten(-75)
                    .resize(1024, 1024, image::imageops::FilterType::Nearest)
                    .blur(10.0)
                    .save_with_format(&background_path_str, image::ImageFormat::Png)
                    .unwrap();

                Some(background_path_str)
            },
            false => None,
        };
        
        debug!("[Downloader] Cover prepared '{}', foreground: {:?}, background: {:?}", cover_url, path_str, background_path_str);
    
        Ok(Cover { 
            foreground: path_str,
            background: background_path_str, 
        })
    }
}