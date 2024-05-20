use std::time::Duration;
use futures::executor;
use macroquad::prelude::*;

use crate::playerbus::{Message, PlayerBus, PlayerStateCase, State, TrackState};

use super::Screen;

pub trait Button {
    fn label(&self, state: State) -> String;
    fn action(&self, state: State);
}

struct PlayPauseButton {
    player_bus: PlayerBus,
}

impl PlayPauseButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for PlayPauseButton {
    fn label(&self, state: State) -> String { 
        match state.player.case {
            PlayerStateCase::Playing => "".to_string(),
            PlayerStateCase::Paused => "".to_string(),
            _ => "".to_string(),
        }
    }
    
    fn action(&self, state: State) {
        match state.player.case {
            PlayerStateCase::Playing => self.player_bus.publish_message(Message::UserPause),
            PlayerStateCase::Paused => self.player_bus.publish_message(Message::UserPlay),
            _ => {},
        }
    }
}

struct NextButton {
    player_bus: PlayerBus,
}

impl NextButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for NextButton {
    fn label(&self, _: State) -> String { 
        "".to_string()
    }
    
    fn action(&self, _: State) {
        self.player_bus.publish_message(Message::UserPlayNext);
    }
}

struct TrackRadioButton {
    player_bus: PlayerBus,
}

impl TrackRadioButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for TrackRadioButton {
    fn label(&self, _: State) -> String { 
        "".to_string()
    }
    
    fn action(&self, state: State) {
        if state.track.is_none() {
            return;
        }
        self.player_bus.publish_message(Message::UserLoadRadio);
    }
}

struct LikeButton {
    player_bus: PlayerBus,
}

impl LikeButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for LikeButton {
    fn label(&self, _: State) -> String { 
        "".to_string()
    }
    
    fn action(&self, state: State) {
        if state.track.is_none() {
            return;
        }
        self.player_bus.publish_message(Message::UserLike);
    }
}

struct ActionsButton {
    player_bus: PlayerBus,
}

impl ActionsButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for ActionsButton {
    fn label(&self, _: State) -> String { 
        "".to_string()
    }
    
    fn action(&self, _: State) {
        self.player_bus.publish_message(Message::UserClickActions);
    }
}

struct LikedAlbumsButton {
    player_bus: PlayerBus,
}

impl LikedAlbumsButton {
    fn new(player_bus: PlayerBus) -> Self {
        Self {
            player_bus,
        }
    }
}

impl Button for LikedAlbumsButton {
    fn label(&self, _: State) -> String {
        "".to_string()
    }

    fn action(&self, _: State) {
        self.player_bus.publish_message(Message::UserClickLikedAlbumsButton);
    }
}

pub struct Buttons {
    buttons: Vec<Box<dyn Button + Send>>,
    size: f32,
    margin: f32,
}

impl Buttons {
    fn init(buttons: Vec<Box<dyn Button + Send>>) -> Self {
        Self {
            buttons,
            size: 48.0,
            margin: 32.0,
        }
    }

    fn widget_width(&self) -> f32 {
        (self.buttons.len() as f32) * self.size + ((self.buttons.len() as f32)-1.0) * self.margin
    }

    fn widget_x(&self) -> f32 {
        screen_width() / 2.0 - self.widget_width() / 2.0
    }

    fn widget_y(&self) -> f32 {
        screen_height() - 48.0 - self.size
    }
}

pub struct Player {
    buttons: Buttons,
    state: State,
    cover_foreground_path: String,
    cover_foreground: Texture2D,
    cover_background_path: String,
    cover_background: Texture2D,
}

impl Player {
    pub fn init(player_bus: PlayerBus) -> Self {
        let buttons = Buttons::init(vec![
            Box::new(PlayPauseButton::new(player_bus.clone())),
            Box::new(NextButton::new(player_bus.clone())),
            Box::new(LikeButton::new(player_bus.clone())),
            Box::new(TrackRadioButton::new(player_bus.clone())),
            Box::new(ActionsButton::new(player_bus.clone())),
            Box::new(LikedAlbumsButton::new(player_bus.clone())),
        ]);

        let cover_foreground_path = "../static/sample_cover.jpg-foreground.png".to_string();
        let cover_foreground: Texture2D = Texture2D::from_file_with_format(include_bytes!("../../static/sample_cover.jpg-foreground.png"), Some(ImageFormat::Png));
        let cover_background_path = "../static/sample_cover.jpg-background.png".to_string();
        let cover_background: Texture2D = Texture2D::from_file_with_format(include_bytes!("../../static/sample_cover.jpg-background.png"), Some(ImageFormat::Png));

        Self {
            buttons,
            state: State::default_state(),
            cover_foreground_path,
            cover_foreground,
            cover_background_path,
            cover_background,
        }
    }
    fn render_covers(&self, _: &super::Gui) {
        draw_texture_ex(&self.cover_background, 0.0, -212.0, WHITE, DrawTextureParams {
            rotation: 0.0,
            ..Default::default()
        });

        draw_texture_ex(&self.cover_foreground, screen_width() / 2.0 - 160.0, 112.0, WHITE, DrawTextureParams {
            rotation: 0.0,
            ..Default::default()
        });
    }

    fn render_title(&self, track: TrackState, ui: &super::Gui) {
        draw_text_ex(&track.title, 17.0, 41.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: BLACK, ..Default::default() },);
        draw_text_ex(&track.title, 16.0, 40.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: WHITE, ..Default::default() },);
        
        draw_text_ex(format!("{} - {}", track.artist_name, track.album_name).as_str(), 17.0, 73.0, TextParams { font_size: 24, font: Some(&ui.fonts.subtitle), color: BLACK, ..Default::default() },);
        draw_text_ex(format!("{} - {}", track.artist_name, track.album_name).as_str(), 16.0, 72.0, TextParams { font_size: 24, font: Some(&ui.fonts.subtitle), color: WHITE, ..Default::default() },);
    }

    fn render_progress(&self, track: TrackState, ui: &super::Gui) {
        let time_duration_actual = self.state.player.playing_time.unwrap();
        let seconds = time_duration_actual.as_secs() % 60;
        let minutes = (time_duration_actual.as_secs() / 60) % 60;
        let time_text_actual = format!("{minutes}:{seconds:0>2}");

        let time_text_end = Self::duration_formated(&self.state.track.clone().unwrap().duration);
        let time_text_font_size = 16;

        let time_percentage = time_duration_actual.as_secs_f32() / track.duration.as_secs_f32();
        let time_text_center = get_text_center(time_text_end.as_str(), Some(&ui.fonts.icons), time_text_font_size, 1.0, 0.0);

        let buttons_start_position = self.buttons.widget_x();
        let button_y = self.buttons.widget_y();
        let buttons_widget_width = self.buttons.widget_width();

        draw_rectangle(
            buttons_start_position - 48.0, 
            button_y - 24.0, 
            buttons_widget_width + 48.0 + 48.0, 
            4.0, 
            GRAY
        );

        draw_rectangle(
            buttons_start_position - 48.0, 
            button_y - 24.0, 
            (buttons_widget_width + 48.0 + 48.0) * time_percentage, 
            4.0, 
            WHITE
        );

        draw_text_ex(
            time_text_actual.as_str(), 
            buttons_start_position - 96.0, 
            button_y - 16.0 - time_text_center.y - 1.0, 
            TextParams { font_size: time_text_font_size, font: Some(&ui.fonts.subtitle), color: WHITE, ..Default::default() },
        );

        draw_text_ex(
            time_text_end.as_str(), 
            buttons_start_position + buttons_widget_width + 72.0,
            button_y - 16.0 - time_text_center.y - 1.0, 
            TextParams { font_size: time_text_font_size, font: Some(&ui.fonts.subtitle), color: WHITE, ..Default::default() },
        );
    }

    fn render_buttons(&self, ui: &super::Gui) {
        let button_size = self.buttons.size;
        let button_margin = self.buttons.margin;
        let buttons_start_position = self.buttons.widget_x();
        let button_y = self.buttons.widget_y();

        if is_mouse_button_pressed(MouseButton::Left) {
            for (i, button) in self.buttons.buttons.iter().enumerate() {
                let rectangle = Rect::new(
                    buttons_start_position + ((i as f32) * (button_size + button_margin)), 
                    button_y, 
                    button_size, 
                    button_size, 
                );
                let (mouse_x,mouse_y) = mouse_position();
                let rectangle_rect = Rect::new(mouse_x,mouse_y,1.0, 1.0);
    
                if rectangle_rect.intersect(rectangle).is_some() {
                    draw_rectangle(
                        buttons_start_position + ((i as f32) * (button_size + button_margin)), 
                        button_y, 
                        button_size, 
                        button_size, 
                        WHITE
                    );
                    button.action(self.state.clone());
                }
            }
        }

        for (i, button) in self.buttons.buttons.iter().enumerate() {
            let button_center = get_text_center(button.label(self.state.clone()).as_str(), Some(&ui.fonts.icons), button_size as u16, 1.0, 0.0);
            
            draw_text_ex(
                button.label(self.state.clone()).as_str(),
                buttons_start_position + ((i as f32) * (button_size + button_margin)) + button_size / 2.0 - button_center.x + 1.0,
                button_y + button_size - 8.0 + 1.0,
                TextParams {
                    font_size: button_size as u16,
                    font: Some(&ui.fonts.icons),
                    color: BLACK,
                    ..Default::default()
                },
            );
            draw_text_ex(
                button.label(self.state.clone()).as_str(),
                buttons_start_position + ((i as f32) * (button_size + button_margin)) + button_size / 2.0 - button_center.x,
                button_y + button_size - 8.0,
                TextParams {
                    font_size: button_size as u16,
                    font: Some(&ui.fonts.icons),
                    ..Default::default()
                },
            );
        }
    }

    fn render_loading(&self, ui: &super::Gui) {
        draw_text_ex("Loading...", 17.0, 41.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: BLACK, ..Default::default() },);
        draw_text_ex("Loading...", 16.0, 40.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: WHITE, ..Default::default() },);
    }

    fn duration_formated(duration: &Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        format!("{minutes}:{seconds:0>2}")
    }
}

impl Screen for Player {
    fn nav_id(&self) -> String {
        "/player".to_owned()
    }

    fn on_show(&mut self) {}

    fn update(&mut self, state: State) {
        self.state = state.clone();

        if state.track.is_some() {
            let track = state.track.clone().unwrap();
            if track.cover.foreground.is_some() && !track.cover.clone().foreground.unwrap().eq(self.cover_foreground_path.as_str()) {
                self.cover_foreground_path = track.cover.clone().foreground.unwrap().clone();
                self.cover_foreground = executor::block_on(load_texture(self.cover_foreground_path.as_str())).unwrap();
            }
            if track.cover.background.is_some() && !track.cover.clone().background.unwrap().eq(self.cover_background_path.as_str()) {
                self.cover_background_path = track.cover.clone().background.unwrap().clone();
                self.cover_background = executor::block_on(load_texture(self.cover_background_path.as_str())).unwrap();
            }
        }
    }

    fn render(&self, ui: &super::Gui) {
        self.render_covers(ui);

        match ui.state.player.case {
            PlayerStateCase::Loading => self.render_loading(ui),
            _ => {
                if let Some(track_state) = &self.state.track {
                    self.render_title(track_state.clone(), ui);
                    self.render_progress(track_state.clone(), ui);
                }
            }
        }

        self.render_buttons(ui);
    }
}