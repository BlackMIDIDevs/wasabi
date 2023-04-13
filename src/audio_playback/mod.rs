use kdmapi::{KDMAPIStream, KDMAPI};
use std::{
    ops::RangeInclusive,
    path::PathBuf,
    sync::{Arc, RwLock},
};
use xsynth_core::{channel::ChannelInitOptions, soundfont::SoundfontInitOptions};

use crate::{
    midi::{InRamMIDIFile, LiveLoadMIDIFile, MIDIFileBase, MIDIFileUnion},
    settings::{MidiLoading, Synth, WasabiSettings},
};

use self::xsynth::{convert_to_channel_init, convert_to_sf_init};
pub mod xsynth;

pub struct ManagedSynth {
    pub midi_file: Option<MIDIFileUnion>,
    pub player: Arc<RwLock<SimpleTemporaryPlayer>>,
}

impl ManagedSynth {
    pub fn new(settings: &mut WasabiSettings) -> Self {
        Self {
            midi_file: None,
            player: match settings.synth.synth {
                Synth::Kdmapi => Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                    AudioPlayerType::Kdmapi,
                ))),
                Synth::XSynth => {
                    let synth = Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                        AudioPlayerType::XSynth {
                            buffer: settings.synth.buffer_ms,
                            ignore_range: settings.synth.vel_ignore.clone(),
                            options: convert_to_channel_init(settings),
                        },
                    )));
                    synth
                        .write()
                        .unwrap()
                        .set_soundfont(&settings.synth.sfz_path, convert_to_sf_init(settings));
                    synth
                        .write()
                        .unwrap()
                        .set_layer_count(match settings.synth.layer_count {
                            0 => None,
                            _ => Some(settings.synth.layer_count),
                        });
                    synth
                }
            },
        }
    }

    pub fn load_midi(&mut self, settings: &mut WasabiSettings, midi_path: PathBuf) {
        if let Some(midi_file) = self.midi_file.as_mut() {
            midi_file.timer_mut().pause();
        }
        self.player.write().unwrap().reset();
        self.midi_file = None;

        if let Some(midi_path) = midi_path.to_str() {
            match settings.midi.midi_loading {
                MidiLoading::Ram => {
                    let mut midi_file = MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
                        midi_path,
                        self.player.clone(),
                        settings.midi.random_colors,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
                MidiLoading::Live => {
                    let mut midi_file = MIDIFileUnion::Live(LiveLoadMIDIFile::load_from_file(
                        midi_path,
                        self.player.clone(),
                        settings.midi.random_colors,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum AudioPlayerType {
    XSynth {
        buffer: f64,
        ignore_range: RangeInclusive<u8>,
        options: ChannelInitOptions,
    },
    Kdmapi,
}

pub struct SimpleTemporaryPlayer {
    player_type: AudioPlayerType,
    xsynth: Option<xsynth::XSynthPlayer>,
    kdmapi: Option<KDMAPIStream>,
}

impl SimpleTemporaryPlayer {
    pub fn new(player_type: AudioPlayerType) -> Self {
        let (xsynth, kdmapi) = match player_type.clone() {
            AudioPlayerType::XSynth {
                buffer,
                ignore_range,
                options,
            } => {
                let xsynth = xsynth::XSynthPlayer::new(buffer, ignore_range, options);
                (Some(xsynth), None)
            }
            AudioPlayerType::Kdmapi => {
                let kdmapi = KDMAPI.open_stream();
                (None, Some(kdmapi))
            }
        };
        Self {
            player_type,
            xsynth,
            kdmapi,
        }
    }

    pub fn switch_player(&mut self, player_type: AudioPlayerType) {
        self.reset();
        self.xsynth = None;
        self.kdmapi = None;
        let new_player = Self::new(player_type);

        self.player_type = new_player.player_type;
        self.xsynth = new_player.xsynth;
        self.kdmapi = new_player.kdmapi;
    }

    pub fn get_voice_count(&self) -> u64 {
        match self.player_type {
            AudioPlayerType::XSynth { .. } => {
                if let Some(xsynth) = &self.xsynth {
                    xsynth.get_voice_count()
                } else {
                    0
                }
            }
            AudioPlayerType::Kdmapi => 0,
        }
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for e in data {
            self.push_event(e);
        }
    }

    pub fn push_event(&mut self, data: u32) {
        match self.player_type {
            AudioPlayerType::XSynth { .. } => {
                if let Some(xsynth) = self.xsynth.as_mut() {
                    xsynth.push_event(data);
                }
            }
            AudioPlayerType::Kdmapi => {
                if let Some(kdmapi) = self.kdmapi.as_mut() {
                    kdmapi.send_direct_data(data);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        match self.player_type {
            AudioPlayerType::XSynth { .. } => {
                if let Some(xsynth) = self.xsynth.as_mut() {
                    xsynth.reset();
                }
            }
            AudioPlayerType::Kdmapi => {
                if let Some(kdmapi) = self.kdmapi.as_mut() {
                    kdmapi.reset();
                }
            }
        }
    }

    pub fn set_layer_count(&mut self, layers: Option<usize>) {
        if let AudioPlayerType::XSynth { .. } = self.player_type {
            if let Some(xsynth) = self.xsynth.as_mut() {
                xsynth.set_layer_count(layers);
            }
        }
    }

    pub fn set_soundfont(&mut self, path: &str, options: SoundfontInitOptions) {
        if let AudioPlayerType::XSynth { .. } = self.player_type {
            if let Some(xsynth) = self.xsynth.as_mut() {
                xsynth.set_soundfont(path, options);
            }
        }
    }
}
