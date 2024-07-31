use colors_transform::{Color, Rgb};
use egui::Color32;
use serde_derive::Deserialize;
use std::fs;

use super::{MidiLoading, MidiSettings, Synth, SynthSettings, VisualSettings, WasabiSettings};

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
    use_threadpool: bool,
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
    pub fn migrate() -> Result<WasabiSettings, toml::de::Error> {
        let config_path = WasabiSettings::get_config_path();
        let content = fs::read_to_string(config_path).unwrap_or_default();
        let cfg = toml::from_str::<WasabiConfigFileV0>(&content)?;
        if let (Ok(bg), Ok(bar)) = (
            Rgb::from_hex_str(&cfg.bg_color),
            Rgb::from_hex_str(&cfg.bar_color),
        ) {
            Ok(WasabiSettings {
                synth: SynthSettings {
                    synth: Synth::from(cfg.synth),
                    buffer_ms: cfg.buffer_ms,
                    use_threadpool: cfg.use_threadpool,
                    limit_layers: cfg.limit_layers,
                    layer_count: cfg.layer_count,
                    fade_out_kill: cfg.fade_out_kill,
                    linear_envelope: cfg.linear_envelope,
                    use_effects: cfg.use_effects,
                    sfz_path: cfg.sfz_path,
                    vel_ignore: cfg.vel_ignore_lo..=cfg.vel_ignore_hi,
                },
                midi: MidiSettings {
                    note_speed: cfg.note_speed,
                    random_colors: cfg.random_colors,
                    key_range: cfg.first_key..=cfg.last_key,
                    midi_loading: MidiLoading::from(cfg.midi_loading),
                },
                visual: VisualSettings {
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
        } else {
            Ok(WasabiSettings::default())
        }
    }
}
