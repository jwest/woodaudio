use std::{io::Read, time::{Duration, Instant}};
use macroquad::prelude::*;

use crate::playerbus::{self, PlayerBus, PlayerState, PlayerTrackState};

pub struct Gui {
    track_state: PlayerTrackState,
    buttons: Vec<String>,
    buttons_state: Vec<bool>,
}

impl Gui {
    pub fn default_state() -> Gui {
        let track_state = PlayerTrackState::default_state();

        let buttons: Vec<String> = vec![
            "".to_string(),
            "".to_string(), 
            //"".to_string(), 
            //"".to_string(), 
            //"".to_string(),
        ];

        let mut buttons_state: Vec<bool> = Vec::with_capacity(buttons.len());
        for i in 0..buttons.len() {
            buttons_state.push(false);
        }

        Gui { track_state, buttons, buttons_state }
    }

    fn duration_formated(duration: &Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        format!("{}:{:0>2}", minutes, seconds)
    }

    pub async fn gui_loop(&mut self, player_bus: PlayerBus) {
        println!("Load fonts");
        let font_title = load_ttf_font_from_bytes(include_bytes!("../static/NotoSans_Condensed-SemiBold.ttf")).unwrap();
        let font_subtitle = load_ttf_font_from_bytes(include_bytes!("../static/NotoSans_Condensed-Light.ttf")).unwrap();
        let font_icons = load_ttf_font_from_bytes(include_bytes!("../static/fontello.ttf")).unwrap();
        
        println!("Load textures");
        let mut cover_background: Texture2D = Texture2D::from_file_with_format(include_bytes!("../static/sample_cover.jpg-background.png"), Some(ImageFormat::Png));
        let mut cover_foreground: Texture2D = Texture2D::from_file_with_format(include_bytes!("../static/sample_cover.jpg-foreground.png"), Some(ImageFormat::Png));

        loop {
            match self.track_state.player_state {
                PlayerState::Playing => {},
                PlayerState::Paused => {
                    self.track_state.playing_time = Instant::now() - self.track_state.duration;
                },
                PlayerState::Loading => {},
            }

            let new_state = player_bus.read_state();
            if new_state.is_some() {
                self.track_state = new_state.unwrap();
                if (self.track_state.cover.is_some()) {
                    cover_foreground = load_texture(self.track_state.cover.clone().unwrap().clone().as_str()).await.unwrap();
                }
                if (self.track_state.cover_background.is_some()) {
                    cover_background = load_texture(self.track_state.cover_background.clone().unwrap().clone().as_str()).await.unwrap();
                }
            }
            
            clear_background(BLACK);
    
            draw_texture_ex(&cover_background, 0.0, -212.0, WHITE, DrawTextureParams {
                rotation: 0.0,
                ..Default::default()
            });
    
            draw_texture_ex(&cover_foreground, screen_width() / 2.0 - 160.0, 96.0, WHITE, DrawTextureParams {
                rotation: 0.0,
                ..Default::default()
            });
    
            draw_text_ex(&self.track_state.title, 17.0, 41.0,  TextParams { font_size: 32, font: Some(&font_title), color: BLACK, ..Default::default() },);
            draw_text_ex(&self.track_state.title, 16.0, 40.0,  TextParams { font_size: 32, font: Some(&font_title), color: WHITE, ..Default::default() },);
            
            draw_text_ex(format!("{} - {}", self.track_state.artist_name, self.track_state.album_name).as_str(), 17.0, 73.0, TextParams { font_size: 24, font: Some(&font_subtitle), color: BLACK, ..Default::default() },);
            draw_text_ex(format!("{} - {}", self.track_state.artist_name, self.track_state.album_name).as_str(), 16.0, 72.0, TextParams { font_size: 24, font: Some(&font_subtitle), color: WHITE, ..Default::default() },);
    
            let button_size: f32 = 48.0;
            let button_margin: f32 = 32.0;
    
            let button_len = self.buttons.len() as i16;
            
            let buttons_widget_width = f32::from(button_len) * button_size + (f32::from(button_len)-1.0) * button_margin;
            let buttons_start_position = screen_width() / 2.0 - buttons_widget_width / 2.0;
    
            let button_y = screen_height() - 72.0 - button_size;
    
            let time_duration_actual = Instant::now() - self.track_state.playing_time;
            let seconds = time_duration_actual.as_secs() % 60;
            let minutes = (time_duration_actual.as_secs() / 60) % 60;
            let time_text_actual = format!("{}:{:0>2}", minutes, seconds);
    
            let time_text_end = Self::duration_formated(&self.track_state.duration);
            let time_text_font_size = 16;
    
            let time_percentage = time_duration_actual.as_secs_f32() / self.track_state.duration.as_secs_f32();
    
            let time_text_center = get_text_center(time_text_end.as_str(), Some(&font_icons), time_text_font_size, 1.0, 0.0);
    
            draw_rectangle(
                buttons_start_position - 48.0, 
                button_y - 32.0, 
                buttons_widget_width + 48.0 + 48.0, 
                4.0, 
                GRAY
            );
    
            draw_rectangle(
                buttons_start_position - 48.0, 
                button_y - 32.0, 
                (buttons_widget_width + 48.0 + 48.0) * time_percentage, 
                4.0, 
                WHITE
            );
    
            draw_text_ex(
                time_text_actual.as_str(), 
                buttons_start_position - 96.0, 
                button_y - 24.0 - time_text_center.y - 1.0, 
                TextParams { font_size: time_text_font_size, font: Some(&font_subtitle), color: WHITE, ..Default::default() },
            );
    
            draw_text_ex(
                time_text_end.as_str(), 
                buttons_start_position + buttons_widget_width + 96.0,
                button_y - 24.0 - time_text_center.y - 1.0, 
                TextParams { font_size: time_text_font_size, font: Some(&font_subtitle), color: WHITE, ..Default::default() },
            );
    
    
            if is_mouse_button_pressed(MouseButton::Left) {
                for (i, _) in self.buttons.iter().enumerate() {
                    let rectangle = Rect::new(
                        buttons_start_position + ((i as f32) * (button_size + button_margin)), 
                        button_y, 
                        button_size, 
                        button_size, 
                    );
                    let (mouse_x,mouse_y) = mouse_position();
                    let rectangle_rect = Rect::new(mouse_x,mouse_y,1.0, 1.0);
        
                    if rectangle_rect.intersect(rectangle).is_some() {
                        self.buttons_state[i] = true;

                        if (i == 0) {
                            player_bus.call(playerbus::PlayerBusAction::PausePlay)
                        }

                        if (i == 1) {
                            player_bus.call(playerbus::PlayerBusAction::NextSong)
                        }
                    }
                }
           }
    
            for (i, button_name) in self.buttons.iter().enumerate() {
                if self.buttons_state[i] {
                    draw_rectangle(
                        buttons_start_position + ((i as f32) * (button_size + button_margin)), 
                        button_y, 
                        button_size, 
                        button_size, 
                        WHITE
                    );            
                }
    
                let button_center = get_text_center(button_name, Some(&font_icons), button_size as u16, 1.0, 0.0);
                
                draw_text_ex(
                    button_name,
                    buttons_start_position + ((i as f32) * (button_size + button_margin)) + button_size / 2.0 - button_center.x + 1.0,
                    button_y + button_size - 8.0 + 1.0,
                    TextParams {
                        font_size: button_size as u16,
                        font: Some(&font_icons),
                        color: BLACK,
                        ..Default::default()
                    },
                );
                draw_text_ex(
                    button_name,
                    buttons_start_position + ((i as f32) * (button_size + button_margin)) + button_size / 2.0 - button_center.x,
                    button_y + button_size - 8.0,
                    TextParams {
                        font_size: button_size as u16,
                        font: Some(&font_icons),
                        ..Default::default()
                    },
                );
            }
    
            for i in 0..self.buttons.len() { self.buttons_state[i] = false }
    
            next_frame().await;

            std::thread::sleep(Duration::from_millis(50));
        }
    }
}

