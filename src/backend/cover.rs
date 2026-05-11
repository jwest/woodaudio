use std::{error::Error, io::Cursor, path::PathBuf};

use bytes::Bytes;
use image::{io::Reader as ImageReader};
use tempfile::NamedTempFile;

pub struct CoverProcessor {
    bytes: Bytes,
}

impl CoverProcessor {
    pub fn new(bytes: Bytes) -> Self {
        Self { bytes }
    }

    pub fn generate_foreground(&self) -> Result<PathBuf, Box<dyn Error>> {
        let tmp_path = Self::generate_tmp_file()?;

        // Decode, resize, save, drop — all in one scope so DynamicImage is freed immediately
        {
            let image = ImageReader::new(Cursor::new(&self.bytes))
                .with_guessed_format()?
                .decode()?;

            image
                .resize(320, 320, image::imageops::FilterType::Nearest)
                .save_with_format(&tmp_path, image::ImageFormat::Png)?;
        }

        Ok(tmp_path)
    }

    pub fn generate_background(&self) -> Result<PathBuf, Box<dyn Error>> {
        let tmp_path = Self::generate_tmp_file()?;

        // Decode at small size — 128x128 is enough for blurred background
        // This avoids the huge 1024x1024 intermediate buffer (~4 MB RGBA)
        {
            let image = ImageReader::new(Cursor::new(&self.bytes))
                .with_guessed_format()?
                .decode()?;

            image
                .resize(128, 128, image::imageops::FilterType::Nearest)
                .brighten(-75)
                .blur(5.0)
                .save_with_format(&tmp_path, image::ImageFormat::Png)?;
        }

        Ok(tmp_path)
    }

    fn generate_tmp_file() -> Result<PathBuf, Box<dyn Error>> {
        let path = NamedTempFile::new()?.into_temp_path();
        let image_tmp_path = path.keep()?.to_str().unwrap().to_string();
        Ok(PathBuf::from(image_tmp_path))
    }
}
