use super::*;

pub struct EmptyPlayer {}

impl EmptyPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl MidiAudioPlayer for EmptyPlayer {
    fn reset(&mut self) {}

    fn push_event(&mut self, _data: u32) {}

    fn voice_count(&self) -> u64 {
        0
    }

    fn configure(&mut self, _settings: &SynthSettings) {}

    fn set_soundfonts(
        &mut self,
        _soundfonts: &Vec<WasabiSoundfont>,
        _loading_status: Arc<LoadingStatus>,
    ) {
    }
}
