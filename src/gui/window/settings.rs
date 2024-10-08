use std::path::PathBuf;

use soundfonts::EguiSFList;

use crate::{
    settings::{Colors, Synth, WasabiSettings},
    state::{SettingsTab, WasabiState},
    utils,
};

use super::WasabiError;

mod midi;
mod soundfonts;
mod synth;
mod visual;

const CATEG_SPACE: f32 = 26.0;
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
            sf_list.add_item(sf.clone(), false);
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
    ) {
        let frame = utils::create_window_frame(ctx);
        let win = ctx.available_rect();

        egui::Window::new("Settings")
            .resizable(true)
            .collapsible(false)
            .title_bar(true)
            .enabled(true)
            .frame(frame)
            .default_size([win.width() * 0.7, win.height() * 0.7])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .min_size([700.0, 400.0])
            .show(ctx, |ui| {
                egui::TopBottomPanel::top("settings_tab_selector")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.style_mut()
                            .text_styles
                            .get_mut(&egui::TextStyle::Button)
                            .unwrap()
                            .size = 18.0;

                        ui.columns(4, |columns| {
                            columns[0].vertical_centered_justified(|ui| {
                                ui.selectable_value(
                                    &mut state.settings_tab,
                                    SettingsTab::Visual,
                                    "\u{1f4bb} Visual",
                                )
                            });
                            columns[1].vertical_centered_justified(|ui| {
                                ui.selectable_value(
                                    &mut state.settings_tab,
                                    SettingsTab::Midi,
                                    "\u{1f3b5} MIDI",
                                )
                            });
                            columns[2].vertical_centered_justified(|ui| {
                                ui.selectable_value(
                                    &mut state.settings_tab,
                                    SettingsTab::Synth,
                                    "\u{1f3b9} Synth",
                                )
                            });
                            columns[3].vertical_centered_justified(|ui| {
                                ui.add_enabled_ui(
                                    settings.synth.synth == Synth::XSynth
                                        || (settings.synth.synth == Synth::Kdmapi
                                            && !settings.synth.kdmapi.use_om_sflist),
                                    |ui| {
                                        ui.selectable_value(
                                            &mut state.settings_tab,
                                            SettingsTab::SoundFonts,
                                            "\u{1f50a} SoundFonts",
                                        )
                                    },
                                )
                            });
                        });
                        ui.add_space(8.0);
                    });

                egui::TopBottomPanel::bottom("settings_save_panel")
                    .resizable(false)
                    .show_inside(ui, |ui| {
                        ui.add_space(8.0);
                        ui.columns(2, |columns| {
                            columns[0].with_layout(
                                egui::Layout::top_down(egui::Align::RIGHT),
                                |ui| {
                                    if ui.button("\u{1F4BE} Save").clicked() {
                                        settings
                                            .save_to_file()
                                            .unwrap_or_else(|e| state.errors.error(&e));
                                    }
                                },
                            );
                            columns[1].with_layout(
                                egui::Layout::top_down(egui::Align::LEFT),
                                |ui| {
                                    if ui.button("\u{2716} Close").clicked() {
                                        state.show_settings = false;
                                    }
                                },
                            );
                        });
                    });

                let width = ui.available_width() - 40.0;
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().animated(true).show(ui, |ui| {
                        match state.settings_tab {
                            SettingsTab::Visual => self.show_visual_settings(ui, settings, width),
                            SettingsTab::Midi => {
                                self.show_midi_settings(ui, settings, state, width)
                            }
                            SettingsTab::Synth => {
                                self.show_synth_settings(ui, settings, state, width)
                            }
                            SettingsTab::SoundFonts => self.sf_list.show(ui, settings, state),
                        }
                    })
                });
            });
    }

    pub fn load_palettes(&mut self, settings: &mut WasabiSettings) -> Result<(), WasabiError> {
        self.palettes.clear();

        let files = std::fs::read_dir(WasabiSettings::get_palettes_dir())
            .map_err(WasabiError::FilesystemError)?;

        for file in files.filter_map(|i| i.ok()) {
            if let Ok(ftype) = file.file_type() {
                if ftype.is_file() {
                    let path = file.path();
                    let selected = settings.midi.palette_path == path
                        && settings.midi.colors == Colors::Palette;

                    self.palettes.push(FilePalette { path, selected });
                }
            }
        }

        Ok(())
    }

    pub fn load_midi_devices(&mut self, settings: &mut WasabiSettings) -> Result<(), WasabiError> {
        self.midi_devices.clear();
        let con = midir::MidiOutput::new("wasabi")
            .map_err(|e| WasabiError::SynthError(format!("{e:?}")))?;

        // Add all valid ports
        for port in con.ports().iter() {
            let name = con
                .port_name(&port)
                .map_err(|e| WasabiError::SynthError(format!("{e:?}")))?;
            self.midi_devices.push(MidiDevice {
                name,
                selected: false,
            });
        }

        // Select the device specified in settings if found, or select the first available
        let saved = settings.synth.midi_device.clone();
        if let Some(found) = self.midi_devices.iter_mut().find(|d| d.name == saved) {
            found.selected = true;
        } else if !self.midi_devices.is_empty() {
            self.midi_devices[0].selected = true;
            settings.synth.midi_device = self.midi_devices[0].name.clone();
        }

        Ok(())
    }
}
