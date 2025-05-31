use std::sync::{Arc, RwLock};

use crate::{
    gui::window::{GuiMessageSystem, LoadingStatus},
    settings::{Synth, SynthSettings, WasabiSoundfont},
};

mod xsynth;
pub use xsynth::*;

#[cfg(supported_os)]
mod kdmapi;
#[cfg(supported_os)]
pub use kdmapi::*;

#[cfg(supported_os)]
mod midiout;
#[cfg(supported_os)]
pub use midiout::*;

enum MidiAudioPlayer {
    XSynth(XSynthPlayer),
    #[cfg(supported_os)]
    Kdmapi(KdmapiPlayer),
    #[cfg(supported_os)]
    MidiDevice(MidiDevicePlayer),
    None,
}

pub struct WasabiAudioPlayer(RwLock<MidiAudioPlayer>);

impl WasabiAudioPlayer {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self(RwLock::new(MidiAudioPlayer::None)))
    }

    pub fn voice_count(&self) -> Option<u64> {
        match &*self.0.read().unwrap() {
            MidiAudioPlayer::XSynth(player) => Some(player.voice_count()),
            _ => None,
        }
    }

    pub fn push_events(&self, data: impl Iterator<Item = u32>) {
        match &mut *self.0.write().unwrap() {
            MidiAudioPlayer::XSynth(player) => player.push_events(data),
            #[cfg(supported_os)]
            MidiAudioPlayer::Kdmapi(player) => player.push_events(data),
            #[cfg(supported_os)]
            MidiAudioPlayer::MidiDevice(player) => player.push_events(data),
            _ => {}
        }
    }

    pub fn configure(&self, settings: &SynthSettings) {
        match &mut *self.0.write().unwrap() {
            MidiAudioPlayer::XSynth(player) => player.configure(&settings.xsynth),
            #[cfg(supported_os)]
            MidiAudioPlayer::Kdmapi(player) => player.configure(&settings.kdmapi),
            _ => {}
        }
    }

    pub fn set_soundfonts(
        &self,
        soundfonts: &[WasabiSoundfont],
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) {
        match &mut *self.0.write().unwrap() {
            MidiAudioPlayer::XSynth(player) => {
                player.set_soundfonts(soundfonts, loading_status, errors)
            }
            #[cfg(supported_os)]
            MidiAudioPlayer::Kdmapi(player) => player.set_soundfonts(soundfonts, errors),
            _ => {}
        }
    }

    pub fn reset(&self) {
        match &mut *self.0.write().unwrap() {
            MidiAudioPlayer::XSynth(player) => player.reset(),
            #[cfg(supported_os)]
            MidiAudioPlayer::Kdmapi(player) => player.reset(),
            #[cfg(supported_os)]
            MidiAudioPlayer::MidiDevice(player) => player.reset(),
            _ => {}
        }
    }

    pub fn switch(
        &self,
        settings: &SynthSettings,
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) {
        // First drop the previous synth to avoid any loading errors
        *self.0.write().unwrap() = MidiAudioPlayer::None;

        // Create the new synth object based on the settings
        let synth = match settings.synth {
            Synth::XSynth => {
                MidiAudioPlayer::XSynth(XSynthPlayer::new(settings.xsynth.config.clone()))
            }
            #[cfg(supported_os)]
            Synth::Kdmapi => match KdmapiPlayer::new() {
                Ok(kdmapi) => MidiAudioPlayer::Kdmapi(kdmapi),
                Err(e) => {
                    errors.error(&e);
                    MidiAudioPlayer::None
                }
            },
            #[cfg(supported_os)]
            Synth::MidiDevice => match MidiDevicePlayer::new(settings.midi_device.clone()) {
                Ok(midiout) => MidiAudioPlayer::MidiDevice(midiout),
                Err(e) => {
                    errors.error(&e);
                    MidiAudioPlayer::None
                }
            },
            Synth::None => MidiAudioPlayer::None,
        };

        // Apply the synth to the struct
        *self.0.write().unwrap() = synth;

        // Configure the synth and load the soundfont list
        self.configure(settings);
        self.set_soundfonts(&settings.soundfonts, loading_status, errors);
    }
}
