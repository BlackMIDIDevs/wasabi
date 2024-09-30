use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use soundfonts::EguiSFList;

use crate::{
    audio_playback::WasabiAudioPlayer,
    settings::WasabiSettings,
    state::{SettingsTab, WasabiState},
};

mod midi;
mod soundfonts;
mod synth;
mod visual;

const CATEG_SPACE: f32 = 12.0;
const SPACING: [f32; 2] = [40.0, 12.0];

#[derive(Clone)]
struct FilePalette {
    pub path: PathBuf,
    pub selected: bool,
}

#[derive(Clone)]
struct MidiDevice {
    pub name: String,
    pub selected: bool,
}

pub struct SettingsWindow {
    palettes: Vec<FilePalette>,
    midi_devices: Vec<MidiDevice>,
    sf_list: EguiSFList,
}

impl SettingsWindow {
    pub fn new(settings: &WasabiSettings) -> Self {
        let mut sf_list = EguiSFList::new();
        for sf in settings.synth.soundfonts.iter() {
            sf_list.add_item(sf.clone()).unwrap_or_default();
        }

        Self {
            palettes: Vec::new(),
            midi_devices: Vec::new(),
            sf_list,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        settings: &mut WasabiSettings,
        state: &mut WasabiState,
        synth: Arc<RwLock<WasabiAudioPlayer>>,
    ) {
        let frame =
            egui::Frame::inner_margin(egui::Frame::window(ctx.style().as_ref()), super::WIN_MARGIN);

        egui::Window::new("Settings")
            .resizable(true)
            .collapsible(false)
            .title_bar(true)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .movable(false)
            .enabled(true)
            .frame(frame)
            .default_size([600.0, 400.0])
            .min_size([500.0, 200.0])
            .open(&mut state.show_settings)
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("settings_tab_selector")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.style_mut()
                            .text_styles
                            .get_mut(&egui::TextStyle::Button)
                            .unwrap()
                            .size = 20.0;

                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut state.settings_tab,
                                SettingsTab::Visual,
                                "\u{1f4bb} Visual",
                            );
                            ui.selectable_value(
                                &mut state.settings_tab,
                                SettingsTab::Midi,
                                "\u{1f3b5} MIDI",
                            );
                            ui.selectable_value(
                                &mut state.settings_tab,
                                SettingsTab::Synth,
                                "\u{1f3b9} Synth",
                            );
                            ui.selectable_value(
                                &mut state.settings_tab,
                                SettingsTab::SoundFonts,
                                "\u{1f50a} SoundFonts",
                            );
                        });
                        ui.add_space(4.0);
                    });

                egui::TopBottomPanel::bottom("settings_save_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.add_space(4.0);
                        ui.centered_and_justified(|ui| {
                            if ui.button("Save").clicked() {
                                settings.save_to_file();
                            }
                        });
                    });

                let width = ui.available_width() - 40.0;
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().animated(true).show(ui, |ui| {
                        match state.settings_tab {
                            SettingsTab::Visual => self.show_visual_settings(ui, settings, width),
                            SettingsTab::Midi => self.show_midi_settings(ui, settings, width),
                            SettingsTab::Synth => {
                                self.show_synth_settings(ui, settings, width, synth)
                            }
                            SettingsTab::SoundFonts => {
                                self.show_soundfont_settings(ui, settings, width, synth)
                            }
                        }
                    })
                });
            });
    }

    pub fn load_palettes(&mut self) {
        self.palettes.clear();

        let files = std::fs::read_dir(WasabiSettings::get_palettes_dir()).unwrap();

        for file in files.filter_map(|i| i.ok()) {
            if let Ok(ftype) = file.file_type() {
                if ftype.is_file() {
                    self.palettes.push(FilePalette {
                        path: file.path(),
                        selected: false,
                    });
                }
            }
        }
    }

    pub fn load_midi_devices(&mut self, settings: &mut WasabiSettings) {
        self.midi_devices.clear();
        if let Ok(con) = midir::MidiOutput::new("wasabi") {
            let ports = con.ports();
            for port in ports.iter() {
                if let Ok(name) = con.port_name(&port) {
                    self.midi_devices.push(MidiDevice {
                        name,
                        selected: false,
                    });
                }
            }
        }

        let saved = settings.synth.midi_device.clone();
        if let Some(found) = self.midi_devices.iter_mut().find(|d| d.name == saved) {
            found.selected = true;
            return;
        }

        if !self.midi_devices.is_empty() {
            self.midi_devices[0].selected = true;
            settings.synth.midi_device = self.midi_devices[0].name.clone();
        }
    }
}