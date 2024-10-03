use std::sync::{Arc, RwLock};

use crate::{
    audio_playback::WasabiAudioPlayer, gui::window::loading::LoadingStatus,
    settings::WasabiSettings,
};

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
        synth: Arc<RwLock<WasabiAudioPlayer>>,
        loading_status: Arc<LoadingStatus>,
    ) {
        self.sf_list.show(ui, settings, synth, loading_status);
    }
}
