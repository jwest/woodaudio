use std::{error::Error, fs, io::Read};
use std::path::PathBuf;

use bytes::{Buf, Bytes};
use log::info;
use suppaftp::{types::FileType, FtpStream};

use crate::config::{ExporterFile, ExporterFTP};


pub trait CacheRead {
    fn read_file(&mut self, output_file_name: &str, output_dir: Option<&str>) -> Result<Option<bytes::Bytes>, Box<dyn Error>>;
}

pub trait Exporter {
    fn write_file(&mut self, source: bytes::Bytes, output_file_name: &str, output_dir: Option<&str>) -> Result<(), Box<dyn Error>>;
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

impl Exporter for FileStorage {
    fn write_file(&mut self, source: Bytes, output_file_name: &str, _output_dir: Option<&str>) -> Result<(), Box<dyn Error>> {
        let file_name = self.file_name_with_create_dir(output_file_name)?;
        fs::write(file_name, source)?;
        Ok(())
    }
}