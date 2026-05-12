use std::path::PathBuf;

use ini::{Ini, Properties};

const TIDAL_CLIENT_ID: &str = "4N3n6Q1x95LL5K7p";
const TIDAL_CLIENT_SECRET: &str = "oKOXfJW371cX6xaZ0PyhgGNBdNLlBZd4AKKYougMjik";

trait ParseIni {
    fn get_string(&self, name: &str) -> String;
    fn get_string_with_default(&self, name: &str, default: &str) -> String;
    fn get_bool(&self, name: &str) -> bool; 
    fn get_bool_with_default(&self, name: &str, default: bool) -> bool;
    fn get_u16_with_default(&self, name: &str, default: u16) -> u16;
}

impl ParseIni for Option<&Properties> {
    fn get_string(&self, name: &str) -> String {
        self.map(|properties| properties.get(name)).flatten().unwrap_or("").to_string()
    }
    fn get_string_with_default(&self, name: &str, default: &str) -> String {
        self.map(|properties| properties.get(name)).flatten().unwrap_or(default).to_string()   
    }
    fn get_bool(&self, name: &str) -> bool {
        matches!(self.get_string(name).as_str(), "true")
    }
    fn get_bool_with_default(&self, name: &str, default: bool) -> bool {
        match self.get_string(name).as_str() {
            "true" => true,
            "false" => false,
            _ => default,
        }
    }
    fn get_u16_with_default(&self, name: &str, default: u16) -> u16 {
        self.map(|properties| properties.get(name)).flatten()
            .map(|val| val.parse::<u16>().unwrap_or(default))
            .unwrap_or(default)
    }
}

fn bool_to_string(value: bool) -> String {
    if value { "true".to_string() } else { "false".to_string() }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Tidal {
    pub client_id: String,
    pub client_secret: String,
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub audio_quality: String,
}

impl Tidal {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("Tidal"));
        Self {
            client_id: properties.get_string_with_default("client_id", TIDAL_CLIENT_ID),
            client_secret: properties.get_string_with_default("client_secret", TIDAL_CLIENT_SECRET),
            token_type: properties.get_string_with_default("token_type", "Bearer"),
            access_token: properties.get_string("access_token"),
            refresh_token: properties.get_string("refresh_token"),
            audio_quality: properties.get_string_with_default("audio_quality", "LOSSLESS"),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("Tidal"))
            .set("client_id", self.client_id.clone())
            .set("client_secret", self.client_secret.clone())
            .set("token_type", self.token_type.clone())
            .set("access_token", self.access_token.clone())
            .set("refresh_token", self.refresh_token.clone())
            .set("audio_quality", self.audio_quality.clone());
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Player {
    pub without_cold_start: bool,
}

impl Player {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("Player"));
        Self {
            without_cold_start: properties.get_bool_with_default("without_cold_start", false),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("Player"))
            .set("without_cold_start", bool_to_string(self.without_cold_start));
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Gui {
    pub enabled: bool,
    pub window_x: u16,
    pub window_y: u16,
    pub display_cover_background: bool,
    pub display_cover_foreground: bool,
}

impl Gui {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("GUI"));
        Self {
            enabled: properties.get_bool_with_default("enabled", true),
            window_x: properties.get_u16_with_default("window_x", 800),
            window_y: properties.get_u16_with_default("window_y", 480),
            display_cover_background: properties.get_bool_with_default("display_cover_background", true),
            display_cover_foreground: properties.get_bool_with_default("display_cover_foreground", true),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("GUI"))
            .set("enabled", bool_to_string(self.enabled))
            .set("window_x", self.window_x.to_string())
            .set("window_y", self.window_y.to_string())
            .set("display_cover_background", bool_to_string(self.display_cover_background))
            .set("display_cover_foreground", bool_to_string(self.display_cover_foreground));
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Gpio {
    pub enabled: bool,
    pub next_song_pin: u16,
    pub like_song_pin: u16,
}

impl Gpio {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("GPIO"));
        Self {
            enabled: properties.get_bool_with_default("enabled", true),
            next_song_pin: properties.get_u16_with_default("next_song_pin", 12),
            like_song_pin: properties.get_u16_with_default("like_song_pin", 16),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("GPIO"))
            .set("enabled", bool_to_string(self.enabled))
            .set("next_song_pin", self.next_song_pin.to_string())
            .set("like_song_pin", self.like_song_pin.to_string());
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ExporterFTP {
    pub enabled: bool,
    pub server: String,
    pub share: String,
    pub password: String,
    pub username: String,
    pub cache_read: bool,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ExporterFile {
    pub enabled: bool,
    pub path: String,
}

impl crate::config::ExporterFile {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("ExporterFile"));
        Self {
            enabled: properties.get_bool("enabled"),
            path: properties.get_string("path"),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("ExporterFile"))
            .set("enabled", bool_to_string(self.enabled))
            .set("path", self.path.clone());
    }
}

impl ExporterFTP {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("ExporterFTP"));
        Self {
            enabled: properties.get_bool("enabled"),
            server: properties.get_string("server"),
            share: properties.get_string("share"),
            password: properties.get_string("password"),
            username: properties.get_string("username"),
            cache_read: properties.get_bool("cache_read"),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("ExporterFTP"))
            .set("enabled", bool_to_string(self.enabled))
            .set("server", self.server.clone())
            .set("share", self.share.clone())
            .set("password", self.password.clone())
            .set("username", self.username.clone())
            .set("cache_read", bool_to_string(self.cache_read));
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct CmdEvents {
    pub on_save: String,
    pub on_track_change: String,
    pub on_like: String,
}

impl CmdEvents {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("CMD_EVENTS"));
        Self {
            on_save: properties.get_string_with_default("on_save", "filetool.sh -b"),
            on_track_change: properties.get_string("on_track_change"),
            on_like: properties.get_string("on_like"),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("CMD_EVENTS"))
            .set("on_save", self.on_save.clone())
            .set("on_track_change", self.on_track_change.clone())
            .set("on_like", self.on_like.clone());
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Config {
    path: PathBuf,
    pub tidal: Tidal,
    pub player: Player,
    pub gui: Gui,
    pub gpio: Gpio,
    pub exporter_file: ExporterFile,
    pub exporter_ftp: ExporterFTP,
    pub cmd_events: CmdEvents,
}

impl Config {
    pub fn init_default_path() -> Self {
        let config_path = home::home_dir().unwrap().join(".config/woodaudio/config.ini");
        Self::init(config_path)
    }
    pub fn init(path: PathBuf) -> Self {
        let conf = Ini::load_from_file(path.clone())
            .unwrap_or_default();

        let config = Self {
            cmd_events: CmdEvents::init(&conf),
            path,
            tidal: Tidal::init(&conf),
            player: Player::init(&conf),
            gui: Gui::init(&conf),
            gpio: Gpio::init(&conf),
            exporter_file: ExporterFile::init(&conf),
            exporter_ftp: ExporterFTP::init(&conf),
        };

        config.save();

        config
    }
    pub fn save(&self) {
        let mut conf = Ini::new();
        self.tidal.prepare_to_save(&mut conf);
        self.player.prepare_to_save(&mut conf);
        self.gui.prepare_to_save(&mut conf);
        self.gpio.prepare_to_save(&mut conf);
        self.exporter_file.prepare_to_save(&mut conf);
        self.exporter_ftp.prepare_to_save(&mut conf);
        self.cmd_events.prepare_to_save(&mut conf);
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        conf.write_to_file(self.path.clone()).unwrap();

        if !self.cmd_events.on_save.is_empty() {
            match std::process::Command::new("sh").arg("-c").arg(&self.cmd_events.on_save).spawn() {
                Ok(_) => {},
                Err(e) => log::error!("[Config] Failed to execute on-save command '{}': {}", self.cmd_events.on_save, e),
            }
        }
    }
}