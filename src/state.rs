use std::path::PathBuf;

#[derive(Default, PartialEq)]
pub enum SettingsTab {
    #[default]
    Visual,
    Midi,
    Synth,
    SoundFonts,
}

pub struct WasabiState {
    pub fullscreen: bool,

    pub panel_pinned: bool,
    pub panel_popup_id: egui::Id,
    pub stats_visible: bool,

    pub show_settings: bool,
    pub show_shortcuts: bool,
    pub show_about: bool,

    pub settings_tab: SettingsTab,

    pub last_location: PathBuf,
}

impl Default for WasabiState {
    fn default() -> Self {
        Self {
            fullscreen: false,

            panel_pinned: true,
            panel_popup_id: egui::Id::new("options_popup"),
            stats_visible: true,

            show_settings: false,
            show_shortcuts: false,
            show_about: false,

            settings_tab: SettingsTab::default(),

            last_location: PathBuf::default(),
        }
    }
}
