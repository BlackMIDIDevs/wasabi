use std::sync::{Arc, RwLock};

use crate::{
    gui::window::{GuiMessageSystem, LoadingStatus},
    settings::{Synth, SynthSettings, WasabiSettings, WasabiSoundfont},
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
    fn voice_count(&self) -> u64;
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
    pub fn new(player: Box<dyn MidiAudioPlayer>) -> Arc<Self> {
        Arc::new(Self {
            player: RwLock::new(player),
        })
    }

    pub fn switch(&self, new_player: Box<dyn MidiAudioPlayer>) {
        *self.player.write().unwrap() = new_player;
    }

    pub fn voice_count(&self) -> u64 {
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

    pub fn create_synth(
        settings: &WasabiSettings,
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) -> Box<dyn MidiAudioPlayer> {
        let mut synth: Box<dyn MidiAudioPlayer> = match settings.synth.synth {
            Synth::XSynth => Box::new(XSynthPlayer::new(settings.synth.xsynth.config.clone())),
            Synth::Kdmapi => Box::new(KdmapiPlayer::new()),
            Synth::MidiDevice => {
                if let Ok(midiout) = MidiDevicePlayer::new(settings.synth.midi_device.clone()) {
                    Box::new(midiout)
                } else {
                    Box::new(EmptyPlayer::new())
                }
            }
            Synth::None => Box::new(EmptyPlayer::new()),
        };
        synth.set_soundfonts(&settings.synth.soundfonts, loading_status, errors);
        synth.configure(&settings.synth);

        synth
    }
}
