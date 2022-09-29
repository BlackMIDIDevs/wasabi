use egui::Color32;
use std::{
    path::Path,
    fs,
    io::Write,
};
use serde_derive::{Deserialize, Serialize};
use colors_transform::{Rgb, Color};

#[derive(Deserialize, Serialize)]
struct WasabiConfigFile {
    note_speed: f64,
    bg_color: String,
    bar_color: String,
    random_colors: bool,
    sfz_path: String,
    first_key: usize,
    last_key: usize,
}

pub struct WasabiPermanentSettings {
    pub note_speed: f64,
    pub bg_color: Color32,
    pub bar_color: Color32,
    pub random_colors: bool,
    pub sfz_path: String,
    pub first_key: usize,
    pub last_key: usize,
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
            bg_color: Color32::from_rgb(90, 90, 90),
            bar_color: Color32::from_rgb(65, 0, 30),
            random_colors: true,
            sfz_path: "".to_string(),
            first_key: 0,
            last_key: 127,
        }
    }
}

static CONFIG_PATH: &str = "wasabi_config.toml";

impl WasabiPermanentSettings {
    pub fn new_or_load() -> Self {
        if !Path::new(CONFIG_PATH).exists() {
            let s = WasabiPermanentSettings::default();
            let cfg = WasabiConfigFile {
                note_speed: s.note_speed,
                bg_color: Rgb::from(s.bg_color.r() as f32, s.bg_color.g() as f32, s.bg_color.b() as f32).to_css_hex_string(),
                bar_color: Rgb::from(s.bar_color.r() as f32, s.bar_color.g() as f32, s.bar_color.b() as f32).to_css_hex_string(),
                random_colors: s.random_colors,
                sfz_path: s.sfz_path.clone(),
                first_key: s.first_key,
                last_key: s.last_key,
            };
            let toml: String = toml::to_string(&cfg).unwrap();
            let mut file = fs::File::create(CONFIG_PATH).unwrap();
            file.write_all(toml.as_bytes()).expect("Error creating config");
            s
        } else {
            let content = std::fs::read_to_string(CONFIG_PATH).unwrap();
            let s: WasabiConfigFile = toml::from_str(&content).unwrap();
            let bg_color_tmp = Rgb::from_hex_str(&s.bg_color).unwrap();
            let bar_color_tmp = Rgb::from_hex_str(&s.bar_color).unwrap();
            WasabiPermanentSettings {
                note_speed: s.note_speed,
                bg_color: Color32::from_rgb(bg_color_tmp.get_red() as u8,
                                            bg_color_tmp.get_green() as u8,
                                            bg_color_tmp.get_blue() as u8),
                bar_color: Color32::from_rgb(bar_color_tmp.get_red() as u8,
                                             bar_color_tmp.get_green() as u8,
                                             bar_color_tmp.get_blue() as u8),
                random_colors: s.random_colors,
                sfz_path: s.sfz_path,
                first_key: s.first_key,
                last_key: s.last_key,
            }
        }
    }

    pub fn save_to_file(&self) {
        let cfg = WasabiConfigFile {
            note_speed: self.note_speed,
            bg_color: Rgb::from(self.bg_color.r() as f32, self.bg_color.g() as f32, self.bg_color.b() as f32).to_css_hex_string(),
            bar_color: Rgb::from(self.bar_color.r() as f32, self.bar_color.g() as f32, self.bar_color.b() as f32).to_css_hex_string(),
            random_colors: self.random_colors,
            sfz_path: self.sfz_path.clone(),
            first_key: self.first_key,
            last_key: self.last_key,
        };
        let toml: String = toml::to_string(&cfg).unwrap();
        if Path::new(CONFIG_PATH).exists() {
            fs::remove_file(CONFIG_PATH).expect("Error deleting old config");
        }
        let mut file = fs::File::create(CONFIG_PATH).unwrap();
        file.write_all(toml.as_bytes()).expect("Error creating config");
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
