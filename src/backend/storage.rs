use std::{error::Error, fs, io::Read};
use std::path::PathBuf;
use rand::seq::IteratorRandom;

use bytes::{Buf, Bytes};
use log::info;
use metaflac::block::{Picture, PictureType, VorbisComment};
use metaflac::Tag;
use suppaftp::{types::FileType, FtpStream};
use tempfile::NamedTempFile;

use crate::config::{ExporterFile, ExporterFTP};
use crate::playlist::{BufferedTrack, Cover, Track};

extern crate rand;

pub trait CacheRead {
    fn read_file(&mut self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<Bytes>, Box<dyn Error>>;
}

pub trait CacheRandomRead {
    fn read_random_file(&mut self, output_dir: Option<&str>) -> Result<Option<BufferedTrack>, Box<dyn Error>>;
}

pub trait Exporter {
    fn write_file(&mut self, track: Track, source: Bytes, output_file_name: &str, output_dir: Option<&str>, cover: Option<Vec<u8>>) -> Result<(), Box<dyn Error>>;
}

pub struct FtpStorage {
    client: FtpStream,
    cache_read: bool,
    output_path: String,
}

impl FtpStorage {
    pub fn init(config: ExporterFTP) -> Self {
        let client = Self::connect_client(config.clone());
        Self { client, cache_read: config.cache_read, output_path: config.share }
    }
    fn connect_client(config: ExporterFTP) -> FtpStream {
        let mut client = FtpStream::connect(config.server).unwrap();
        client.login(config.username, config.password).unwrap();
        client.transfer_type(FileType::Binary).unwrap();
        client.set_mode(suppaftp::Mode::ExtendedPassive);
        client
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
    fn write_file(&mut self, _: Track, source: Bytes, output_file_name: &str, output_dir: Option<&str>, _: Option<Vec<u8>>) -> Result<(), Box<dyn Error>> {
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
        reader.read(&mut output)?;
        drop(reader);

        if output.is_empty() {
            return Ok(None);
        }

        info!("[Cache] file readed from ftp: {:?} {:?}", file_name, output);

        Ok(Some(output.into()))
    }
}

#[derive(Clone)]
pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn init(config: ExporterFile) -> Self {
        Self { path: PathBuf::from(config.path) }
    }
    fn file_name_with_create_dir(&mut self, output_file_name: &str) -> Result<String, Box<dyn Error>> {
        fs::create_dir_all(&self.path)?;
        Ok(format!("{}/{output_file_name}", &self.path.to_str().unwrap()))
    }

    fn get_or_default(tag_content: Option<&Vec<String>>) -> String {
        tag_content.unwrap_or(&vec![]).get(0).unwrap_or(&"".to_string()).to_string()
    }

    fn generate_tmp_file() -> Result<PathBuf, Box<dyn Error>> {
        let path = NamedTempFile::new()?.into_temp_path();
        let image_tmp_path = path.keep()?.to_str().unwrap().to_string();
        Ok(PathBuf::from(image_tmp_path))
    }
}

impl CacheRead for FileStorage {
    fn read_file(&mut self, output_file_name: &str, _output_dir: Option<&str>) -> Result<Option<Bytes>, Box<dyn Error>> {
        let file_name = self.file_name_with_create_dir(output_file_name)?;
        match fs::read(file_name) {
            Ok(file) => Ok(Some(bytes::Bytes::from(file))),
            Err(_) => Ok(None)
        }
    }
}

impl CacheRandomRead for FileStorage {
    fn read_random_file(&mut self, _output_dir: Option<&str>) -> Result<Option<BufferedTrack>, Box<dyn Error>> {
        let mut rng = rand::thread_rng();
        let files = fs::read_dir(&self.path)?;
        let file = files.choose(&mut rng).unwrap()?;

        match fs::read(file.path()) {
            Ok(content) => {
                let tag = Tag::read_from_path(file.path()).unwrap_or_default();
                let vorbis_comment = VorbisComment::new();
                let vorbis = tag.vorbis_comments().unwrap_or(&vorbis_comment);

                let front = tag.pictures().filter(|picture| picture.picture_type == PictureType::CoverFront).next();

                let cover = match front {
                    Some(picture) => {
                        let file = Self::generate_tmp_file()?;
                        fs::write(&file, &picture.data)?;
                        Cover {
                            foreground: Some(file.to_str().unwrap().to_string()),
                            background: None,
                        }
                    },
                    None => {
                        Cover::empty()
                    },
                };

                let buffered_track = BufferedTrack {
                    track: Track {
                        id: "".to_string(),
                        title: Self::get_or_default(vorbis.title()),
                        artist_name: Self::get_or_default(vorbis.artist()),
                        album_name: Self::get_or_default(vorbis.album()),
                        album_image: "".to_string(),
                        duration: Default::default(),
                    },
                    stream: bytes::Bytes::from(content),
                    cover,
                };
                Ok(Some(buffered_track))
            },
            Err(_) => Ok(None)
        }
    }
}

impl Exporter for FileStorage {
    fn write_file(&mut self, track: Track, source: Bytes, output_file_name: &str, _output_dir: Option<&str>, cover: Option<Vec<u8>>) -> Result<(), Box<dyn Error>> {
        let file_name = self.file_name_with_create_dir(output_file_name)?;
        fs::write(file_name.clone(), source)?;

        let mut tag = Tag::read_from_path(file_name)?;
        let vorbis = tag.vorbis_comments_mut();
        vorbis.set_title(vec![track.title]);
        vorbis.set_album(vec![track.album_name]);
        vorbis.set_artist(vec![track.artist_name]);

        if let Some(cover) = cover {
            tag.add_picture("image/png", PictureType::CoverFront, cover);
        }

        tag.save()?;
        Ok(())
    }
}