#[cfg(target_os = "windows")]
use std::io::Write;

use crate::{gui::window::WasabiError, utils};

use super::*;
use kdmapi_rs::{KDMAPIStream, KDMAPI};

pub struct KdmapiPlayer {
    stream: KDMAPIStream,
    use_om_list: bool,
}

impl KdmapiPlayer {
    pub fn new() -> Result<Self, WasabiError> {
        let kdmapi = KDMAPI
            .as_ref()
            .map_err(|e| WasabiError::SynthError(format!("Failed to load KDMAPI: {e}")))?;

        let stream = kdmapi
            .open_stream()
            .map_err(|e| WasabiError::SynthError(format!("Failed to load KDMAPI: {e}")))?;

        Ok(Self {
            stream,
            use_om_list: false,
        })
    }
}

impl MidiAudioPlayer for KdmapiPlayer {
    fn reset(&mut self) {
        let reset = utils::create_reset_midi_messages();
        for ev in reset {
            self.push_event(ev);
        }

        self.stream.reset();
    }

    fn push_event(&mut self, data: u32) {
        self.stream.send_direct_data(data);
    }

    fn voice_count(&self) -> Option<u64> {
        None
    }

    fn configure(&mut self, settings: &SynthSettings) {
        self.use_om_list = settings.kdmapi.use_om_sflist;
    }

    #[allow(unused_variables)]
    fn set_soundfonts(
        &mut self,
        soundfonts: &Vec<WasabiSoundfont>,
        _loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) {
        // Due to Windows using UTF16 formatting, this is currently only available for
        // the official Windows OmniMIDI.
        #[cfg(target_os = "windows")]
        if !self.use_om_list {
            let list = utils::create_om_sf_list(soundfonts);

            let mut path = WasabiSettings::get_config_dir();
            path.push("wasabi-sflist.csflist");

            if let Ok(mut file) = std::fs::File::create(&path) {
                file.write_all(list.as_bytes()).unwrap_or_else(|e| {
                    errors.warning(format!(
                        "Failed to create SoundFont list for OmniMIDI: {}",
                        e.to_string()
                    ))
                });
            }

            if !self
                .stream
                .load_custom_soundfonts_list(path.to_str().unwrap_or_default())
            {
                let error = WasabiError::SynthError(
                    "Failed to load custom SoundFont list in OmniMIDI.".into(),
                );
                errors.error(&error);
            }
        }
    }
}
