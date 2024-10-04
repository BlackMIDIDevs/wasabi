use std::sync::{Arc, RwLock};

use crate::{
    gui::window::LoadingStatus,
    settings::{SynthSettings, WasabiSoundfont},
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
    );
    fn reset(&mut self);
}

pub struct WasabiAudioPlayer {
    player: RwLock<Box<dyn MidiAudioPlayer>>,
}

impl WasabiAudioPlayer {
    pub fn new(player: Box<dyn MidiAudioPlayer>) -> Self {
        Self {
            player: RwLock::new(player),
        }
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

    pub fn set_soundfonts(
        &self,
        soundfonts: &Vec<WasabiSoundfont>,
        loading_status: Arc<LoadingStatus>,
    ) {
        self.player
            .write()
            .unwrap()
            .set_soundfonts(soundfonts, loading_status);
    }

    pub fn reset(&self) {
        self.player.write().unwrap().reset();
    }
}
