use crate::settings::{SynthSettings, WasabiSoundfont};

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
    fn set_soundfonts(&mut self, soundfonts: &Vec<WasabiSoundfont>);
    fn reset(&mut self);
}

pub struct WasabiAudioPlayer {
    player: Box<dyn MidiAudioPlayer>,
}

impl WasabiAudioPlayer {
    pub fn new(player: Box<dyn MidiAudioPlayer>) -> Self {
        Self { player: player }
    }

    pub fn switch(&mut self, new_player: Box<dyn MidiAudioPlayer>) {
        self.player = new_player;
    }

    pub fn voice_count(&self) -> u64 {
        self.player.voice_count()
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for ev in data {
            self.player.push_event(ev);
        }
    }

    pub fn configure(&mut self, settings: &SynthSettings) {
        self.player.configure(settings);
    }

    pub fn set_soundfonts(&mut self, soundfonts: &Vec<WasabiSoundfont>) {
        self.player.set_soundfonts(soundfonts);
    }

    pub fn reset(&mut self) {
        self.player.reset();
    }
}
