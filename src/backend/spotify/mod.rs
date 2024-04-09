use std::{io::{self, Read, Seek, SeekFrom}, sync::Arc, time::Duration};

use bytes::{BufMut, BytesMut};
use librespot::{audio::{AudioDecrypt, AudioFile}, core::{config::SessionConfig, mercury::MercuryError, session::Session, spotify_id::{self, SpotifyId}}, discovery::Credentials, metadata::{Metadata, Playlist}};
use log::{error, warn};
use rand::thread_rng;
use rand::seq::SliceRandom;
use tokio::runtime::Runtime;

use librespot::metadata::FileFormat;
use crate::playlist::Track;

use super::Backend;

#[derive(Clone)]
pub struct SpotifyBackend {
    session: Session,
    rt: Arc<Runtime>,
}

impl Backend for SpotifyBackend {
    fn init(config: &mut crate::config::Config, player_bus: crate::playerbus::PlayerBus) -> Self {
        let rt = Runtime::new().unwrap();
        let session_config = SessionConfig::default();
        let credentials = Credentials::with_password(
            &config.spotify.username,
            &config.spotify.password
        );

        let (session, _) = rt.block_on(Session::connect(session_config, credentials, None, false)).unwrap();

        Self { session, rt: Arc::new(rt) }        
    }

    fn discovery(&self, discovery_fn: impl Fn(crate::playlist::Track)) {
        // https://open.spotify.com/track/2374M0fQpWi3dLnB54qaLX?si=b210c40e49054ec5
        // https://open.spotify.com/playlist/4eisu1M0pxHf3aIbWybbL2?si=3b8ef55483084ce7
        // let id = SpotifyId::from_base62("2374M0fQpWi3dLnB54qaLX").unwrap();
        // todo!()
        // let response = self.rt.block_on(self.session.mercury().get(uri))?;
        
        // match response.payload.first() {
        //     None => {
        //         warn!("Empty payload");
        //         panic!("{}", MercuryError.into());
        //     }
        //     Some(data) => match librespot::Album::Message::parse_from_bytes(data) {
        //         Err(e) => {
        //             warn!("Error parsing message from bytes: {}", e);
        //             panic!("{}", e);
        //         }
        //         Ok(msg) => match Self::parse(&msg, self.session) {
        //             Err(e) => {
        //                 warn!("Error parsing message: {:?}", e);
        //                 panic!("{}", e);
        //             }
        //             Ok(parsed_msg) => Ok(parsed_msg),
        //         },
        //     },
        // };

        let plist_uri = SpotifyId::from_uri("spotify:playlist:4eisu1M0pxHf3aIbWybbL2").unwrap();
        let mut plist = match self.rt.block_on(Playlist::get(&self.session, plist_uri)) {
            Ok(res) => res,
            Err(err) => {
                error!("[Backend Spotify] {:?}", err);
                panic!("{:?}", err);
            },
        };
        
        let mut rng_items = thread_rng();
        plist.tracks.shuffle(&mut rng_items);

        for track_id in plist.tracks {
            let plist_track = self.rt.block_on(librespot::metadata::Track::get(&self.session, track_id)).unwrap();
            
            let track = Track { 
                id: plist_track.id.to_base62().unwrap(), 
                title: plist_track.name, 
                artist_name: plist_track.artists[0].id.to_string(),
                album_name: plist_track.album.id.to_string(), 
                album_image: "0dfd3368-3aa1-49a3-935f-10ffb39803c0".to_string(), 
                duration: Duration::from_millis(plist_track.duration.try_into().unwrap()),
            };

            discovery_fn(track);
            return;
        }
    }

    fn get_track(&self, track_id: String) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
        let spotify_id = SpotifyId::from_base62(&track_id).unwrap();
        let plist_track = self.rt.block_on(librespot::metadata::Track::get(&self.session, spotify_id)).unwrap();
        let file_id = plist_track.files.get(&FileFormat::OGG_VORBIS_320).unwrap();
        let encrypted_file = self.rt.block_on(AudioFile::open(
            &self.session,
            *file_id,
            stream_data_rate(FileFormat::OGG_VORBIS_320),
            true,
        )).unwrap();

        let key = match self.rt.block_on(self.session.audio_key().request(spotify_id, *file_id)) {
            Ok(key) => key,
            Err(e) => {
                panic!("Unable to load decryption key: {:?}", e);
            }
        };

        let decrypted_file = AudioDecrypt::new(key, encrypted_file);
        let audio_file = Subfile::new(decrypted_file, 0xa7);

        let mut bytes = BytesMut::new();

        for b in audio_file.bytes() {
            bytes.put_u8(b.unwrap());
        }

        Ok(bytes.freeze())
    }

    fn get_cover(&self, cover_url: String) -> Result<bytes::Bytes, Box<dyn std::error::Error>> {
        // let plist_track = self.rt.block_on(librespot::metadata::Track::get(&self.session, SpotifyId::from_base62(&track_id).unwrap())).unwrap();
        // plist_track.
        todo!();
    }

    fn discovery_radio(&self, id: &str, discovery_fn: impl Fn(Vec<crate::playlist::Track>)) {
        todo!()
    }

    fn discovery_track(&self, id: &str, discovery_fn: impl Fn(Vec<crate::playlist::Track>)) {
        todo!()
    }

    fn discovery_album(&self, id: &str, discovery_fn: impl Fn(Vec<crate::playlist::Track>)) {
        todo!()
    }

    fn discovery_artist(&self, id: &str, discovery_fn: impl Fn(Vec<crate::playlist::Track>)) {
        todo!()
    }

    fn add_track_to_favorites(&self, track_id: &str) {
        todo!()
    }
}

fn stream_data_rate(format: FileFormat) -> usize {
    match format {
        FileFormat::OGG_VORBIS_96 => 12 * 1024,
        FileFormat::OGG_VORBIS_160 => 20 * 1024,
        FileFormat::OGG_VORBIS_320 => 40 * 1024,
        FileFormat::MP3_256 => 32 * 1024,
        FileFormat::MP3_320 => 40 * 1024,
        FileFormat::MP3_160 => 20 * 1024,
        FileFormat::MP3_96 => 12 * 1024,
        FileFormat::MP3_160_ENC => 20 * 1024,
        FileFormat::MP4_128_DUAL => 16 * 1024,
        FileFormat::OTHER3 => 40 * 1024, // better some high guess than nothing
        FileFormat::AAC_160 => 20 * 1024,
        FileFormat::AAC_320 => 40 * 1024,
        FileFormat::MP4_128 => 16 * 1024,
        FileFormat::OTHER5 => 40 * 1024, // better some high guess than nothing
    }
}

struct Subfile<T: Read + Seek> {
    stream: T,
    offset: u64,
}

impl<T: Read + Seek> Subfile<T> {
    pub fn new(mut stream: T, offset: u64) -> Subfile<T> {
        if let Err(e) = stream.seek(SeekFrom::Start(offset)) {
            error!("Subfile new Error: {}", e);
        }
        Subfile { stream, offset }
    }
}

impl<T: Read + Seek> Read for Subfile<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<T: Read + Seek> Seek for Subfile<T> {
    fn seek(&mut self, mut pos: SeekFrom) -> io::Result<u64> {
        pos = match pos {
            SeekFrom::Start(offset) => SeekFrom::Start(offset + self.offset),
            x => x,
        };

        let newpos = self.stream.seek(pos)?;

        Ok(newpos.saturating_sub(self.offset))
    }
}