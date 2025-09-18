use num_enum::FromPrimitive;
use serde_derive::{Deserialize, Serialize};
use std::{fmt::Debug, slice::Iter, str::FromStr};

#[repr(usize)]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum MidiParsing {
    #[default]
    Ram = 0,
    Live = 1,
    Cake = 2,
}

impl MidiParsing {
    pub const fn as_str(self) -> &'static str {
        match self {
            MidiParsing::Ram => "Standard (RAM)",
            MidiParsing::Live => "Standard (Live)",
            MidiParsing::Cake => "Cake",
        }
    }
}

impl FromStr for MidiParsing {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ram" => Ok(MidiParsing::Ram),
            "live" => Ok(MidiParsing::Live),
            "cake" => Ok(MidiParsing::Cake),
            s => Err(format!(
                "{} was not expected. Expected one of `ram`, `live` or `cake`",
                s
            )),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[repr(usize)]
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum Synth {
    #[default]
    XSynth = 0,
    #[cfg(supported_os)]
    Kdmapi = 1,
    #[cfg(all(supported_os, not(target_os = "freebsd")))]
    MidiDevice = 2,
    None = 3,
}

impl Synth {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Synth::XSynth => "Built-In (XSynth)",
            #[cfg(supported_os)]
            Synth::Kdmapi => "KDMAPI",
            #[cfg(all(supported_os, not(target_os = "freebsd")))]
            Synth::MidiDevice => "MIDI Device",
            Synth::None => "None",
        }
    }
}

impl FromStr for Synth {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "xsynth" => Ok(Synth::XSynth),
            #[cfg(supported_os)]
            "kdmapi" => Ok(Synth::Kdmapi),
            #[cfg(supported_os)]
            "mididevice" => Ok(Synth::MidiDevice),
            "none" => Ok(Synth::None),
            s => Err(format!(
                "{} was not expected. Expected one of `xsynth`, `kdmapi`, `mididevice` or `none`",
                s
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(usize)]
#[serde(rename_all = "lowercase")]
pub enum Statistics {
    #[default]
    Time = 0,
    Fps = 1,
    VoiceCount = 2,
    Rendered = 3,
    NoteCount = 4,
    Polyphony = 5,
    Nps = 6,
}

impl Statistics {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Statistics::Time => "Time",
            Statistics::Fps => "FPS",
            Statistics::VoiceCount => "Voice Count",
            Statistics::Rendered => "Rendered",
            Statistics::NoteCount => "Note Count",
            Statistics::Polyphony => "Polyphony",
            Statistics::Nps => "NPS",
        }
    }

    pub fn iter() -> Iter<'static, Statistics> {
        static STATISTICS: [Statistics; 7] = [
            Statistics::Time,
            Statistics::Fps,
            Statistics::Rendered,
            Statistics::Nps,
            Statistics::Polyphony,
            Statistics::VoiceCount,
            Statistics::NoteCount,
        ];
        STATISTICS.iter()
    }
}

impl FromStr for Statistics {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "time" => Ok(Statistics::Time),
            "fps" => Ok(Statistics::Fps),
            "voicecount" => Ok(Statistics::VoiceCount),
            "rendered" => Ok(Statistics::Rendered),
            "notecount" => Ok(Statistics::NoteCount),
            "polyphony" => Ok(Statistics::Polyphony),
            "nps" => Ok(Statistics::Nps),
            s => Err(format!("{} was not expected.", s)),
        }
    }
}

#[repr(usize)]
#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[serde(rename_all = "lowercase")]
pub enum Colors {
    #[default]
    Rainbow = 0,
    Random = 1,
    Palette = 2,
}

impl Colors {
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Colors::Rainbow => "Rainbow",
            Colors::Random => "Random",
            Colors::Palette => "Palette",
        }
    }
}

impl FromStr for Colors {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rainbow" => Ok(Colors::Rainbow),
            "random" => Ok(Colors::Random),
            "palette" => Ok(Colors::Palette),
            s => Err(format!(
                "{} was not expected. Expected one of `ranbow`, `random` or `palette`",
                s
            )),
        }
    }
}
