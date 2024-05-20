use futures::StreamExt;
use log::{debug, error};
use macroquad::color::{BLACK, Color, WHITE, YELLOW};
use macroquad::input::{is_mouse_button_pressed, mouse_position, MouseButton};
use macroquad::math::Rect;
use macroquad::prelude::{draw_rectangle, draw_text_ex, draw_texture_ex, DrawTextureParams, get_text_center, ImageFormat, is_mouse_button_down, screen_width, TextParams, Texture2D};
use crate::gui::{Gui, Screen};
use crate::playerbus::State;

#[derive(Clone)]
#[derive(Debug)]
struct List {
    items: Vec<Option<Item>>,
    current: usize,
}

impl List {
    pub fn init(items: Vec<Option<Item>>) -> Self {
        Self {
            items,
            current: 0,
        }
    }

    pub fn next(&mut self) {
        if self.current < self.items.len() {
            self.current = self.current + 1;
        } else {
            self.current = self.items.len() - 1;
        }
    }

    pub fn prev(&mut self) {
        if self.current > 0 {
            self.current = self.current - 1;
        } else {
            self.current = 0;
        }
    }

    fn get(&self, i: i32) -> Option<Item> {
        if i >= 0 && i < self.items.len() as i32 {
            let item = self.items[i as usize].clone();
            debug!("[Browser gui] i: {:?}, item: {:?}", i as usize, item);
            item
        } else {
            debug!("[Browser gui] i: {:?}, item: None", i);
            None
        }
    }

    pub(crate) fn is_first(&self) -> bool {
        self.current == 0
    }

    pub(crate) fn is_last(&self) -> bool {
        if self.items.is_empty() {
            true
        } else {
            self.current == self.items.len() - 1
        }
    }

    pub fn get_prev_2(&self) -> Option<Item> { self.get(self.current as i32 - 2) }
    pub fn get_prev(&self) -> Option<Item> { self.get(self.current as i32 - 1) }
    pub fn get_current(&self) -> Option<Item> { self.get(self.current as i32) }
    pub fn get_next(&self) -> Option<Item> { self.get(self.current as i32 + 1) }
    pub fn get_next_2(&self) -> Option<Item> { self.get(self.current as i32 + 2) }
}

fn mock_item(i: i32) -> Option<Item> {
    Some(Item {
        title: format!("{} - {}", "Comfortably numb".to_string(), i),
        artist: format!("{} - {}", "Pink Floyd".to_string(), i),
        cover: Some(Cover {
            path: "../static/sample_cover.jpg-foreground.png".to_string(),
            texture: Texture2D::from_file_with_format(include_bytes!("../../static/sample_cover.jpg-foreground.png"), Some(ImageFormat::Png)),
        })
    })
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Cover {
    path: String,
    texture: Texture2D,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Item {
    artist: String,
    title: String,
    cover: Option<Cover>,
}

#[derive(Clone)]
#[derive(Debug)]
struct GuiState {
    carousel_position_y_correction: f32,
    last_mouse_position: (f32, f32),
    current_mouse_position: (f32, f32),
    last_step_clicked: bool,
    acceleration_after_swype: f32,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Browse {
    items: List,
    gui_state: GuiState,
}

impl Browse {
    pub fn init() -> Self {
        Self {
            items: List::init(vec![]),
            gui_state: GuiState {
                carousel_position_y_correction: 0.0,
                last_mouse_position: (0.0, 0.0),
                current_mouse_position: (0.0, 0.0),
                last_step_clicked: false,
                acceleration_after_swype: 0.0,
            }
        }
    }

    fn back_button(ui: &&Gui) {
        let button_size = 48.0;
        let button_center = get_text_center("", Some(&ui.fonts.icons), button_size as u16, 1.0, 0.0);

        if is_mouse_button_pressed(MouseButton::Left) {
            let rectangle = Rect::new(
                16.0,
                16.0,
                button_size,
                button_size,
            );
            let (mouse_x, mouse_y) = mouse_position();
            let rectangle_rect = Rect::new(mouse_x, mouse_y, 1.0, 1.0);

            if rectangle_rect.intersect(rectangle).is_some() {
                draw_rectangle(
                    16.0,
                    16.0,
                    button_size,
                    button_size,
                    WHITE
                );

                ui.player_bus.publish_message(crate::playerbus::Message::UserClickBackToPlayer);
                return;
            }
        }

        draw_text_ex(
            "",
            16.0 + button_center.x,
            48.0 + 8.0,
            TextParams {
                font_size: button_size as u16,
                font: Some(&ui.fonts.icons),
                ..Default::default()
            },
        );
    }

    fn render_item(ui: &Gui, item: Option<Item>, width_correction: f32, current: bool) {
        if item.is_some() {
            if item.clone().unwrap().cover.is_some() {
                draw_texture_ex(
                    &item.clone().unwrap().cover.unwrap().texture,
                    screen_width() / 2.0 - 160.0 + width_correction,
                    96.0, WHITE,
                    DrawTextureParams {
                        rotation: 0.0,
                        ..Default::default()
                    });
            }

            if !current {
                draw_rectangle(
                    screen_width() / 2.0 - 168.0 + width_correction,
                    88.0,
                    336.0,
                    336.0,
                    Color::new(0.00, 0.00, 0.00, 0.6)
                );
            }

            let center_artist = get_text_center(item.clone().unwrap().artist.as_str(), Some(&ui.fonts.title), 32, 1.0, 0.0);
            draw_text_ex(item.clone().unwrap().artist.as_str(), screen_width() / 2.0 - center_artist.x + width_correction, 480.0,  TextParams { font_size: 32, font: Some(&ui.fonts.title), color: WHITE, ..Default::default() },);

            let center_title = get_text_center(item.clone().unwrap().title.as_str(), Some(&ui.fonts.subtitle), 24, 1.0, 0.0);
            draw_text_ex(item.clone().unwrap().title.as_str(), screen_width() / 2.0 - center_title.x + width_correction, 512.0, TextParams { font_size: 24, font: Some(&ui.fonts.subtitle), color: WHITE, ..Default::default() },);
        }
    }
}
impl Screen for Browse {
    fn nav_id(&self) -> String {
        "/browse".to_owned()
    }

    fn on_show(&mut self) {
        let items = (1..100).into_iter().map(|i| mock_item(i)).collect();
        self.items = List::init(items);

        self.gui_state = GuiState {
            carousel_position_y_correction: 0.0,
            last_mouse_position: (0.0, 0.0),
            current_mouse_position: (0.0, 0.0),
            last_step_clicked: false,
            acceleration_after_swype: 0.0,
        };
    }

    fn update(&mut self, _: State) {
        self.gui_state.last_mouse_position = self.gui_state.current_mouse_position;
        self.gui_state.current_mouse_position = mouse_position();

        if self.gui_state.carousel_position_y_correction >= 320.0 + 96.0 {
            if !self.items.is_first() {
                self.items.prev();
                self.gui_state.carousel_position_y_correction = self.gui_state.carousel_position_y_correction - 416.0;
            } else {
                self.gui_state.carousel_position_y_correction = 0.0;
            }
        }

        if self.gui_state.carousel_position_y_correction <= -320.0 - 96.0 {
            if !self.items.is_last() {
                self.items.next();
                self.gui_state.carousel_position_y_correction = self.gui_state.carousel_position_y_correction + 416.0;
            } else {
                self.gui_state.carousel_position_y_correction = 0.0;
            }
        }

        let step = self.gui_state.current_mouse_position.0 - self.gui_state.last_mouse_position.0;
        if is_mouse_button_down(MouseButton::Left) && step != 0.0 {
            if !(self.items.is_first() && self.gui_state.carousel_position_y_correction > 0.0 && step > 0.0) && !(self.items.is_last() && self.gui_state.carousel_position_y_correction < 0.0 && step < 0.0) {
                self.gui_state.carousel_position_y_correction = self.gui_state.carousel_position_y_correction + step;
                self.gui_state.acceleration_after_swype = step;
            }
        } else if (self.gui_state.last_step_clicked && self.gui_state.last_mouse_position != self.gui_state.current_mouse_position) || self.gui_state.acceleration_after_swype != 0.0 { //&& (self.gui_state.carousel_position_y_correction > 30.0 || self.gui_state.carousel_position_y_correction < -30.0) {
            if self.gui_state.acceleration_after_swype == 0.0 {
                self.gui_state.acceleration_after_swype = self.gui_state.current_mouse_position.0 - self.gui_state.last_mouse_position.0;
            }

            if (self.items.is_first() && self.gui_state.acceleration_after_swype > 0.0) || (self.items.is_last() && self.gui_state.acceleration_after_swype < 0.0) {
                self.gui_state.acceleration_after_swype = 0.0;
            }

            self.gui_state.carousel_position_y_correction = self.gui_state.carousel_position_y_correction + self.gui_state.acceleration_after_swype;
            self.gui_state.acceleration_after_swype = if self.gui_state.acceleration_after_swype > 0.0 {
                if self.gui_state.acceleration_after_swype <= 2.0 {
                    0.0
                } else {
                    self.gui_state.acceleration_after_swype - 2.0
                }
            } else {
                if self.gui_state.acceleration_after_swype >= -2.0 {
                    0.0
                } else {
                    self.gui_state.acceleration_after_swype + 2.0
                }
            };
        } else if self.gui_state.carousel_position_y_correction != 0.0 {
            let t = self.gui_state.carousel_position_y_correction / 416.0;
            if self.gui_state.carousel_position_y_correction < 0.0 {
                if self.gui_state.carousel_position_y_correction > -1.0 {
                    self.gui_state.carousel_position_y_correction = 0.0
                } else if !self.items.is_last() {
                    self.gui_state.carousel_position_y_correction = 416.0 * -(t / 1.5);
                }
            }
            if self.gui_state.carousel_position_y_correction > 0.0 {
                if self.gui_state.carousel_position_y_correction < 1.0 {
                    self.gui_state.carousel_position_y_correction = 0.0
                } else if !self.items.is_first() {
                    self.gui_state.carousel_position_y_correction = 416.0 * t / 1.5;
                }
            }
        } else if is_mouse_button_down(MouseButton::Left) && !self.gui_state.last_step_clicked {
            let (mouse_x,mouse_y) = mouse_position();
            let rectangle_rect = Rect::new(mouse_x,mouse_y,1.0, 1.0);

            let prev_rect = Rect::new(
                screen_width() / 2.0 - 160.0 - 320.0 - 96.0 + self.gui_state.carousel_position_y_correction,
                96.0,
                320.0,
                320.0,
            );

            if rectangle_rect.intersect(prev_rect).is_some() {
                self.gui_state.carousel_position_y_correction = 416.0;
            }

            let next_rect = Rect::new(
                screen_width() / 2.0 - 160.0 + 320.0 + 96.0 + self.gui_state.carousel_position_y_correction,
                96.0,
                320.0,
                320.0,
            );

            if rectangle_rect.intersect(next_rect).is_some() {
                self.gui_state.carousel_position_y_correction = -416.0;
            }
        }

        self.gui_state.last_step_clicked = is_mouse_button_down(MouseButton::Left);
    }

    fn render(&self, ui: &Gui) {
        Self::back_button(&ui);

        Self::render_item(ui, self.items.get_prev_2(), -320.0 - 96.0 - 320.0 - 96.0 + self.gui_state.carousel_position_y_correction, false);
        Self::render_item(ui, self.items.get_prev(), -320.0 - 96.0 + self.gui_state.carousel_position_y_correction, false);
        Self::render_item(ui, self.items.get_current(), 0.0 + self.gui_state.carousel_position_y_correction, true);
        Self::render_item(ui, self.items.get_next(), 320.0 + 96.0 + self.gui_state.carousel_position_y_correction, false);
        Self::render_item(ui, self.items.get_next_2(), 320.0 + 96.0 + 320.0 + 96.0 + self.gui_state.carousel_position_y_correction, false);
    }
}