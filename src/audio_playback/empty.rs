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

    fn voice_count(&self) -> Option<u64> {
        None
    }

    fn configure(&mut self, _settings: &SynthSettings) {}

    fn set_soundfonts(
        &mut self,
        _soundfonts: &[WasabiSoundfont],
        _loading_status: Arc<LoadingStatus>,
        _errors: Arc<GuiMessageSystem>,
    ) {
    }
}
