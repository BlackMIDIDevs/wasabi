use super::*;
use kdmapi_rs::{KDMAPIStream, KDMAPI};

pub struct KdmapiPlayer {
    stream: KDMAPIStream,
    use_om_list: bool,
}

impl KdmapiPlayer {
    pub fn new() -> Self {
        Self {
            stream: KDMAPI.open_stream(),
            use_om_list: false,
        }
    }
}

impl MidiAudioPlayer for KdmapiPlayer {
    fn reset(&mut self) {
        self.stream.reset();
    }

    fn push_event(&mut self, data: u32) {
        self.stream.send_direct_data(data);
    }

    fn voice_count(&self) -> u64 {
        0
    }

    fn configure(&mut self, settings: &SynthSettings) {
        self.use_om_list = settings.kdmapi.use_om_sflist;
    }

    fn set_soundfonts(
        &mut self,
        _soundfonts: &Vec<WasabiSoundfont>,
        _loading_status: Arc<LoadingStatus>,
        _errors: Arc<GuiMessageSystem>,
    ) {
        if !self.use_om_list {
            // TODO: Create OM compatible SF list to be sent through "LoadCustomSoundFontsList"
        }
    }
}
