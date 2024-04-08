use std::{error::Error, io::Read};

use bytes::Buf;
use log::info;
use suppaftp::{types::FileType, FtpStream};

use crate::config::ExporterFTP;


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