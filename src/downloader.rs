use std::{error::Error, io::{self, Cursor, Read}, time::Duration};

use bytes::Buf;
use image::io::Reader as ImageReader;
use log::{debug, info};
use pavao::{SmbClient, SmbCredentials, SmbOpenOptions, SmbOptions};
use reqwest::blocking::Client;
use secular::normalized_lower_lay_string;
use tempfile::NamedTempFile;

use crate::{config::{Config, ExporterSambaConfig}, playlist::{BufferedTrack, Cover, Track}, session::Session};

trait CacheRead {
    fn read_file(&self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<bytes::Bytes>, Box<dyn Error>>;
}

trait Exporter {
    fn write_file(&self, source: bytes::Bytes, output_file_name: &str, output_dir: Option<&str>) -> Result<(), Box<dyn Error>>;
}

struct SambaStorage {
    client: SmbClient,
    cache_read: bool,
}

impl SambaStorage {
    fn init(config: ExporterSambaConfig) -> Self {
        let client = SmbClient::new(
            SmbCredentials::default()
                .server(config.server)
                .share(config.share)
                .password(config.password)
                .username(config.username)
                .workgroup(config.workgroup),
            SmbOptions::default().one_share_per_server(true),
        ).unwrap();

        Self { client, cache_read: config.cache_read }
    }
}

impl Exporter for SambaStorage {
    fn write_file(&self, source: bytes::Bytes, output_file_name: &str, output_dir: Option<&str>) -> Result<(), Box<dyn Error>> {
        let file_name = match output_dir {
            Some(dir) => {
                self.client.mkdir(dir, 0o755.into())?;
                format!("/{}/{}", dir, output_file_name)
            },
            None => format!("/{}", output_file_name),
        };

        let mut reader = source.reader();
        let mut writer = self.client.open_with(
            file_name.clone(),
            SmbOpenOptions::default().create(true).write(true),
        )?;

        io::copy(&mut reader, &mut writer)?;
        info!("[Exporter] file saved on samba: {:?}", file_name);
        Ok(())
    }
}

impl CacheRead for SambaStorage {
    fn read_file(&self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<bytes::Bytes>, Box<dyn Error>> {
        if !self.cache_read {
            return Ok(None);
        }

        let file_name = match output_dir {
            Some(dir) => {
                self.client.mkdir(dir, 0o755.into())?;
                format!("/{}/{}", dir, output_file_name)
            },
            None => format!("/{}", output_file_name),
        };

        let mut reader = self.client.open_with(
            file_name.clone(),
            SmbOpenOptions::default().create(true).write(true),
        )?;

        let mut output = bytes::BytesMut::new();
        reader.read(&mut output)?;
        drop(reader);

        if output.is_empty() {
            return Ok(None);
        }

        info!("[Cache] file readed from samba: {:?} {:?}", file_name, output);

        Ok(Some(output.into()))
    }
}

pub struct Downloader {
    storage: Option<SambaStorage>,
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
        let storage = match config.exporter_samba.enabled {
            true => Some(SambaStorage::init(config.exporter_samba.clone())),
            false => None,
        };

        Downloader { 
            storage,
            session, 
            display_cover_background: config.gui.display_cover_background, 
            display_cover_foreground: config.gui.display_cover_foreground,
        }
    }
    
    pub fn download_file(&self, track: Track) -> Result<BufferedTrack, Box<dyn Error>> {
        if self.storage.is_some() {
            match self.storage.as_ref().unwrap().read_file(&track.file_name(), None) {
                Ok(Some(file)) => {
                    return Ok(BufferedTrack {
                        track: track.clone(),
                        stream: file,
                        cover: match self.download_album_cover(track.album_image) {
                            Ok(cover) => cover,
                            Err(_) => Cover::empty(),
                        },
                    })
                },
                _ => {
                    debug!("[Storage] cache empty or error for {:?}", track);
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
    
            if self.storage.is_some() {
                let export_bytes = bytes_response.clone();
                self.storage.as_ref().unwrap().write_file(export_bytes, &track.file_name(), None)?;
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
    
        panic!("Track Download fail!");
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
                let _ = cover
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
                
                let _ = background
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