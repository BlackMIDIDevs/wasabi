use colors_transform::{Color, Rgb};
use directories::BaseDirs;
use egui::Color32;
use serde_derive::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
struct WasabiConfigFile {
    note_speed: f64,
    bg_color: String,
    bar_color: String,
    random_colors: bool,
    sfz_path: String,
    first_key: u8,
    last_key: u8,
    midi_loading: usize,
    buffer_ms: f64,
    layer_count: usize,
    synth: usize,
}

pub struct WasabiPermanentSettings {
    pub note_speed: f64,
    pub bg_color: Color32,
    pub bar_color: Color32,
    pub random_colors: bool,
    pub sfz_path: String,
    pub key_range: RangeInclusive<u8>,
    pub midi_loading: usize,
    pub buffer_ms: f64,
    pub layer_count: usize,
    pub synth: usize,
}

pub struct WasabiTemporarySettings {
    pub panel_visible: bool,
    pub stats_visible: bool,
    pub settings_visible: bool,
}

impl Default for WasabiPermanentSettings {
    fn default() -> Self {
        WasabiPermanentSettings {
            note_speed: 0.25,
            bg_color: Color32::from_rgb(30, 30, 30),
            bar_color: Color32::from_rgb(145, 0, 0),
            random_colors: false,
            sfz_path: "".to_string(),
            key_range: 0..=127,
            midi_loading: 0,
            buffer_ms: 10.0,
            layer_count: 4,
            synth: 0,
        }
    }
}

static CONFIG_PATH: &str = "wasabi_config.toml";

impl WasabiPermanentSettings {
    pub fn new_or_load() -> Self {
        let config_path = Self::get_config_path();
        if !Path::new(&config_path).exists() {
            Self::load_and_save_defaults()
        } else {
            match fs::read_to_string(&config_path) {
                Ok(content) => match toml::from_str::<WasabiConfigFile>(&content) {
                    Ok(cfg) => {
                        if let (Ok(bg), Ok(bar)) = (
                            Rgb::from_hex_str(&cfg.bg_color),
                            Rgb::from_hex_str(&cfg.bar_color),
                        ) {
                            Self {
                                note_speed: cfg.note_speed,
                                bg_color: Color32::from_rgb(
                                    bg.get_red() as u8,
                                    bg.get_green() as u8,
                                    bg.get_blue() as u8,
                                ),
                                bar_color: Color32::from_rgb(
                                    bar.get_red() as u8,
                                    bar.get_green() as u8,
                                    bar.get_blue() as u8,
                                ),
                                random_colors: cfg.random_colors,
                                sfz_path: cfg.sfz_path,
                                key_range: cfg.first_key..=cfg.last_key,
                                midi_loading: cfg.midi_loading,
                                buffer_ms: cfg.buffer_ms,
                                layer_count: cfg.layer_count,
                                synth: cfg.synth,
                            }
                        } else {
                            Self::load_and_save_defaults()
                        }
                    }
                    Err(..) => Self::load_and_save_defaults(),
                },
                Err(..) => Self::load_and_save_defaults(),
            }
        }
    }

    pub fn save_to_file(&self) {
        let config_path = Self::get_config_path();
        let cfg = WasabiConfigFile {
            note_speed: self.note_speed,
            bg_color: Rgb::from(
                self.bg_color.r() as f32,
                self.bg_color.g() as f32,
                self.bg_color.b() as f32,
            )
            .to_css_hex_string(),
            bar_color: Rgb::from(
                self.bar_color.r() as f32,
                self.bar_color.g() as f32,
                self.bar_color.b() as f32,
            )
            .to_css_hex_string(),
            random_colors: self.random_colors,
            sfz_path: self.sfz_path.clone(),
            first_key: *self.key_range.start(),
            last_key: *self.key_range.end(),
            midi_loading: self.midi_loading,
            buffer_ms: self.buffer_ms,
            layer_count: self.layer_count,
            synth: self.synth,
        };
        let toml: String = toml::to_string(&cfg).unwrap();
        if Path::new(&config_path).exists() {
            fs::remove_file(&config_path).expect("Error deleting old config");
        }
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(toml.as_bytes())
            .expect("Error creating config");
    }

    fn load_and_save_defaults() -> Self {
        match fs::remove_file(CONFIG_PATH) {
            Ok(..) => {
                let cfg = Self::default();
                Self::save_to_file(&cfg);
                cfg
            }
            Err(..) => Self::default(),
        }
    }

    fn get_config_path() -> String {
        if let Some(base_dirs) = BaseDirs::new() {
            let mut path: PathBuf = base_dirs.config_dir().to_path_buf();
            path.push("wasabi");
            path.push("wasabi-config.toml");

            if let Ok(..) = std::fs::create_dir_all(path.parent().unwrap()) {
                if let Some(path) = path.to_str() {
                    path.to_string()
                } else {
                    "wasabi-config.toml".to_string()
                }
            } else {
                "wasabi-config.toml".to_string()
            }
        } else {
            "wasabi-config.toml".to_string()
        }
    }
}

impl WasabiTemporarySettings {
    pub fn new() -> Self {
        Self {
            panel_visible: true,
            stats_visible: true,
            settings_visible: false,
        }
    }
}
