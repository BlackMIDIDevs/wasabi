use crate::settings::{KdmapiSettings, WasabiSettings};
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

    pub fn reset(&mut self) {
        let reset = utils::create_reset_midi_messages();
        self.push_events(reset.into_iter());
        self.stream.reset();
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for ev in data {
            self.stream.send_direct_data(ev);
        }
    }

    pub fn configure(&mut self, settings: &KdmapiSettings) {
        self.use_om_list = settings.use_om_sflist;
    }

    pub fn set_soundfonts(
        &mut self,
        soundfonts: &[WasabiSoundfont],
        errors: Arc<GuiMessageSystem>,
    ) {
        if !self.use_om_list {
            let list = utils::create_om_sf_list(soundfonts);

            let mut path = WasabiSettings::get_config_dir();
            path.push("wasabi-sflist.csflist");

            if let Ok(mut file) = std::fs::File::create(&path) {
                file.write_all(list.as_bytes()).unwrap_or_else(|e| {
                    errors.warning(format!(
                        "Failed to create SoundFont list for OmniMIDI: {}",
                        e
                    ))
                });
            }

            if let Some(false) = self
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

    pub fn voice_count(&self) -> Option<u64> {
        self.stream.get_voice_count()
    }
}
