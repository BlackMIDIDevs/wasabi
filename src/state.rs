use std::path::PathBuf;

#[derive(Clone, Default)]
pub struct WasabiState {
    pub fullscreen: bool,
    pub settings_visible: bool,
    pub xsynth_settings_visible: bool,
    pub last_midi_file: Option<PathBuf>,
    pub last_sfz_file: Option<PathBuf>,
}
