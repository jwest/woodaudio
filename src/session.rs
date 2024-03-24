use reqwest::blocking::{Client, Response};
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;

use std::error::Error;
use std::time::Duration;
use std::{time, thread};
use log::info;

use crate::config::Config;

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

#[derive(Debug)]
#[derive(Clone)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceAuthorization {
    verification_uri_complete: String,
    device_code: String,
    // expires_in: u16,
    // interval: u16,
}

impl DeviceAuthorization {
    pub fn format_url(&self) -> String {
        format!("https://{}", self.verification_uri_complete)
    }
    pub fn wait_for_link(&self, config: &mut Config) -> Result<Session, Box<dyn Error>> {
        let client_id = "zU4XHVVkc2tDPo4t";
        let client_secret = "VJKhDFqJPqvsPVNBV6ukXTJmwlvbttP7wlMlrc72se4%3D";
        let client = reqwest::blocking::Client::builder().build()?;

        for _ in 0..60 {
            thread::sleep(Duration::from_secs(2));

            let params = &[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("device_code", &self.device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("scope", "r_usr w_usr w_sub"),
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

                return Session::try_from_file(&config)
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
    pub fn try_from_file(config: &Config) -> Result<Session, Box<dyn Error>> {
        Session::init(format!("{} {}", config.tidal.token_type, config.tidal.access_token))
    }
    pub fn login_link() -> Result<DeviceAuthorization, Box<dyn Error>> {
        let client_id = "zU4XHVVkc2tDPo4t";
        let client = reqwest::blocking::Client::builder()
            .build()?;
        let res = client.post("https://auth.tidal.com:443/v1/oauth2/device_authorization")
            .form(&[("client_id", client_id), ("scope", "r_usr+w_usr+w_sub")])
            .send()?;

        let device_auth_response = res.json::<DeviceAuthorization>()?;
        info!("[Session] login link: {}, waiting...", device_auth_response.format_url());

        Ok(device_auth_response)
    }
    pub fn init(token: String) -> Result<Session, Box<dyn Error>> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token.as_str()).unwrap());
    
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
                token: token.clone(),
                api_path: "https://api.tidal.com/v1".to_string(),
            });
        }

        info!("[Session] outdated, refresh needed {:?}", res);

        Err("Session is outdated".into())
    }
    pub fn check_internet_connection() -> bool {
        info!("Wait for internet connection to tidal... ");
        let res = reqwest::blocking::Client::default().get("https://api.tidal.com/").send();
        res.is_ok()
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
