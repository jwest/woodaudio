use reqwest::blocking::{Client, Response};
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;
use ini::Ini;

use std::error::Error;
use std::{time, thread};
use log::info;

#[derive(Debug)]
#[derive(Clone)]
pub struct Session {
    session_id: String,
    country_code: String,
    user_id: i64,
    token: String,
    api_path: String,
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

impl Session {
    pub fn init_from_config_file() -> Result<Session, Box<dyn Error>> {
        let config_path = home::home_dir().unwrap().join("config.ini");
        let conf = Ini::load_from_file(config_path).unwrap();

        let tidal_section = conf.section(Some("Tidal")).unwrap();
        let token_type = tidal_section.get("token_type").unwrap();
        let access_token = tidal_section.get("access_token").unwrap();

        Session::init(format!("{} {}", token_type, access_token))
    }
    pub fn init(token: String) -> Result<Session, Box<dyn Error>> {
        Session::wait_for_internet_connection();

        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token.as_str()).unwrap());
    
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()?;
        let res = client.get("https://api.tidal.com/v1/sessions").send()?;

        let session = res.json::<ResponseTidalSession>()?;
    
        info!("[Session] {:?}", session);
    
        Ok(Session { 
            session_id: session.session_id, 
            country_code: session.country_code, 
            user_id: session.user_id, 
            token: token.clone(),
            api_path: "https://api.tidal.com/v1".to_string(),
        })
    }
    fn wait_for_internet_connection() {
        loop {
            info!("Wait for internet connection to tidal... ");
            let res = reqwest::blocking::Client::default().get("https://api.tidal.com/").send();
            if res.is_ok() {
                break;
            }
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
    pub fn get_page_for_you(&self) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/pages/for_you?countryCode={}&deviceType=BROWSER", self.api_path, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn get_mix(&self, mix_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/pages/mix?countryCode={}&deviceType=BROWSER&mixId={}", self.api_path, self.country_code, mix_id))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn get_favorites(&self) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/users/{}/favorites/tracks?countryCode={}&limit=100&offset=0", self.api_path, self.user_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn get_album(&self, album_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/albums/{}/tracks?countryCode={}&deviceType=BROWSER", self.api_path, album_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn get_artist(&self, artist_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/artists/{}/toptracks?countryCode={}&deviceType=BROWSER", self.api_path, artist_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn get_track_radio(&self, track_id: &str) -> Result<Value, Box<dyn Error>> {
        let response = self.request(format!("{}/tracks/{}/radio?countryCode={}&deviceType=BROWSER", self.api_path, track_id, self.country_code))?;
        let body = response.text()?;
        let result: Value = serde_json::from_str(&body)?;
        Ok(result)
    }
    pub fn add_track_to_favorites(&self, track_id: &str) -> Result<(), Box<dyn Error>> {
        self.build_client().post(format!("{}/users/{}/favorites/tracks?countryCode={}&deviceType=BROWSER", self.api_path, self.user_id, self.country_code))
            .form(&[("trackId", track_id)])
            .send()?;
        Ok(())
    }
    pub fn get_track_url(&self, track_id: String) -> Result<String, Box<dyn Error>> {
        let response = self.request(format!("{}/tracks/{}/urlpostpaywall?sessionId={}&urlusagemode=STREAM&audioquality=LOSSLESS&assetpresentation=FULL", self.api_path, track_id, self.session_id))?;
        if response.status().is_success() {
            let url = response.json::<ResponseMedia>()?.urls[0].clone();
            Ok(url)
        } else {
            let status_code = response.status().to_string();
            let body_text = response.text()?;
            info!("[Client] Retry download track id: {} in 5s... ({}: {})", track_id, status_code, body_text);
            thread::sleep(time::Duration::from_secs(5));
            self.get_track_url(track_id)
        }
    }
}
