use std::time::Duration;
use macroquad::prelude::*;

use crate::{config::Config, session::{DeviceAuthorization, Session}};

pub struct SessionGui {
    config: Config,
    internet_connection: bool,
    font: Font,
    session: Option<Session>,
    device_auth: Option<DeviceAuthorization>,
}

impl SessionGui {
    pub fn init(config: Config) -> SessionGui {
        Self { 
            session: None,
            device_auth: None,
            internet_connection: false,
            config,
            font: load_ttf_font_from_bytes(include_bytes!("../static/NotoSans_Condensed-SemiBold.ttf")).unwrap(),
        }
    }

    pub async fn update_state(&mut self) {
        if !self.internet_connection {
            self.internet_connection = Session::check_internet_connection();
        } else if self.device_auth.is_some() {
            match self.device_auth.clone().unwrap().wait_for_link(&mut self.config) {
                Ok(session) => self.session = Some(session.clone()),
                Err(_) => {
                    self.device_auth = Some(Session::login_link().unwrap());
                },
            }
        } else if self.session.is_none() {
            match Session::try_from_file(&self.config) {
                Ok(session) => self.session = Some(session),
                Err(_) => {
                    self.device_auth = Some(Session::login_link().unwrap());
                },
            }
        }
    }
    
    pub async fn gui_loop(&mut self) -> Session {
        loop {
            clear_background(BLACK);

            if self.session.is_some() {
                return self.session.clone().unwrap();
            }
            
            let link = self.device_auth.clone().map(|d| d.format_url()).unwrap_or("Loading...".to_string());
            self.render_text(link);
    
            next_frame().await;

            self.update_state().await;
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn render_text(&self, text: String) {
        draw_text_ex(text.as_str(), 96.0, 96.0,  TextParams { font_size: 32, font: Some(&self.font), color: WHITE, ..Default::default() },);
    }
}