use directories::BaseDirs;
use egui::Color32;
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs,
    io::Write,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};
use xsynth_core::soundfont::SoundfontInitOptions;
use xsynth_realtime::XSynthRealtimeConfig;

mod enums;
mod migrations;

pub use enums::*;

// region: gui

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GuiSettings {
    pub check_for_updates: bool,
    pub fps_limit: usize,
    pub skip_control: f64,
    pub speed_control: f64,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            fps_limit: 0,
            skip_control: 1.0,
            speed_control: 0.05,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct StatisticsSettings {
    pub border: bool,
    pub floating: bool,
    pub opacity: f32,
    pub order: Vec<(Statistics, bool)>,
}

impl Default for StatisticsSettings {
    fn default() -> Self {
        Self {
            border: true,
            floating: true,
            opacity: 0.5,
            order: Statistics::iter().map(|i| (*i, true)).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SceneSettings {
    pub bg_color: Color32,
    pub bar_color: Color32,
    pub statistics: StatisticsSettings,
    pub note_speed: f64,
    pub key_range: RangeInclusive<u8>,
}

impl Default for SceneSettings {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(30, 30, 30),
            bar_color: Color32::from_rgb(145, 0, 0),
            statistics: Default::default(),
            note_speed: 0.25,
            key_range: 0..=127,
        }
    }
}

// endregion

// region: midi

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MidiSettings {
    pub parsing: MidiParsing,
    pub colors: Colors,
    pub palette_path: PathBuf,
}

impl Default for MidiSettings {
    fn default() -> Self {
        Self {
            parsing: MidiParsing::Cake,
            colors: Colors::Rainbow,
            palette_path: PathBuf::new(),
        }
    }
}

// endregion

// region: synth

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct XSynthSettings {
    pub config: XSynthRealtimeConfig,
    pub limit_layers: bool,
    pub layers: usize,
}

impl Default for XSynthSettings {
    fn default() -> Self {
        Self {
            config: Default::default(),
            limit_layers: true,
            layers: 4,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct KdmapiSettings {
    pub use_om_sflist: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct WasabiSoundfont {
    pub path: PathBuf,
    pub enabled: bool,
    pub options: SoundfontInitOptions,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct SynthSettings {
    pub synth: Synth,
    pub soundfonts: Vec<WasabiSoundfont>,

    pub xsynth: XSynthSettings,
    pub kdmapi: KdmapiSettings,
    pub midi_device: String,
}

impl Default for SynthSettings {
    fn default() -> Self {
        Self {
            synth: Synth::XSynth,
            soundfonts: Vec::new(),
            xsynth: Default::default(),
            kdmapi: Default::default(),
            midi_device: String::new(),
        }
    }
}

// endregion

// region: general

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct WasabiSettings {
    pub gui: GuiSettings,
    pub scene: SceneSettings,
    pub midi: MidiSettings,
    pub synth: SynthSettings,
}

impl WasabiSettings {
    const VERSION_TEXT: &str = "# DON'T EDIT THIS LINE; Version: 2\n";

    pub fn new_or_load() -> Self {
        let config_path = Self::get_config_path();
        if !Path::new(&config_path).exists() {
            return Self::load_and_save_defaults();
        } else if let Ok(config) = fs::read_to_string(&config_path) {
            if config.starts_with(Self::VERSION_TEXT) {
                let offset = Self::VERSION_TEXT.len();
                if let Ok(config) = serde_json::from_str(&config[offset..]) {
                    return config;
                }
            } else if config.starts_with("# DON'T EDIT THIS LINE; Version: 1") {
                if let Ok(cfg) = migrations::WasabiConfigFileV1::migrate_to_v2(config) {
                    cfg.save_to_file();
                    return cfg;
                }
            } else {
                if let Ok(v1) = migrations::WasabiConfigFileV0::migrate_to_v1(config) {
                    let cfg = migrations::WasabiConfigFileV1::migrate_to_v2_raw(v1);
                    cfg.save_to_file();
                    return cfg;
                }
            }
        }

        println!("Error loading config. Resetting.");
        Self::load_and_save_defaults()
    }

    pub fn save_to_file(&self) {
        let config_path = Self::get_config_path();
        let cfg: String = serde_json::to_string_pretty(&self).unwrap();
        if let Ok(mut file) = fs::File::create(&config_path) {
            file.write_all(Self::VERSION_TEXT.as_bytes()).unwrap();
            file.write_all(cfg.as_bytes())
                .expect("Error creating config");
        }
    }

    fn load_and_save_defaults() -> Self {
        let cfg = Self::default();
        Self::save_to_file(&cfg);
        cfg
    }

    fn get_config_dir() -> PathBuf {
        if let Some(base_dirs) = BaseDirs::new() {
            let mut path: PathBuf = base_dirs.config_dir().to_path_buf();
            path.push("wasabi");

            if std::fs::create_dir_all(&path).is_ok() {
                return path;
            }
        }

        PathBuf::from("./")
    }

    fn get_config_path() -> PathBuf {
        let mut path = Self::get_config_dir();
        path.push("wasabi-config.toml");

        path
    }

    pub fn get_palettes_dir() -> PathBuf {
        let mut path = Self::get_config_dir();
        path.push("palettes");
        std::fs::create_dir_all(&path).unwrap_or_default();

        path
    }
}

// endregion
