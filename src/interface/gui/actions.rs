use std::{fmt::Display, fs, process::Command};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::State;

use super::{Gui, Screen};

#[derive(Serialize, Deserialize)]
struct ActionCommand {
    program: String,
    args: Vec<String>,
}

impl Display for ActionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program: {}, args: {:?}", self.program, self.args)
    }
}

#[derive(Serialize, Deserialize)]
struct Action {
    label: String,
    command: ActionCommand,
}

#[derive(Serialize, Deserialize)]
pub struct Actions {
    actions: Vec<Action>
}

impl Actions {
    pub fn init(config_path: String) -> Self {
        let config_raw = fs::read_to_string(config_path).unwrap_or(serde_json::to_string(&Actions { actions: vec![] }).unwrap());
        serde_json::from_str(config_raw.as_str()).expect("Error on deserialize actions config file")
    }
}

impl Screen for Actions {
    fn render(&self, gui: &Gui) {
        let button_size = 48.0;

        for (i, action) in self.actions.iter().enumerate() {
            draw_rectangle(
                200.0,
                16.0 + ((i as f32) * button_size + (i as f32) * 16.0),
                624.0, 
                48.0, 
                WHITE
            );
            draw_rectangle(
                201.0,
                1.0+16.0 + ((i as f32) * button_size + (i as f32) * 16.0),
                622.0, 
                48.0-2.0, 
                BLACK
            );

            draw_text_ex(&action.label, 200.0 + 16.0, 16.0 + ((i as f32) * button_size + (i as f32) * 16.0) + 32.0,  TextParams { font_size: 24, font: Some(&gui.fonts.title), color: WHITE, ..Default::default() },);
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            for (i, action) in self.actions.iter().enumerate() {
                let rectangle = Rect::new(
                    200.0,
                    16.0 + ((i as f32) * button_size + (i as f32) * 16.0),
                    624.0, 
                    48.0,
                );
                let (mouse_x,mouse_y) = mouse_position();
                let rectangle_rect = Rect::new(mouse_x,mouse_y,1.0, 1.0);
    
                if rectangle_rect.intersect(rectangle).is_some() {
                    draw_rectangle(
                        200.0,
                        16.0 + ((i as f32) * button_size + (i as f32) * 16.0),
                        624.0, 
                        48.0, 
                        WHITE
                    );
                    match Command::new(action.command.program.as_str()).args(action.command.args.as_slice()).spawn() {
                        Ok(_) => info!("[Actions] Command {} executed with sucess", action.command),
                        Err(err) => error!("[Actions] Command {} executed with errors: {:?}", action.command, err),
                    }
                }
            }
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            let rectangle = Rect::new(
                16.0,
                16.0,
                button_size, 
                button_size, 
            );
            let (mouse_x,mouse_y) = mouse_position();
            let rectangle_rect = Rect::new(mouse_x,mouse_y,1.0, 1.0);

            if rectangle_rect.intersect(rectangle).is_some() {
                draw_rectangle(
                    16.0,
                    16.0,
                    button_size, 
                    button_size, 
                    WHITE
                );

                gui.player_bus.publish_message(crate::state::Message::UserClickBackToPlayer);
                return;
            }
        }

        let button_center = get_text_center("", Some(&gui.fonts.icons), button_size as u16, 1.0, 0.0);

        draw_text_ex(
            "",
            16.0 + button_center.x,
            48.0 + 8.0,
            TextParams {
                font_size: button_size as u16,
                font: Some(&gui.fonts.icons),
                ..Default::default()
            },
        );
    }
    
    fn update(&mut self, _: State) {
        
    }
    
    fn nav_id(&self) -> String {
        "/actions".to_owned()
    }

    fn on_show(&mut self) {}
}