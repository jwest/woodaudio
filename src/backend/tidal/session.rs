use bytes::Bytes;
use reqwest::blocking::{Client, Response};
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;

use std::error::Error;
use std::time::Duration;
use std::thread;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use log::{debug, error, info, warn};

use crate::config::{Config, Tidal};
use crate::state::{Message, PlayerBus};

#[derive(Debug)]
#[derive(Clone)]
pub(super) struct Session {
    session_id: String,
    country_code: String,
    user_id: i64,
    token: String,
    api_path: String,
    audio_quality: String,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseTidalSession {
    session_id: String,
    country_code: String,
    user_id: i64,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResponseMedia {
    urls: Vec<String>,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceAuthorization {
    verification_uri_complete: String,
    device_code: String,
    // expires_in: u16,
    // interval: u16,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
struct RefreshAuthorization {
    token_type: String,
    access_token: String,
}

fn client_id() -> String {
    String::from_utf8(BASE64_STANDARD.decode([
        BASE64_STANDARD.decode(b"WmxneVNuaGtiVzUw").unwrap(),
        BASE64_STANDARD.decode(b"V2xkTE1HbDRWQT09").unwrap()
    ].concat()).unwrap()).unwrap()
}
fn client_secret() -> String {
    String::from_utf8(BASE64_STANDARD.decode([
        BASE64_STANDARD.decode(b"TVU1dU9VRm1SRUZxZUhKblNrWktZa3RPVjB4bFFY").unwrap(),
        BASE64_STANDARD.decode(b"bExSMVpIYlVsT2RWaFFVRXhJVmxoQmRuaEJaejA9").unwrap()
    ].concat()).unwrap()).unwrap()
}

impl Tidal {
    pub fn token(&self) -> String {
        format!("{} {}", self.token_type, self.access_token)
    }
}

impl DeviceAuthorization {
    fn format_url(&self) -> String {
        format!("https://{}", self.verification_uri_complete)
    }
    fn wait_for_link(&self, config: &mut Config) -> Result<Session, Box<dyn Error>> {
        let client = reqwest::blocking::Client::builder().build()?;

        for _ in 0..60 {
            thread::sleep(Duration::from_secs(2));

            let params = &[
                ("client_id", client_id()),
                ("client_secret", client_secret()),
                ("device_code", self.device_code.clone()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code".to_string()),
                ("scope", "r_usr w_usr w_sub".to_string()),
            ];
            
            let res = client.post("https://auth.tidal.com:443/v1/oauth2/token")
                .form(params)
                .send()?;

            info!("[Session] token resposne: {:?}", res.status());

            if res.status().is_success() {
                let session_response = res.json::<ResponseSession>()?;

                config.tidal.token_type = session_response.token_type;
                config.tidal.access_token = session_response.access_token;
                config.tidal.refresh_token = session_response.refresh_token;
                config.save();

                return Session::try_from_file(config)
            }
        }

        self.wait_for_link(config)
    }
}

#[derive(Debug)]
#[derive(Deserialize)]
struct ResponseSession {
    access_token: String,
    refresh_token: String,
    token_type: String,
}

impl Session {
    pub(super) fn setup(config: &mut Config, player_bus: PlayerBus) -> Session {
        player_bus.publish_message(Message::TidalBackendStarted);
        Session::check_internet_connection();

        match Session::try_from_file(config) {
            Ok(session) => {
                player_bus.publish_message(Message::TidalBackendInitialized);
                return session;
            },
            _ => {}
        }

        let device_auth = Session::login_link().unwrap();

        player_bus.publish_message(Message::TidalBackendLoginLinkCreated(device_auth.clone().format_url()));

        match device_auth.wait_for_link(config) {
            Ok(session) => {
                player_bus.publish_message(Message::TidalBackendInitialized);
                session
            },
            Err(_) => Session::setup(config, player_bus),
        }
    }
    fn try_from_file(config: &mut Config) -> Result<Session, Box<dyn Error>> {
        Session::init(config)
    }
    fn login_link() -> Result<DeviceAuthorization, Box<dyn Error>> {

        let client = reqwest::blocking::Client::builder()
            .build()?;
        let res = client.post("https://auth.tidal.com:443/v1/oauth2/device_authorization")
            .form(&[("client_id", client_id().as_str()), ("scope", "r_usr+w_usr+w_sub")])
            .send();

        let device_auth_response = if res.is_ok() {
            res?.json::<DeviceAuthorization>()?
        } else {
            warn!("[Session] waiting for login link, next try...");
            thread::sleep(Duration::from_secs(1));
            Self::login_link()?
        };
        info!("[Session] login link: {}, waiting...", device_auth_response.format_url());

        Ok(device_auth_response)
    }
    fn init(config: &mut Config) -> Result<Session, Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(config.tidal.token().as_str()).unwrap());
    
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        let res = client.get("https://api.tidal.com/v1/sessions").send()?;

        if res.status().is_success() {
            let session = res.json::<ResponseTidalSession>()?;
        
            info!("[Session] {:?}", session);
        
            return Ok(Session { 
                session_id: session.session_id, 
                country_code: session.country_code, 
                user_id: session.user_id, 
                token: config.tidal.token().clone(),
                api_path: "https://api.tidal.com/v1".to_string(),
                audio_quality: config.tidal.audio_quality.clone(),
            });
        }

        info!("[Session] outdated, refresh needed, {:?}", res);

        Self::refresh_token(config)?;
        Self::init(config)
    }
    fn read_session(token: &str) -> Result<ResponseTidalSession, Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token).unwrap());

        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        let res = client.get("https://api.tidal.com/v1/sessions").send()?;

        let session = res.json::<ResponseTidalSession>()?;
        Ok(session)
    }
    fn refresh_token(config: &mut Config) -> Result<(), Box<dyn Error>> {
        let client = reqwest::blocking::Client::builder()
            .build()?;
        let res = client.post("https://auth.tidal.com:443/v1/oauth2/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", config.tidal.refresh_token.as_str()),
                ("client_id", client_id().as_str()),
                ("client_secret", client_secret().as_str()),
            ])
            .send()?;

        let refresh_auth_response = res.json::<RefreshAuthorization>()?;

        config.tidal.token_type = refresh_auth_response.token_type;
        config.tidal.access_token = refresh_auth_response.access_token;
        config.save();
        info!("[Session] refreshed with success");

        Ok(())
    }
    fn check_internet_connection() {
        let res = reqwest::blocking::Client::default().get("https://api.tidal.com/").send();
        if res.is_err() {
            warn!("Wait for internet connection to tidal, next try... ");
            thread::sleep(Duration::from_secs(2));
            Self::check_internet_connection()
        }
    }
    fn build_client(&self) -> Client {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(self.token.as_str()).unwrap());

        reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build().unwrap()
    }
    fn request(&self, url: String) -> Result<Response, Box<dyn Error>> {
        let res = self.build_client().get(url).send()?;
        Ok(res)
    }
    pub(super) fn get_page_for_you(&self) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/pages/for_you?countryCode={}&deviceType=BROWSER", self.api_path, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub(super) fn get_mix(&self, mix_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/pages/mix?countryCode={}&deviceType=BROWSER&mixId={}", self.api_path, self.country_code, mix_id))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub(super) fn get_favorites(&self) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/users/{}/favorites/tracks?countryCode={}&limit=100&offset=0", self.api_path, self.user_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    // pub(super) fn get_favorite_albums(&self) -> Result<Value, Box<dyn Error>> {
    //     let response = self.request(format!("{}/users/{}/favorites/albums?countryCode={}&limit=100&offset=0", self.api_path, self.user_id, self.country_code))?;
    //     let body = response.text()?;
    //     let result: Value = serde_json::from_str(&body)?;
    //     Ok(result)
    // }
    pub(super) fn get_album(&self, album_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/albums/{}/tracks?countryCode={}&deviceType=BROWSER", self.api_path, album_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub(super) fn get_artist(&self, artist_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/artists/{}/toptracks?countryCode={}&deviceType=BROWSER", self.api_path, artist_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub(super) fn get_track_radio(&self, track_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/tracks/{}/radio?countryCode={}&deviceType=BROWSER", self.api_path, track_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub(super) fn add_track_to_favorites(&self, track_id: &str) -> Result<(), Box<dyn Error>> {
        self.build_client().post(format!("{}/users/{}/favorites/tracks?countryCode={}&deviceType=BROWSER", self.api_path, self.user_id, self.country_code))
            .form(&[("trackId", track_id)])
            .send()?;
        Ok(())
    }
    pub(super) fn get_track_url(&mut self, track_id: String) -> Result<String, Box<dyn Error>> {
        let download_url = format!("{}/tracks/{}/urlpostpaywall?sessionId={}&urlusagemode=STREAM&audioquality={}&assetpresentation=FULL", self.api_path, track_id, self.session_id, self.audio_quality);
        debug!("Download track: {}, with url: {}", track_id, download_url);
        let response = self.request(download_url)?;
        if response.status().is_success() {
            let url = response.json::<ResponseMedia>()?.urls[0].clone();
            Ok(url)
        } else {
            if response.status().is_client_error() {
                let session = Self::read_session(self.token.as_str())?;
                error!("[SESSION] renew session old: {}, new: {}", self.session_id, session.session_id);
                self.session_id = self.session_id.clone();
            }
            let status_code = response.status().to_string();
            let body_text = response.text()?;
            info!("[Client] Failed to download track id: {} (status: {}, body: {})", track_id, status_code, body_text);

            Err(format!("Failed to download track id: {} (status: {}, body: {})", track_id, status_code, body_text).into())
        }
    }
    pub(super) fn get_track_bytes(&mut self, track_id: String) -> Result<Bytes, Box<dyn Error>> {
        let url = self.get_track_url(track_id.clone())?;
            
        let file_response = Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?.get(url).send()?;

        Ok(file_response.bytes()?)
    }
    pub(super) fn get_cover_bytes(&self, cover_url: String) -> Result<Bytes, Box<dyn Error>> {
        let file_response = Client::builder()
            .timeout(Duration::from_secs(500))
            .build()?
            .get(&cover_url).send()?
            .bytes()?;

        Ok(file_response)
    }
}
