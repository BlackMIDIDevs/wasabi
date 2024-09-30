use std::sync::{Arc, RwLock};

use crate::{audio_playback::WasabiAudioPlayer, settings::WasabiSettings};

use super::SettingsWindow;

mod list;
pub use list::*;
mod cfg;
pub use cfg::*;

impl SettingsWindow {
    pub fn show_soundfont_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        width: f32,
        synth: Arc<RwLock<WasabiAudioPlayer>>,
    ) {
        self.sf_list.show(ui, settings, width, synth);
    }
}
