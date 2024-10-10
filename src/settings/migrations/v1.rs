use egui::Color32;
use serde_derive::Deserialize;
use std::ops::RangeInclusive;
use xsynth_realtime::{ChannelInitOptions, XSynthRealtimeConfig};

use super::serializers::{color32_serde, range_serde};
use crate::settings::{
    MidiParsing, MidiSettings, SceneSettings, Synth, SynthSettings, WasabiSettings,
    WasabiSoundfont, XSynthSettings,
};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct VisualSettingsV1 {
    #[serde(with = "color32_serde")]
    pub bg_color: Color32,
    #[serde(with = "color32_serde")]
    pub bar_color: Color32,
    pub show_top_pannel: bool,
    pub show_statistics: bool,
    pub fullscreen: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct MidiSettingsV1 {
    pub note_speed: f64,
    pub random_colors: bool,
    #[serde(with = "range_serde")]
    pub key_range: RangeInclusive<u8>,
    pub midi_loading: MidiParsing,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct SynthSettingsV1 {
    pub synth: Synth,
    pub buffer_ms: f64,
    pub sfz_path: String,
    pub limit_layers: bool,
    pub layer_count: usize,
    #[serde(with = "range_serde")]
    pub vel_ignore: RangeInclusive<u8>,
    pub fade_out_kill: bool,
    pub use_effects: bool,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct WasabiConfigFileV1 {
    pub synth: SynthSettingsV1,
    pub midi: MidiSettingsV1,
    pub visual: VisualSettingsV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_midi_file: Option<String>,
}

impl WasabiConfigFileV1 {
    pub fn migrate_to_v2(content: String) -> Result<WasabiSettings, toml::de::Error> {
        let cfg = toml::from_str::<WasabiConfigFileV1>(&content)?;
        Ok(Self::migrate_to_v2_raw(cfg))
    }

    pub fn migrate_to_v2_raw(cfg: Self) -> WasabiSettings {
        WasabiSettings {
            scene: SceneSettings {
                bg_color: cfg.visual.bg_color,
                bar_color: cfg.visual.bar_color,
                statistics: Default::default(),
                note_speed: cfg.midi.note_speed,
                key_range: cfg.midi.key_range,
            },
            midi: MidiSettings {
                parsing: cfg.midi.midi_loading,
                ..Default::default()
            },
            synth: SynthSettings {
                synth: cfg.synth.synth,
                soundfonts: vec![WasabiSoundfont {
                    path: cfg.synth.sfz_path.into(),
                    enabled: true,
                    options: Default::default(),
                }],
                xsynth: XSynthSettings {
                    layers: cfg.synth.layer_count,
                    limit_layers: cfg.synth.limit_layers,
                    config: XSynthRealtimeConfig {
                        render_window_ms: cfg.synth.buffer_ms,
                        channel_init_options: ChannelInitOptions {
                            fade_out_killing: cfg.synth.fade_out_kill,
                        },
                        ignore_range: cfg.synth.vel_ignore,
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}
