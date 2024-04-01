use macroquad::prelude::*;

use crate::{config::Config, playerbus::{Message::SessionUpdated, State}, session::{DeviceAuthorization, Session}};

use super::Screen;

pub struct SessionGui {
    config: Config,
    internet_connection: bool,
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
        }
    }

    fn render_text(&self, text: String, ui: &super::Gui) {
        draw_text_ex(text.as_str(), 96.0, 96.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: WHITE, ..Default::default() },);
    }
}

impl Screen for SessionGui {
    fn update(&mut self, _: State) {
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

    fn render(&self, ui: &super::Gui) {
        if self.session.is_some() {
            ui.player_bus.publish_message(SessionUpdated(self.session.clone().unwrap()));
            return;
        }
        
        let link = self.device_auth.clone().map_or("Loading...".to_string(), |d| d.format_url());
        self.render_text(link, ui);
    }
    
    fn nav_id(&self) -> String {
        "/session".to_string()
    }
}