use std::sync::{Arc, Mutex};
use std::time::Duration;
use macroquad::prelude::*;

use crate::config::Config;
use crate::playerbus::{BroadcastChannel, Command, PlayerBus, State};

use self::actions::Actions;
use self::player::Player;
use self::session::SessionGui;

pub mod session;
pub mod actions;
pub mod player;

pub trait Screen {
    fn nav_id(&self) -> String;
    fn update(&mut self, state: State);
    fn render(&self, ui: &Gui);
}

#[derive(Clone)]
pub struct ScreenRegistry {
    path: String,
    active: usize,
    screens: Vec<Arc<Mutex<Box<dyn Screen>>>>,
}

impl ScreenRegistry {
    fn init(screens: Vec<Box<dyn Screen>>) -> Self {
        Self {
            path: screens[0].nav_id(),
            active: 0,
            screens: screens.into_iter().map(|b| Arc::new(Mutex::new(b))).collect()
        }
    }
    fn navigate(&mut self, path: String) {
        match self.screens.iter()
            .enumerate()
            .find(|(_, screen)| screen.lock().unwrap().nav_id().eq(&path)) {
                Some((i, screen)) => {
                    self.path = path;
                    self.active = i;
                    debug!("[ScreenRegistry] navigated to {}, {:?}", i, screen.lock().unwrap().nav_id());
                },
                None => error!("[ScreenRegistry] Screen not found: {}", path),
            }
    }
    fn update(&mut self, state: State) {
        self.screens[self.active].lock().unwrap().update(state);
    }
    fn render(&self, ui: &Gui) {
        self.screens[self.active].lock().unwrap().render(ui);
    }
}

pub struct Gui {
    player_bus: PlayerBus,
    channel: BroadcastChannel,
    screen_registry: ScreenRegistry,
    state: State,
    fonts: Fonts,
}

pub struct Fonts{
    title: Font,
    subtitle: Font,
    icons: Font,
}

impl Gui {
    pub fn init(mut player_bus: PlayerBus, config: Config) -> Gui {
        let state = State::default_state();

        let channel = player_bus.register_command_channel(vec!["ShowScreen".to_string()]);

        let fonts = Fonts {
            title: load_ttf_font_from_bytes(include_bytes!("../../static/NotoSans_Condensed-SemiBold.ttf")).unwrap(),
            subtitle: load_ttf_font_from_bytes(include_bytes!("../../static/NotoSans_Condensed-Light.ttf")).unwrap(),
            icons: load_ttf_font_from_bytes(include_bytes!("../../static/fontello.ttf")).unwrap(),
        };

        Gui { 
            player_bus: player_bus.clone(),
            channel,
            state,
            screen_registry: ScreenRegistry::init(vec![
                Box::new(SessionGui::init(config)),
                Box::new(Player::init(player_bus)),
                Box::new(Actions::init(home::home_dir().unwrap().join("actions.json").to_str().unwrap().to_string())),
            ]),
            fonts,
        }
    }

    async fn update_state(&mut self) {
        let new_state = self.player_bus.read_state();
        self.state = new_state;
    }

    fn render_screen(&mut self) {
        clear_background(BLACK);
    
        let command = self.channel.read_command();
        match command {
            Some(Command::ShowScreen(path)) => {
                self.screen_registry.navigate(path);
            },
            _ => {}
        }

        self.screen_registry.update(self.state.clone());
        self.screen_registry.render(self);
    }

    pub async fn gui_loop(&mut self) {
        loop {
            self.update_state().await;
            self.render_screen();
            
            next_frame().await;
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
