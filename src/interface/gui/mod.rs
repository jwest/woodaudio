use std::time::Duration;
use image::Rgb;
use qrcode::QrCode;
use slint::{Image, LogicalSize, Rgb8Pixel, SharedPixelBuffer, WindowSize};

use crate::state::{BackendState, PlayerBus};

slint::include_modules!();

pub struct Gui {
    player_bus: PlayerBus,
    ui: AppWindow,
}

fn duration_formated(duration: &Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    format!("{minutes}:{seconds:0>2}")
}

impl Gui {
    pub fn init(player_bus: PlayerBus) -> Gui {
        let ui = AppWindow::new().unwrap();
        ui.set_track_name("new track!".into());

        Self { player_bus, ui }
    }
    pub fn gui_loop(&mut self) {
        self.ui.window().set_size(WindowSize::Logical(LogicalSize::new(1024.0, 600.0)));

        let main_window_weak = self.ui.as_weak();
        let bus = self.player_bus.clone();

        self.ui.global::<Data>().on_request_new_value(move || {
            let current_state = bus.read_state().clone();

            let current_track_name = current_state.track.clone().map( |track| track.title).unwrap_or("Loading...".to_string());
            let current_artist_name = current_state.track.clone().map( |track| track.artist_name).unwrap_or("".to_string());
            let current_album_name = current_state.track.clone().map( |track| track.album_name).unwrap_or("".to_string());

            let current_track_duration = &current_state.track.clone().map( |track| track.duration).unwrap_or(Duration::ZERO);
            let current_duration = &current_state.player.playing_time.unwrap_or(Duration::ZERO);

            if let Some(handle) = main_window_weak.upgrade() {
                handle.global::<Data>().set_current_track_name(current_track_name.into());
                handle.global::<Data>().set_current_artist_name(current_artist_name.into());
                handle.global::<Data>().set_current_album_name(current_album_name.into());

                handle.global::<Data>().set_current_track_duration(duration_formated(current_track_duration).into());
                handle.global::<Data>().set_current_duration(duration_formated(current_duration).into());
                handle.global::<Data>().set_current_duration_percentage(current_duration.as_secs_f32() / current_track_duration.as_secs_f32());

                match current_state.backends.tidal {
                    BackendState::WaitingForLoginByLink(login_link) => {
                        handle.global::<Data>().set_is_session_exist(false);
                        handle.global::<Data>().set_session_code(login_link.clone().into());

                        let qrcode = QrCode::new(login_link).unwrap();
                        let image = qrcode.render::<Rgb<u8>>().build();
                        let pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                            image.as_raw(),
                            image.width(),
                            image.height(),
                        );

                        handle.global::<Data>().set_session_qrcode(Image::from_rgb8(pixel_buffer));
                    }
                    _ => {
                        handle.global::<Data>().set_is_session_exist(true);
                        handle.global::<Data>().set_session_code(String::new().into());
                        handle.global::<Data>().set_session_qrcode(Image::default());
                    }
                }
            }
        });

        self.ui.run().unwrap();
    }
}