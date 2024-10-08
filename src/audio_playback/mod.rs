use std::sync::{Arc, RwLock};

use crate::{
    gui::window::{GuiMessageSystem, LoadingStatus},
    settings::{Synth, SynthSettings, WasabiSoundfont},
    state::WasabiState,
};

mod xsynth;
pub use xsynth::*;
mod kdmapi;
pub use kdmapi::*;
mod midiout;
pub use midiout::*;
mod empty;
pub use empty::*;

pub trait MidiAudioPlayer: Send + Sync {
    fn voice_count(&self) -> Option<u64>;
    fn push_event(&mut self, data: u32);
    fn configure(&mut self, settings: &SynthSettings);
    fn set_soundfonts(
        &mut self,
        soundfonts: &Vec<WasabiSoundfont>,
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    );
    fn reset(&mut self);
}

pub struct WasabiAudioPlayer {
    player: RwLock<Box<dyn MidiAudioPlayer>>,
}

impl WasabiAudioPlayer {
    pub fn empty() -> Arc<Self> {
        Arc::new(Self {
            player: RwLock::new(Box::new(EmptyPlayer::new())),
        })
    }

    pub fn voice_count(&self) -> Option<u64> {
        self.player.read().unwrap().voice_count()
    }

    pub fn push_events(&self, data: impl Iterator<Item = u32>) {
        for ev in data {
            self.player.write().unwrap().push_event(ev);
        }
    }

    pub fn configure(&self, settings: &SynthSettings) {
        self.player.write().unwrap().configure(settings);
    }

    pub fn set_soundfonts(&self, soundfonts: &Vec<WasabiSoundfont>, state: &WasabiState) {
        self.player.write().unwrap().set_soundfonts(
            soundfonts,
            state.loading_status.clone(),
            state.errors.clone(),
        );
    }

    pub fn reset(&self) {
        self.player.write().unwrap().reset();
    }

    pub fn switch(
        &self,
        settings: &SynthSettings,
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) {
        // First drop the previous synth to avoid any loading errors
        *self.player.write().unwrap() = Box::new(EmptyPlayer::new());

        // Create the new synth object based on the settings
        let mut synth: Box<dyn MidiAudioPlayer> = match settings.synth {
            Synth::XSynth => Box::new(XSynthPlayer::new(settings.xsynth.config.clone())),
            Synth::Kdmapi => match KdmapiPlayer::new() {
                Ok(kdmapi) => Box::new(kdmapi),
                Err(e) => {
                    errors.error(&e);
                    Box::new(EmptyPlayer::new())
                }
            },
            Synth::MidiDevice => match MidiDevicePlayer::new(settings.midi_device.clone()) {
                Ok(midiout) => Box::new(midiout),
                Err(e) => {
                    errors.error(&e);
                    Box::new(EmptyPlayer::new())
                }
            },
            Synth::None => Box::new(EmptyPlayer::new()),
        };

        // Configure the synth and load the soundfont list
        synth.configure(&settings);
        synth.set_soundfonts(&settings.soundfonts, loading_status, errors);

        // Apply the synth to the struct
        *self.player.write().unwrap() = synth;
    }
}
