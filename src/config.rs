use std::path::PathBuf;

use ini::{Ini, Properties};

trait ParseIni {
    fn get_string(&self, name: &str) -> String;
    fn get_string_with_default(&self, name: &str, default: &str) -> String;
    fn get_bool(&self, name: &str) -> bool; 
    fn get_bool_with_default(&self, name: &str, default: bool) -> bool;   
}

impl ParseIni for Option<&Properties> {
    fn get_string(&self, name: &str) -> String {
        self.map(|properties| properties.get(name)).flatten().unwrap_or("").to_string()
    }
    fn get_string_with_default(&self, name: &str, default: &str) -> String {
        self.map(|properties| properties.get(name)).flatten().unwrap_or(default).to_string()   
    }
    fn get_bool(&self, name: &str) -> bool {
        match self.get_string(name).as_str() {
            "true" => true,
            _ => false,
        }
    }
    fn get_bool_with_default(&self, name: &str, default: bool) -> bool {
        match self.get_string(name).as_str() {
            "true" => true,
            "false" => false,
            _ => default,
        }
    }
}

fn bool_to_string(value: bool) -> String {
    match value {
        true => "true",
        false => "false",
    }.to_string()
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TidalConfig {
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
}

impl TidalConfig {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("Tidal"));
        Self {
            token_type: properties.get_string_with_default("token_type", "Bearer"),
            access_token: properties.get_string("access_token"),
            refresh_token: properties.get_string("refresh_token"),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("Tidal"))
            .set("token_type", self.token_type.clone())
            .set("access_token", self.access_token.clone())
            .set("refresh_token", self.refresh_token.clone());
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct GuiConfig {
    pub display_cover_background: bool,
    pub display_cover_foreground: bool,
}

impl GuiConfig {
    fn init(conf: &Ini) -> Self {
        let properties = conf.section(Some("GUI"));
        Self {
            display_cover_background: properties.get_bool_with_default("display_cover_background", true),
            display_cover_foreground: properties.get_bool_with_default("display_cover_foreground", true),
        }
    }
    fn prepare_to_save(&self, ini: &mut Ini) {
        ini.with_section(Some("GUI"))
            .set("display_cover_background", bool_to_string(self.display_cover_background))
            .set("display_cover_foreground", bool_to_string(self.display_cover_foreground));
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ExporterFTPConfig {
    pub enabled: bool,
    pub server: String,
    pub share: String,
    pub password: String,
    pub username: String,
    pub cache_read: bool,
}

impl ExporterFTPConfig {
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
pub struct Config {
    path: PathBuf,
    pub tidal: TidalConfig,
    pub gui: GuiConfig,
    pub exporter_ftp: ExporterFTPConfig,
}

impl Config {
    pub fn init_default_path() -> Self {
        let config_path = home::home_dir().unwrap().join("config.ini");
        Self::init(config_path)
    }
    pub fn init(path: PathBuf) -> Self {
        let conf = Ini::load_from_file(path.clone()).unwrap_or(Ini::new());

        Self { 
            path: path,
            tidal: TidalConfig::init(&conf),
            gui: GuiConfig::init(&conf), 
            exporter_ftp: ExporterFTPConfig::init(&conf),
        }
    }
    pub fn save(&self) {
        let mut conf = Ini::new();
        self.tidal.prepare_to_save(&mut conf);
        self.gui.prepare_to_save(&mut conf);
        self.exporter_ftp.prepare_to_save(&mut conf);
        conf.write_to_file(self.path.clone()).unwrap();
    }
}