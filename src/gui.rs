use std::time::{Duration, Instant};
use macroquad::prelude::*;

use crate::{discovery::DiscoveryStore, playerbus::{self, PlayerBus, PlayerStateCase, State, TrackState}};

pub struct Gui {
    player_bus: PlayerBus,
    state: State,
    buttons: Buttons,
    fonts: Fonts,
    cover_foreground: Texture2D,
    cover_background: Texture2D,
}

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
            PlayerStateCase::Loading => "".to_string(),
        }
    }
    
    fn action(&self, _: State) {
        self.player_bus.call(playerbus::PlayerBusAction::PausePlay)
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
        self.player_bus.call(playerbus::PlayerBusAction::NextSong)
    }
}

struct TrackRadioButton {
    player_bus: PlayerBus,
    discovery_store: DiscoveryStore,
}

impl TrackRadioButton {
    fn new(player_bus: PlayerBus, discovery_store: DiscoveryStore) -> Self {
        Self {
            player_bus,
            discovery_store,
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
        self.player_bus.call(playerbus::PlayerBusAction::PausePlay);
        let _ = self.discovery_store.discovery_radio(&state.track.unwrap().id);
        self.player_bus.call(playerbus::PlayerBusAction::NextSong);
    }
}

pub struct Buttons {
    buttons: Vec<Box<dyn Button>>,
    size: f32,
    margin: f32,
}

impl Buttons {
    fn init(buttons: Vec<Box<dyn Button>>) -> Self {
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

pub struct Fonts{
    title: Font,
    subtitle: Font,
    icons: Font,
}

impl Gui {
    pub fn init(player_bus: PlayerBus, discovery_store: DiscoveryStore) -> Gui {
        let state = State::default_state();

        let buttons = Buttons::init(vec![
            Box::new(PlayPauseButton::new(player_bus.clone())),
            Box::new(NextButton::new(player_bus.clone())),
            Box::new(TrackRadioButton::new(player_bus.clone(), discovery_store.clone())),
        ]);

        let fonts = Fonts {
            title: load_ttf_font_from_bytes(include_bytes!("../static/NotoSans_Condensed-SemiBold.ttf")).unwrap(),
            subtitle: load_ttf_font_from_bytes(include_bytes!("../static/NotoSans_Condensed-Light.ttf")).unwrap(),
            icons: load_ttf_font_from_bytes(include_bytes!("../static/fontello.ttf")).unwrap(),
        };

        let cover_foreground: Texture2D = Texture2D::from_file_with_format(include_bytes!("../static/sample_cover.jpg-foreground.png"), Some(ImageFormat::Png));
        let cover_background: Texture2D = Texture2D::from_file_with_format(include_bytes!("../static/sample_cover.jpg-background.png"), Some(ImageFormat::Png));

        Gui { 
            player_bus,
            state, 
            buttons,
            fonts,
            cover_foreground,
            cover_background,
        }
    }

    fn duration_formated(duration: &Duration) -> String {
        let seconds = duration.as_secs() % 60;
        let minutes = (duration.as_secs() / 60) % 60;
        format!("{}:{:0>2}", minutes, seconds)
    }

    async fn update_state(&mut self) {
        let new_state = self.player_bus.read_state();
        if new_state.is_some() {
            self.state = new_state.unwrap();
            if self.state.track.is_some() {
                let track = self.state.track.clone().unwrap();
                if track.cover.is_some() {
                    self.cover_foreground = load_texture(track.cover.clone().unwrap().foreground.clone().as_str()).await.unwrap();
                    self.cover_background = load_texture(track.cover.clone().unwrap().background.clone().as_str()).await.unwrap();
                }
            }
        }
    }

    pub async fn gui_loop(&mut self) {
        loop {
            self.update_state().await;
            
            self.render();
    
            next_frame().await;

            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn render_covers(&self) {
        draw_texture_ex(&self.cover_background, 0.0, -212.0, WHITE, DrawTextureParams {
            rotation: 0.0,
            ..Default::default()
        });

        draw_texture_ex(&self.cover_foreground, screen_width() / 2.0 - 160.0, 112.0, WHITE, DrawTextureParams {
            rotation: 0.0,
            ..Default::default()
        });
    }

    fn render_title(&self, track: TrackState) {
        draw_text_ex(&track.title, 17.0, 41.0,  TextParams { font_size: 32, font: Some(&self.fonts.title), color: BLACK, ..Default::default() },);
        draw_text_ex(&track.title, 16.0, 40.0,  TextParams { font_size: 32, font: Some(&self.fonts.title), color: WHITE, ..Default::default() },);
        
        draw_text_ex(format!("{} - {}", track.artist_name, track.album_name).as_str(), 17.0, 73.0, TextParams { font_size: 24, font: Some(&self.fonts.subtitle), color: BLACK, ..Default::default() },);
        draw_text_ex(format!("{} - {}", track.artist_name, track.album_name).as_str(), 16.0, 72.0, TextParams { font_size: 24, font: Some(&self.fonts.subtitle), color: WHITE, ..Default::default() },);
    }

    fn render_progress(&mut self, track: TrackState) {
        let time_duration_actual = Instant::now() - self.state.player.playing_time.unwrap();
        let seconds = time_duration_actual.as_secs() % 60;
        let minutes = (time_duration_actual.as_secs() / 60) % 60;
        let time_text_actual = format!("{}:{:0>2}", minutes, seconds);

        let time_text_end = Self::duration_formated(&self.state.track.clone().unwrap().duration);
        let time_text_font_size = 16;

        let time_percentage = time_duration_actual.as_secs_f32() / track.duration.as_secs_f32();
        let time_text_center = get_text_center(time_text_end.as_str(), Some(&self.fonts.icons), time_text_font_size, 1.0, 0.0);

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
            TextParams { font_size: time_text_font_size, font: Some(&self.fonts.subtitle), color: WHITE, ..Default::default() },
        );

        draw_text_ex(
            time_text_end.as_str(), 
            buttons_start_position + buttons_widget_width + 72.0,
            button_y - 16.0 - time_text_center.y - 1.0, 
            TextParams { font_size: time_text_font_size, font: Some(&self.fonts.subtitle), color: WHITE, ..Default::default() },
        );
    }

    fn render_buttons(&mut self) {
        let button_size = self.buttons.size;
        let button_margin = self.buttons.margin;
        let buttons_start_position = self.buttons.widget_x();
        let button_y = self.buttons.widget_y();

        if is_mouse_button_pressed(MouseButton::Left) {
            for (i, button) in self.buttons.buttons.iter_mut().enumerate() {
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
            let button_center = get_text_center(button.label(self.state.clone()).as_str(), Some(&self.fonts.icons), button_size as u16, 1.0, 0.0);
            
            draw_text_ex(
                button.label(self.state.clone()).as_str(),
                buttons_start_position + ((i as f32) * (button_size + button_margin)) + button_size / 2.0 - button_center.x + 1.0,
                button_y + button_size - 8.0 + 1.0,
                TextParams {
                    font_size: button_size as u16,
                    font: Some(&self.fonts.icons),
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
                    font: Some(&self.fonts.icons),
                    ..Default::default()
                },
            );
        }
    }

    fn render_loading(&self) {
        draw_text_ex("Loading...", 17.0, 41.0,  TextParams { font_size: 32, font: Some(&self.fonts.title), color: BLACK, ..Default::default() },);
        draw_text_ex("Loading...", 16.0, 40.0,  TextParams { font_size: 32, font: Some(&self.fonts.title), color: WHITE, ..Default::default() },);
    }

    fn render(&mut self) {
        clear_background(BLACK);
    
        self.render_covers();

        match self.state.player.case {
            PlayerStateCase::Loading => self.render_loading(),
            _ => {
                if self.state.track.is_some() {
                    self.render_title(self.state.track.clone().unwrap());
                    self.render_progress(self.state.track.clone().unwrap());
                }
            }
        }

        self.render_buttons();
    }
}

