use clap::{parser::ValueSource, value_parser, Arg, ArgAction, Command, ValueHint};
use colors_transform::{Color, Rgb};
use egui::Color32;
use miette::{Diagnostic, LabeledSpan, MietteHandlerOpts, NamedSource, ReportHandler};
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs,
    io::Write,
    ops::{Range, RangeInclusive},
    path::Path,
    str::FromStr,
};
use xsynth_core::{channel::ChannelInitOptions, soundfont::SoundfontInitOptions};
use xsynth_realtime::config::XSynthRealtimeConfig;

#[inline(always)]
fn f64_parser(s: &str) -> Result<f64, String> {
    s.parse().map_err(|e| format!("{}", e))
}

#[inline(always)]
fn note_speed(s: &str) -> Result<f64, String> {
    let num: f64 = f64_parser(s)?;
    if (0.0001..=2.0).contains(&num) {
        Ok(2.0001 - num)
    } else {
        Err(String::from("Number must be between >0 and 2.0"))
    }
}

#[inline(always)]
fn color_parser(s: &str) -> Result<Color32, String> {
    let rgb = Rgb::from_hex_str(s).map_err(|e| e.message)?;
    Ok(Color32::from_rgb(
        rgb.get_red() as u8,
        rgb.get_green() as u8,
        rgb.get_blue() as u8,
    ))
}

#[inline(always)]
fn range_parser(s: &str) -> Result<RangeInclusive<u8>, String> {
    let range = s
        .split_once(',')
        .ok_or_else(|| String::from("This argument requires 2 numbers, comma seperated"))?;

    Ok(range.0.parse().map_err(|e| format!("{}", e))?
        ..=range.1.parse().map_err(|e| format!("{}", e))?)
}

mod color32_serde {
    use colors_transform::Rgb;
    use egui::Color32;
    use serde::{de::Visitor, Deserializer, Serializer};

    use super::color_parser;

    pub fn serialize<S>(color: &Color32, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_color =
            Rgb::from(color.r() as f32, color.g() as f32, color.b() as f32).to_css_hex_string();

        ser.serialize_str(&hex_color)
    }

    pub struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A color encoded as a hex string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            color_parser(v)
                .map_err(|e| E::invalid_value(serde::de::Unexpected::Str(v), &e.as_str()))
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_str(ColorVisitor)
    }
}

mod range_serde {
    use std::ops::RangeInclusive;

    use serde::{
        de::{self, Visitor},
        ser::SerializeStruct,
        Deserializer, Serializer,
    };

    use serde_derive::Deserialize;

    pub fn serialize<S>(range: &RangeInclusive<u8>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut ser_struct = ser.serialize_struct("range", 2)?;
        ser_struct.serialize_field("hi", range.start())?;
        ser_struct.serialize_field("lo", range.end())?;
        ser_struct.end()
    }

    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        Hi,
        Lo,
    }

    pub struct RangeVisitor;

    impl<'de> Visitor<'de> for RangeVisitor {
        type Value = RangeInclusive<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A color encoded as a hex string")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut hi = None;
            let mut lo = None;
            while let Some(key) = map.next_key::<Field>()? {
                match key {
                    Field::Hi => {
                        if hi.is_some() {
                            return Err(de::Error::duplicate_field("hi"));
                        }
                        hi = Some(map.next_value::<u8>()?);
                    }
                    Field::Lo => {
                        if lo.is_some() {
                            return Err(de::Error::duplicate_field("lo"));
                        }
                        lo = Some(map.next_value::<u8>()?);
                    }
                }
            }

            let hi = hi.ok_or_else(|| de::Error::missing_field("hi"))?;
            let lo = lo.ok_or_else(|| de::Error::missing_field("lo"))?;

            Ok(hi..=lo)
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<RangeInclusive<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_struct("range", &["hi", "lo"], RangeVisitor)
    }
}

#[repr(usize)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum MidiLoading {
    Ram = 0,
    Live = 1,
}

impl FromStr for MidiLoading {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ram" | "RAM" | "Ram" => Ok(MidiLoading::Ram),
            "live" | "Live" => Ok(MidiLoading::Live),
            s => Err(format!(
                "{} was not expected. Expected one of `ram`, or `live`",
                s
            )),
        }
    }
}

#[repr(usize)]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Synth {
    XSynth = 0,
    Kdmapi = 1,
}

impl FromStr for Synth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "xsynth" | "XSynth" => Ok(Synth::XSynth),
            "kdmapi" | "KDMPAI" => Ok(Synth::Kdmapi),
            s => Err(format!(
                "{} was not expected. Expected one of `xsynth`, or `kdmapi`",
                s
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VisualSettings {
    pub audio_only: bool,
    #[serde(with = "color32_serde")]
    pub bg_color: Color32,
    #[serde(with = "color32_serde")]
    pub bar_color: Color32,
    pub show_top_pannel: bool,
    pub show_statistics: bool,
    pub fullscreen: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MidiSettings {
    pub note_speed: f64,
    pub random_colors: bool,
    #[serde(with = "range_serde")]
    pub key_range: RangeInclusive<u8>,
    pub midi_loading: MidiLoading,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SynthSettings {
    pub synth: Synth,
    pub buffer_ms: f64,
    pub sfz_path: String,
    pub limit_layers: bool,
    pub layer_count: usize,
    #[serde(with = "range_serde")]
    pub vel_ignore: RangeInclusive<u8>,
    pub fade_out_kill: bool,
    pub linear_envelope: bool,
    pub use_effects: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WasabiSettings {
    pub synth: SynthSettings,
    pub midi: MidiSettings,
    pub visual: VisualSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_midi_file: Option<String>,
}

impl Default for WasabiSettings {
    fn default() -> Self {
        WasabiSettings {
            synth: SynthSettings {
                synth: Synth::XSynth,
                buffer_ms: XSynthRealtimeConfig::default().render_window_ms,
                sfz_path: String::new(),
                limit_layers: true,
                layer_count: 4,
                vel_ignore: 0..=0,
                fade_out_kill: ChannelInitOptions::default().fade_out_killing,
                linear_envelope: SoundfontInitOptions::default().linear_release,
                use_effects: SoundfontInitOptions::default().use_effects,
            },
            midi: MidiSettings {
                note_speed: 0.25,
                random_colors: false,
                key_range: 0..=127,
                midi_loading: MidiLoading::Ram,
            },
            visual: VisualSettings {
                audio_only: false,
                bg_color: Color32::from_rgb(30, 30, 30),
                bar_color: Color32::from_rgb(145, 0, 0),
                show_top_pannel: true,
                show_statistics: true,
                fullscreen: false,
            },
            load_midi_file: None,
        }
    }
}

static CONFIG_PATH: &str = "wasabi-config.toml";

#[derive(thiserror::Error)]
#[error("There was an error parsing the config file")]
struct TomlError<'a> {
    message: &'a str,
    src: NamedSource,
    err_span: Option<Range<usize>>,
}

impl<'a> Debug for TomlError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        MietteHandlerOpts::new()
            .terminal_links(false)
            .color(false)
            .tab_width(4)
            .build()
            .debug(self, f)
    }
}

impl<'a> Diagnostic for TomlError<'a> {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        Some(Box::new(
            [if let Some(ref err_span) = self.err_span {
                LabeledSpan::new(
                    Some(self.message.to_string()),
                    err_span.start,
                    err_span.len(),
                )
            } else {
                LabeledSpan::new(Some(self.message.to_string()), 0, 1)
            }]
            .into_iter(),
        ))
    }
}

include!("help/help_cmdline.rs");

impl WasabiSettings {
    pub fn new_or_load() -> Result<Self, String> {
        let config_path = Self::get_config_path();
        let mut config = if !Path::new(&config_path).exists() {
            Self::load_and_save_defaults()
        } else {
            let config = fs::read_to_string(&config_path).unwrap();
            toml::from_str(&config).map_err(|e| {
                format!(
                    "{:?}",
                    TomlError {
                        message: e.message(),
                        src: NamedSource::new(config_path, config),
                        err_span: e.span(),
                    }
                )
            })?
        };

        config.augment_from_args();
        Ok(config)
    }

    pub fn save_to_file(&self) {
        let config_path = Self::get_config_path();
        let toml: String = toml::to_string(&self).unwrap();
        if Path::new(&config_path).exists() {
            fs::remove_file(&config_path).expect("Error deleting old config");
        }
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(toml.as_bytes())
            .expect("Error creating config");
    }

    fn augment_from_args(&mut self) {
        let matches = Command::new("wasabi")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new("synth")
                    .help(synth_short_help!())
                    .long_help(synth_long_help!())
                    .short('S')
                    .long("synth")
                    .value_parser(Synth::from_str),
            )
            .arg(
                Arg::new("buffer-ms")
                    .help(buffer_ms_short_help!())
                    .long_help(buffer_ms_long_help!())
                    .short('b')
                    .long("buffer-ms")
                    .value_parser(f64_parser),
            )
            .arg(
                Arg::new("sfz-path")
                    .help(sfz_path_short_help!())
                    .long_help(sfz_path_long_help!())
                    .short('s')
                    .long("sfz-path")
                    .value_hint(ValueHint::FilePath),
            )
            .arg(
                Arg::new("no-layer-limit")
                    .short('L')
                    .help(layer_limit_short_help!())
                    .long_help(layer_limit_long_help!())
                    .long("no-layer-limit")
                    .conflicts_with("layer-count")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("layer-count")
                    .help(layer_count_short_help!())
                    .long_help(layer_count_long_help!())
                    .short('l')
                    .long("layer-count")
                    .value_parser(value_parser!(usize)),
            )
            .arg(
                Arg::new("vel-ignore")
                    .help(vel_ignore_short_help!())
                    .long_help(vel_ignore_long_help!())
                    .short('v')
                    .long("vel-ignore")
                    .value_parser(range_parser),
            )
            .arg(
                Arg::new("fade-out-kill")
                    .help(fade_out_kill_short_help!())
                    .long_help(fade_out_kill_long_help!())
                    .short('F')
                    .long("fade-out-kill")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("linear-envelope")
                    .help(linear_envelope_short_help!())
                    .long_help(linear_envelope_long_help!())
                    .short('e')
                    .long("linear-envelope")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-effects")
                    .help(no_effects_short_help!())
                    .long_help(no_effects_long_help!())
                    .short('N')
                    .long("no-effects")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("note-speed")
                    .help(note_speed_short_help!())
                    .long_help(note_speed_long_help!())
                    .short('n')
                    .long("note-speed")
                    .value_parser(note_speed),
            )
            .arg(
                Arg::new("random-colors")
                    .help(random_colors_short_help!())
                    .long_help(random_colors_long_help!())
                    .short('r')
                    .long("random-colors")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("key-range")
                    .help(key_range_short_help!())
                    .long_help(key_range_long_help!())
                    .short('k')
                    .long("key-range")
                    .value_parser(range_parser),
            )
            .arg(
                Arg::new("midi-loading")
                    .help(midi_loading_short_help!())
                    .long_help(midi_loading_long_help!())
                    .short('m')
                    .long("midi-loading")
                    .value_parser(MidiLoading::from_str),
            )
            .arg(
                Arg::new("audio-only")
                    .help(audio_only_short_help!())
                    .long_help(audio_only_long_help!())
                    .short('a')
                    .long("audio-only")
                    .requires("midi-file")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("bg-color")
                    .help(bg_color_short_help!())
                    .long_help(bg_color_long_help!())
                    .short('c')
                    .long("bg-color")
                    .value_parser(color_parser),
            )
            .arg(
                Arg::new("bar-color")
                    .help(bar_color_short_help!())
                    .long_help(bar_color_long_help!())
                    .short('C')
                    .long("bar-color")
                    .value_parser(color_parser),
            )
            .arg(
                Arg::new("hide-top-pannel")
                    .help(hide_top_panel_short_help!())
                    .long_help(hide_top_panel_long_help!())
                    .short('t')
                    .long("hide-top-pannel")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("hide-statistics")
                    .help(hide_statistics_short_help!())
                    .long_help(hide_statistics_long_help!())
                    .short('T')
                    .long("hide-statistics")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("fullscreen")
                    .help(fullscreen_short_help!())
                    .long_help(fullscreen_long_help!())
                    .short('f')
                    .long("fullscreen")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("midi-file")
                    .help(midi_file_short_help!())
                    .long_help(midi_file_long_help!())
                    .value_hint(ValueHint::FilePath),
            )
            .get_matches();

        macro_rules! set {
            ($one:ident.$two:ident,$value:expr) => {
                if let Some(value) = matches.get_one($value) {
                    self.$one.$two = *value;
                }
            };
        }

        macro_rules! set_flag {
            ($one:ident.$two:ident,$value:expr) => {
                if matches!(matches.value_source($value), Some(ValueSource::CommandLine)) {
                    if let Some(value) = matches.get_one($value) {
                        self.$one.$two = *value;
                    }
                }
            };
        }

        macro_rules! set_owned {
            ($one:ident.$two:ident,$value:expr,$type:ty) => {
                if let Some(value) = matches.get_one::<$type>($value) {
                    self.$one.$two = value.to_owned();
                }
            };
        }

        self.load_midi_file = matches.get_one::<String>("midi-file").map(|f| f.to_owned());

        // Synth settings
        set!(synth.synth, "synth");
        set!(synth.buffer_ms, "buffer-ms");
        set_owned!(synth.sfz_path, "sfz-path", String);
        set_flag!(synth.limit_layers, "no-layer-limit");
        set!(synth.layer_count, "layer-count");
        set_owned!(synth.vel_ignore, "vel-ignore", RangeInclusive<u8>);
        set_flag!(synth.fade_out_kill, "fade-out-kill");
        set_flag!(synth.linear_envelope, "linear-envelope");
        set_flag!(synth.use_effects, "no-effects");

        // MIDI settings
        set!(midi.note_speed, "note-speed");
        set_flag!(midi.random_colors, "random-colors");
        set_owned!(midi.key_range, "key-range", RangeInclusive<u8>);
        set!(midi.midi_loading, "midi-loading");

        // Visual settings
        set_flag!(visual.audio_only, "audio-only");
        set!(visual.bg_color, "bg-color");
        set!(visual.bar_color, "bar-color");
        set_flag!(visual.show_top_pannel, "hide-top-pannel");
        set_flag!(visual.show_statistics, "hide-statistics");
        set_flag!(visual.fullscreen, "fullscreen");
    }

    fn load_and_save_defaults() -> Self {
        let _ = fs::remove_file(Self::get_config_path());
        let cfg = Self::default();
        Self::save_to_file(&cfg);
        cfg
    }

    fn get_config_path() -> String {
        if let Some(mut path) = dirs::config_dir() {
            path.push("wasabi");
            path.push(CONFIG_PATH);

            if let Ok(..) = std::fs::create_dir_all(path.parent().unwrap()) {
                if let Some(path) = path.to_str() {
                    path.to_string()
                } else {
                    CONFIG_PATH.to_string()
                }
            } else {
                CONFIG_PATH.to_string()
            }
        } else {
            CONFIG_PATH.to_string()
        }
    }
}
