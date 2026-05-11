use std::time::Duration;
use image::Rgb;
use image::io::Reader;
use qrcode::QrCode;
use slint::{Image, LogicalSize, Rgb8Pixel, SharedPixelBuffer, WindowSize};
use crate::config::Config;
use crate::state::{BackendState, Command, PlayerBus, PlayerStateCase};

slint::include_modules!();

pub struct Gui {
    config: Config,
    player_bus: PlayerBus,
    ui: AppWindow,
}

fn duration_formated(duration: &Duration) -> String {
    let seconds = duration.as_secs() % 60;
    let minutes = (duration.as_secs() / 60) % 60;
    format!("{minutes}:{seconds:0>2}")
}

fn load_image_from_path(path: &str) -> Option<Image> {
    let image = Reader::open(path).ok()?.with_guessed_format().ok()?.decode().ok()?;
    let rgb = image.to_rgb8();
    let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
    );
    Some(Image::from_rgb8(buffer))
}

impl Gui {
    pub fn init(config: Config, player_bus: PlayerBus) -> Gui {
        let ui = AppWindow::new().unwrap();
        Self { config, player_bus, ui }
    }
    pub fn gui_loop(&mut self) {
        let gui_config = self.config.gui.clone();
        self.ui.window().set_size(WindowSize::Logical(LogicalSize::new(gui_config.window_x as f32, gui_config.window_y as f32)));

        let main_window_weak = self.ui.as_weak();
        let bus = self.player_bus.clone();

        let request_next_bus = bus.clone();
        self.ui.global::<Data>().on_request_next_track(move || {
            request_next_bus.publish_command(Command::Next);
        });

        // Cache: last decoded cover paths and their Slint Images
        let mut cached_bg_path: Option<String> = None;
        let mut cached_fg_path: Option<String> = None;
        let mut cached_bg_image: Image = Image::default();
        let mut cached_fg_image: Image = Image::default();

        self.ui.global::<Data>().on_request_new_value(move || {
            let current_state = bus.read_state();

            let is_loading = matches!(current_state.player.case, PlayerStateCase::Loading);

            let current_track_name = current_state.track.as_ref().map(|t| t.title.clone()).unwrap_or_else(|| "Loading...".to_string());
            let current_artist_name = current_state.track.as_ref().map(|t| t.artist_name.clone()).unwrap_or_default();
            let current_album_name = current_state.track.as_ref().map(|t| t.album_name.clone()).unwrap_or_default();

            let current_cover_foreground = current_state.track.as_ref().and_then(|t| t.cover.foreground.clone());
            let current_cover_background = current_state.track.as_ref().and_then(|t| t.cover.background.clone());

            let current_track_duration = current_state.track.as_ref().map(|t| t.duration).unwrap_or(Duration::ZERO);
            let current_duration = current_state.player.playing_time.unwrap_or(Duration::ZERO);

            // Reload background only when path changes
            match &current_cover_background {
                Some(path) if cached_bg_path.as_deref() != Some(path.as_str()) => {
                    cached_bg_image = load_image_from_path(path).unwrap_or_default();
                    cached_bg_path = Some(path.clone());
                }
                None => {
                    cached_bg_image = Image::default();
                    cached_bg_path = None;
                }
                _ => {} // same path — reuse cached image
            }

            // Reload foreground only when path changes
            match &current_cover_foreground {
                Some(path) if cached_fg_path.as_deref() != Some(path.as_str()) => {
                    cached_fg_image = load_image_from_path(path).unwrap_or_default();
                    cached_fg_path = Some(path.clone());
                }
                None => {
                    cached_fg_image = Image::default();
                    cached_fg_path = None;
                }
                _ => {} // same path — reuse cached image
            }

            if let Some(handle) = main_window_weak.upgrade() {
                handle.global::<Data>().set_window_x(gui_config.window_x as i32);
                handle.global::<Data>().set_window_y(gui_config.window_y as i32);
                handle.global::<Data>().set_is_loading(is_loading);

                handle.global::<Data>().set_current_track_name(current_track_name.into());
                handle.global::<Data>().set_current_artist_name(current_artist_name.into());
                handle.global::<Data>().set_current_album_name(current_album_name.into());

                handle.global::<Data>().set_current_track_duration(duration_formated(&current_track_duration).into());
                handle.global::<Data>().set_current_duration(duration_formated(&current_duration).into());
                handle.global::<Data>().set_current_duration_percentage(current_duration.as_secs_f32() / current_track_duration.as_secs_f32());

                handle.global::<Data>().set_current_cover_background(cached_bg_image.clone());
                handle.global::<Data>().set_current_cover_foreground(cached_fg_image.clone());

                match current_state.backends.tidal {
                    BackendState::WaitingForLoginByLink(ref login_link) => {
                        handle.global::<Data>().set_is_session_exist(false);
                        handle.global::<Data>().set_session_code(login_link.clone().into());

                        let qrcode = QrCode::new(login_link.as_str()).unwrap();
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
