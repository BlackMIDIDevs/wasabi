use clap::{parser::ValueSource, value_parser, Arg, ArgAction, Command, ValueHint};
use colors_transform::{Color, Rgb};
use directories::BaseDirs;
use egui::Color32;
use num_enum::FromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs,
    io::Write,
    ops::RangeInclusive,
    path::{Path, PathBuf},
    str::FromStr,
};
use xsynth_core::{channel::ChannelInitOptions, soundfont::SoundfontInitOptions};
use xsynth_realtime::config::XSynthRealtimeConfig;

mod migrations;

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
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum MidiLoading {
    #[default]
    Ram = 0,
    Live = 1,
    Cake = 2,
}

impl MidiLoading {
    pub const fn as_str(self) -> &'static str {
        match self {
            MidiLoading::Ram => "In RAM",
            MidiLoading::Live => "Live",
            MidiLoading::Cake => "Cake",
        }
    }
}

impl FromStr for MidiLoading {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ram" => Ok(MidiLoading::Ram),
            "live" => Ok(MidiLoading::Live),
            "cake" => Ok(MidiLoading::Cake),
            s => Err(format!(
                "{} was not expected. Expected one of `ram`, `live` or `cake`",
                s
            )),
        }
    }
}

#[repr(usize)]
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum Synth {
    #[default]
    XSynth = 0,
    Kdmapi = 1,
}

impl Synth {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Synth::XSynth => "XSynth",
            Synth::Kdmapi => "KDMAPI",
        }
    }
}

impl FromStr for Synth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xsynth" => Ok(Synth::XSynth),
            "kdmapi" => Ok(Synth::Kdmapi),
            s => Err(format!(
                "{} was not expected. Expected one of `xsynth`, or `kdmapi`",
                s
            )),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct VisualSettings {
    #[serde(with = "color32_serde")]
    pub bg_color: Color32,
    #[serde(with = "color32_serde")]
    pub bar_color: Color32,
    pub show_top_pannel: bool,
    pub show_statistics: bool,
    pub fullscreen: bool,
}

impl Default for VisualSettings {
    fn default() -> Self {
        VisualSettings {
            bg_color: Color32::from_rgb(30, 30, 30),
            bar_color: Color32::from_rgb(145, 0, 0),
            show_top_pannel: true,
            show_statistics: true,
            fullscreen: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MidiSettings {
    pub note_speed: f64,
    pub random_colors: bool,
    #[serde(with = "range_serde")]
    pub key_range: RangeInclusive<u8>,
    pub midi_loading: MidiLoading,
}

impl Default for MidiSettings {
    fn default() -> Self {
        MidiSettings {
            note_speed: 0.25,
            random_colors: false,
            key_range: 0..=127,
            midi_loading: MidiLoading::Cake,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
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

impl Default for SynthSettings {
    fn default() -> Self {
        SynthSettings {
            synth: Synth::XSynth,
            buffer_ms: XSynthRealtimeConfig::default().render_window_ms,
            sfz_path: String::new(),
            limit_layers: true,
            layer_count: 4,
            vel_ignore: 0..=0,
            fade_out_kill: ChannelInitOptions::default().fade_out_killing,
            linear_envelope: SoundfontInitOptions::default().linear_release,
            use_effects: SoundfontInitOptions::default().use_effects,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct WasabiSettings {
    pub synth: SynthSettings,
    pub midi: MidiSettings,
    pub visual: VisualSettings,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_midi_file: Option<String>,
}

static CONFIG_PATH: &str = "wasabi-config.toml";

impl WasabiSettings {
    pub fn new_or_load() -> Self {
        let config_path = Self::get_config_path();
        let mut config = if !Path::new(&config_path).exists() {
            Self::load_and_save_defaults()
        } else {
            let config = fs::read_to_string(&config_path).unwrap();
            if config.starts_with('#') {
                if let Ok(config) = toml::from_str(&config) {
                    config
                } else {
                    Self::load_and_save_defaults()
                }
            } else {
                let config = migrations::WasabiConfigFileV0::migrate().unwrap_or_default();
                config.save_to_file();
                config
            }
        };

        config.augment_from_args();
        config
    }

    pub fn save_to_file(&self) {
        let config_path = Self::get_config_path();
        let toml: String = toml::to_string(&self).unwrap();
        if Path::new(&config_path).exists() {
            fs::remove_file(&config_path).expect("Error deleting old config");
        }
        let mut file = fs::File::create(&config_path).unwrap();
        file.write_all(b"# DON'T EDIT THIS LINE; Version: 1\n\n")
            .unwrap();
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
                    .help("The synthesizer to use")
                    .long_help(
                        "The synthesizer that is used to play the MIDI. \
                        This can either be XSynth (recommended) or KDMAPI. KDMAPI \
                        only works if you have OmniMIDI installed, and are using Windows",
                    )
                    .short('S')
                    .long("synth")
                    .value_parser(Synth::from_str),
            )
            .arg(
                Arg::new("buffer-ms")
                    .help("The amount of time events are held in the buffer")
                    .long_help(
                        "The amount of time that events are held in the \
                        buffer before being played. Higher numbers may increase \
                        performance but also increase latency.",
                    )
                    .short('b')
                    .long("buffer-ms")
                    .value_parser(f64_parser),
            )
            .arg(
                Arg::new("sfz-path")
                    .help("The path to an SFZ SoundFont")
                    .long_help(
                        "The path to any SFZ soundfont. In audio only mode \
                        a soundfont must be passed either via the config file, or
                        this command line option. In the GUI you can set this under \
                        `Open Synth Settings > SFZ Path`",
                    )
                    .short('s')
                    .long("sfz-path")
                    .value_hint(ValueHint::FilePath),
            )
            .arg(
                Arg::new("dont-limit-layers")
                    .help("Do not apply the layer limit to the synth")
                    .long_help(
                        "This allows the synth to create as many layers as it \
                        needs to play the MIDI file faithfully. Only turn this on if your \
                        MIDI sounds bad, or your computer is running on GFuel",
                    )
                    .long("dont-limit-layers")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("layer-count")
                    .help("The amount of layers that the synthesizer can create")
                    .long_help(
                        "The maximum amount of voices the synth is \
                        allowed to create per key per channel",
                    )
                    .short('l')
                    .long("layer-count")
                    .value_parser(value_parser!(usize)),
            )
            .arg(
                Arg::new("vel-ignore")
                    .help("The range of note velocities that the synth will discard")
                    .long_help(
                        "Two numbers, comma seperated, that represent a range of velocities \
                        that the synth will discard, making notes in the range inaudible.",
                    )
                    .short('v')
                    .long("vel-ignore")
                    .value_parser(range_parser),
            )
            .arg(
                Arg::new("fade-out-kill")
                    .help("Once a voice is killed, fade it out")
                    .long_help(
                        "Once the synthesizer kills one of it's voices, it will fade it \
                        out as opposed to simply cutting it off",
                    )
                    .short('F')
                    .long("fade-out-kill")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("linear-envelope")
                    .help("Adjust the rate of decay on the synth's voices")
                    .long_help(
                        "Switch the synth's voice's rate of decay from \
                        exponential to linear. This may be bring a performance \
                        improvement on some systems.",
                    )
                    .short('L')
                    .long("linear-envelope")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("no-effects")
                    .help("Disables the soundfont's effects")
                    .long_help(
                        "Disables soundfont audio effects. \
                        This may improve the performance.",
                    )
                    .short('N')
                    .long("no-effects")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("note-speed")
                    .help("The speed that the notes travel on-screen")
                    .long_help(
                        "The speed at which the notes will move across the screen. This makes \
                        the notes physically longer, causing them to move faster on-screen",
                    )
                    .short('n')
                    .long("note-speed")
                    .value_parser(note_speed),
            )
            .arg(
                Arg::new("random-colors")
                    .help("Make each channel a random color")
                    .long_help("This causes each of the note channels to become a random color")
                    .short('r')
                    .long("random-colors")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("key-range")
                    .help("The key range of the on-screen piano keyboard")
                    .long_help(
                        "Two numbers, comma seperated, that describe the range \
                        of keys to be shown on the on-screen piano keyboard, the range must \
                        be less than 255 and more than 0",
                    )
                    .short('k')
                    .long("key-range")
                    .value_parser(range_parser),
            )
            .arg(
                Arg::new("midi-loading")
                    .help("How the MIDI is loaded into `wasabi`")
                    .long_help(
                        "The method in which the MIDI file is loaded into `wasabi`, the \
                        two possible options are `ram`, which loads the MIDI file entirely into \
                        RAM before beginning playback; and `live` which will read the MIDI file \
                        as it's being played back. The latter method is for using with systems \
                        with low memory",
                    )
                    .short('m')
                    .long("midi-loading")
                    .value_parser(MidiLoading::from_str),
            )
            .arg(
                Arg::new("bg-color")
                    .help("The window background")
                    .long_help("A hex color string describing the background color of the window")
                    .long("bg-color")
                    .value_parser(color_parser),
            )
            .arg(
                Arg::new("bar-color")
                    .help("The color of the bar just above the piano")
                    .long_help(
                        "A hex color string describing the color of the bar just above \
                         the on-screen piano keyboard",
                    )
                    .long("bar-color")
                    .value_parser(color_parser),
            )
            .arg(
                Arg::new("hide-top-pannel")
                    .long_help(
                        "Hides the top panel from view when the app opens. It can be un-hidden \
                        with Ctrl+F",
                    )
                    .help("Hide the top panel")
                    .long("hide-top-pannel")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("hide-statistics")
                    .help("Hide the statistics window")
                    .long_help(
                        "Hides the statistics window from view when the app opens. It can be \
                        un-hidden with Ctrl+G",
                    )
                    .long("hide-statistics")
                    .action(ArgAction::SetFalse),
            )
            .arg(
                Arg::new("fullscreen")
                    .help("Start `wasabi` in fullscreen")
                    .long_help(
                        "Starts `wasabi` in fullscreen mode. `wasabi` will use \
                        borderless fullscreen mode on Linux systems running Wayland, \
                        and exclusive fullscreen mode for everyone else",
                    )
                    .short('f')
                    .long("fullscreen")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("midi-file")
                    .value_hint(ValueHint::FilePath)
                    .help("The MIDI file to immediately begin playing")
                    .long_help(
                        "This MIDI file is played immediately after the app's launch. \
                        This argument is required to use the `--audio-only` option",
                    ),
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
        set_flag!(synth.limit_layers, "dont-limit-layers");
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
        if let Some(base_dirs) = BaseDirs::new() {
            let mut path: PathBuf = base_dirs.config_dir().to_path_buf();
            path.push("wasabi");
            path.push(CONFIG_PATH);

            if std::fs::create_dir_all(path.parent().unwrap()).is_ok() {
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
