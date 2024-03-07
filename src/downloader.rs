use std::{error::Error, io::Cursor, time::Duration};

use image::io::Reader as ImageReader;
use log::debug;
use reqwest::blocking::Client;
use tempfile::NamedTempFile;

use crate::{playlist::{BufferedTrack, Cover, Track}, session::Session};

fn download_album_cover(cover_url: String) -> Result<Cover, Box<dyn Error>> {
    debug!("[Downloader] Prepare cover '{}'...", cover_url);

    let file_response = Client::builder()
        .timeout(Duration::from_secs(300))
        .build()?.get(&cover_url).send()?;

    let cover = ImageReader::new(Cursor::new(file_response.bytes()?)).with_guessed_format()?.decode()?;
    let background = cover.clone();

    let file = NamedTempFile::new()?;
    let path = file.into_temp_path();
    let path_str = path.keep()?.to_str().unwrap().to_string();
    
    let _ = cover
        .resize(320, 320, image::imageops::FilterType::Nearest)
        .save_with_format(&path_str, image::ImageFormat::Png)
        .unwrap();

    let background_file = NamedTempFile::new()?;
    let background_path = background_file.into_temp_path();
    let background_path_str = background_path.keep()?.to_str().unwrap().to_string();
    
    let _ = background
        .brighten(-75)
        .resize(1024, 1024, image::imageops::FilterType::Nearest)
        .blur(10.0)
        .save_with_format(&background_path_str, image::ImageFormat::Png)
        .unwrap();
    
    debug!("[Downloader] Cover prepared '{}', {}", cover_url, path_str);

    Ok(Cover { background: background_path_str, foreground: path_str })
}

pub fn download_file(track: Track, session: &Session) -> Result<BufferedTrack, Box<dyn Error>> {
    for _ in 1..5 {
        let url = session.get_track_url(track.id.clone())?;
        
        let file_response = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?.get(url).send()?;

        if !file_response.status().is_success() {
            continue;
        }

        return Ok(BufferedTrack {
            track: track.clone(),
            stream: file_response.bytes()?,
            cover: match download_album_cover(track.album_image) {
                Ok(cover) => Some(cover),
                Err(_) => None,
            },
        })
    }

    panic!("Track Download fail!");
}