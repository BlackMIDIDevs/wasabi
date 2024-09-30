use colors_transform::{Color, Rgb};
use egui::Color32;
use serde_derive::Deserialize;

use super::v1::{MidiSettingsV1, SynthSettingsV1, VisualSettingsV1};
use super::WasabiConfigFileV1;
use crate::settings::enums::{MidiParsing, Synth};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct WasabiConfigFileV0 {
    note_speed: f64,
    bg_color: String,
    bar_color: String,
    random_colors: bool,
    sfz_path: String,
    first_key: u8,
    last_key: u8,
    midi_loading: usize,
    buffer_ms: f64,
    limit_layers: bool,
    layer_count: usize,
    fade_out_kill: bool,
    linear_envelope: bool,
    use_effects: bool,
    vel_ignore_lo: u8,
    vel_ignore_hi: u8,
    synth: usize,
}

impl WasabiConfigFileV0 {
    pub fn migrate_to_v1(content: String) -> Result<WasabiConfigFileV1, toml::de::Error> {
        let cfg = toml::from_str::<WasabiConfigFileV0>(&content)?;
        let bg = Rgb::from_hex_str(&cfg.bg_color).unwrap_or(Rgb::from(0.1, 0.1, 0.1));
        let bar = Rgb::from_hex_str(&cfg.bar_color).unwrap_or(Rgb::from(0.56, 0.0, 0.0));
        Ok(WasabiConfigFileV1 {
            synth: SynthSettingsV1 {
                synth: Synth::from(cfg.synth),
                buffer_ms: cfg.buffer_ms,
                limit_layers: cfg.limit_layers,
                layer_count: cfg.layer_count,
                fade_out_kill: cfg.fade_out_kill,
                use_effects: cfg.use_effects,
                sfz_path: cfg.sfz_path,
                vel_ignore: cfg.vel_ignore_lo..=cfg.vel_ignore_hi,
            },
            midi: MidiSettingsV1 {
                note_speed: cfg.note_speed,
                random_colors: cfg.random_colors,
                key_range: cfg.first_key..=cfg.last_key,
                midi_loading: MidiParsing::from(cfg.midi_loading),
            },
            visual: VisualSettingsV1 {
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
                show_top_pannel: true,
                show_statistics: true,
                fullscreen: false,
            },
            load_midi_file: None,
        })
    }
}
