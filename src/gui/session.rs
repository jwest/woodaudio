use macroquad::prelude::*;
use qrcode_generator::QrCodeEcc;

use crate::playerbus::State;

use super::Screen;

pub struct SessionGui {
    device_login_link: Option<String>,
    qrcode_image: Option<Texture2D>,
}

impl SessionGui {
    pub fn init() -> SessionGui {
        Self { 
            device_login_link: None,
            qrcode_image: None,
        }
    }

    fn render_text(&self, text: String, ui: &super::Gui) {
        draw_text_ex(text.as_str(), 96.0, 96.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: WHITE, ..Default::default() },);
    }
}

impl Screen for SessionGui {
    fn update(&mut self, state: State) {
        if self.device_login_link.is_none() && state.device_login_link.is_some() {
            let code = qrcode_generator::to_png_to_vec(state.device_login_link.clone().unwrap().as_bytes(), QrCodeEcc::Low, 300).unwrap();
            self.qrcode_image = Some(Texture2D::from_file_with_format(&code, None));
        }
        self.device_login_link = state.device_login_link;
    }

    fn render(&self, ui: &super::Gui) {
        let link = self.device_login_link.clone().map_or("Loading...".to_string(), |link| link);
        self.render_text(link, ui);

        if let Some(image) = &self.qrcode_image {
            draw_texture_ex(image, screen_width() / 2.0 - 160.0, 144.0, WHITE, DrawTextureParams {
                rotation: 0.0,
                ..Default::default()
            });
        }
    }
    
    fn nav_id(&self) -> String {
        "/session".to_string()
    }
}