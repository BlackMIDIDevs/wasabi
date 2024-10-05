use std::{path::PathBuf, sync::Arc};

use crate::{
    audio_playback::{EmptyPlayer, MidiAudioPlayer, WasabiAudioPlayer},
    gui::window::{GuiMessageSystem, LoadingStatus},
};

#[derive(Default, PartialEq)]
pub enum SettingsTab {
    #[default]
    Visual,
    Midi,
    Synth,
    SoundFonts,
}

pub struct WasabiState {
    pub synth: Arc<WasabiAudioPlayer>,

    pub fullscreen: bool,

    pub errors: Arc<GuiMessageSystem>,
    pub loading_status: Arc<LoadingStatus>,

    pub panel_pinned: bool,
    pub panel_id: egui::Id,
    pub panel_popup_id: egui::Id,
    pub stats_visible: bool,

    pub show_settings: bool,
    pub show_shortcuts: bool,
    pub show_about: bool,

    pub settings_tab: SettingsTab,

    pub last_midi_location: PathBuf,
    pub last_sf_location: PathBuf,
}

impl WasabiState {
    pub fn new() -> Self {
        let loading_status = LoadingStatus::new();
        let errors = GuiMessageSystem::new();
        let synth: Box<dyn MidiAudioPlayer> = Box::new(EmptyPlayer::new());

        Self {
            synth: WasabiAudioPlayer::new(synth),

            fullscreen: false,

            errors,
            loading_status,

            panel_pinned: true,
            panel_id: egui::Id::new("playback_panel"),
            panel_popup_id: egui::Id::new("options_popup"),
            stats_visible: true,

            show_settings: false,
            show_shortcuts: false,
            show_about: false,

            settings_tab: SettingsTab::default(),

            last_midi_location: PathBuf::default(),
            last_sf_location: PathBuf::default(),
        }
    }
}
