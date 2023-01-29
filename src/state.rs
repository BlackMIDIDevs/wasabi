use std::path::PathBuf;

#[derive(Clone)]
pub struct WasabiState {
    pub fullscreen: bool,
    pub panel_visible: bool,
    pub stats_visible: bool,
    pub settings_visible: bool,
    pub xsynth_settings_visible: bool,
    pub last_midi_file: Option<PathBuf>,
    pub last_sfz_file: Option<PathBuf>,
}

impl Default for WasabiState {
    fn default() -> Self {
        Self {
            fullscreen: false,
            panel_visible: true,
            stats_visible: true,
            settings_visible: false,
            xsynth_settings_visible: false,
            last_midi_file: None,
            last_sfz_file: None,
        }
    }
}
