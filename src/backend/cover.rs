use std::{error::Error, io::Cursor, path::PathBuf};
use std::time::Duration;

use bytes::Bytes;
use image::{io::Reader as ImageReader, DynamicImage};
use log::error;
use reqwest::blocking::Client;
use tempfile::NamedTempFile;

pub struct CoverProcessor {
    image: DynamicImage,
}

impl CoverProcessor {
    pub fn new(bytes: Bytes) -> Self {
        let image = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        Self { image }
    }

    pub fn generate_foreground(&self) -> Result<PathBuf, Box<dyn Error>> {
        let tmp_path = Self::generate_tmp_file()?;
        
        self.image
            .resize(320, 320, image::imageops::FilterType::Nearest)
            .save_with_format(&tmp_path, image::ImageFormat::Png)
            .unwrap();

        Ok(tmp_path)
    }

    pub fn generate_background(&self) -> Result<PathBuf, Box<dyn Error>> {
        let tmp_path = Self::generate_tmp_file()?;
        
        self.image
            .brighten(-75)
            .resize(1024, 1024, image::imageops::FilterType::Nearest)
            .blur(10.0)
            .save_with_format(&tmp_path, image::ImageFormat::Png)
            .unwrap();

        Ok(tmp_path)
    }

    fn generate_tmp_file() -> Result<PathBuf, Box<dyn Error>> {
        let path = NamedTempFile::new()?.into_temp_path();
        let image_tmp_path = path.keep()?.to_str().unwrap().to_string();
        Ok(PathBuf::from(image_tmp_path))
    }
}