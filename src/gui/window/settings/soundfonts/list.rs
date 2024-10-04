use std::{path::PathBuf, sync::Arc, thread};

use crossbeam_channel::{Receiver, Sender};
use egui::WidgetText;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    audio_playback::WasabiAudioPlayer,
    gui::window::loading::LoadingStatus,
    settings::{WasabiSettings, WasabiSoundfont},
};

use super::SoundfontConfigWindow;

#[derive(Default, Clone, PartialEq, Serialize, Deserialize)]
pub enum SFFormat {
    #[default]
    Sfz,
    Sf2,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SFListItem {
    pub item: WasabiSoundfont,
    pub id: usize,
    pub selected: bool,
    pub format: SFFormat,
}

pub struct EguiSFList {
    list: Vec<SFListItem>,
    id_count: usize,

    sf_picker: (Sender<PathBuf>, Receiver<PathBuf>),
    sf_cfg_win: Vec<SoundfontConfigWindow>,
}

impl EguiSFList {
    pub fn new() -> Self {
        let sf_picker = crossbeam_channel::unbounded();

        Self {
            list: Vec::new(),
            id_count: 0,
            sf_picker,
            sf_cfg_win: Vec::new(),
        }
    }

    pub fn add_item(&mut self, sf: WasabiSoundfont) -> Result<(), String> {
        let err = Err(format!(
            "The selected soundfont does not have the correct format: {:?}",
            sf.path
        ));

        if let Some(ext) = sf.path.extension() {
            let format = match ext.to_str().unwrap().to_lowercase().as_str() {
                "sfz" => SFFormat::Sfz,
                "sf2" => SFFormat::Sf2,
                _ => return err,
            };

            let item = SFListItem {
                item: sf,
                id: self.id_count,
                selected: false,
                format,
            };
            self.list.push(item);
            self.id_count += 1;
            return Ok(());
        }

        err
    }

    pub fn add_path(&mut self, path: PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err(format!("File not found: {:?}", path));
        }

        let item = WasabiSoundfont {
            path,
            enabled: true,
            options: Default::default(),
        };

        self.add_item(item)
    }

    pub fn select_all(&mut self) {
        self.list = self
            .list
            .clone()
            .into_iter()
            .map(|mut item| {
                item.selected = true;
                item
            })
            .collect();
    }

    pub fn remove_selected_items(&mut self) {
        self.list = self
            .list
            .clone()
            .into_iter()
            .filter(|item| !item.selected)
            .collect();

        // I'm bored to make it close only the windows needed, so instead I'll close all of them
        self.sf_cfg_win.clear();
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.sf_cfg_win.clear();
    }

    pub fn as_vec(&self) -> Vec<WasabiSoundfont> {
        self.list.iter().map(|sf| sf.item.clone()).collect()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        synth: Arc<WasabiAudioPlayer>,
        loading_status: Arc<LoadingStatus>,
    ) {
        let events = ui.input(|i| i.events.clone());
        for event in &events {
            if let egui::Event::Key {
                key,
                modifiers,
                pressed,
                ..
            } = event
            {
                match *key {
                    egui::Key::A => {
                        if *pressed && modifiers.ctrl {
                            self.select_all();
                        }
                    }
                    egui::Key::Delete => {
                        self.remove_selected_items();
                    }
                    _ => {}
                }
            }
        }

        {
            let recv = self.sf_picker.1.clone();
            if !recv.is_empty() {
                for path in recv {
                    //state.last_location = path.clone();
                    if path.is_file() {
                        if let Err(error) = self.add_path(path.clone()) {
                            let title = if let Some(filen) = path.file_name() {
                                format!(
                                    "There was an error adding \"{}\" to the list.",
                                    filen.to_str().unwrap()
                                )
                            } else {
                                "There was an error adding the selected soundfont to the list."
                                    .to_string()
                            };
                            // TODO: errors
                        }
                    }
                    break;
                }
            }
        }

        self.sf_cfg_win = self
            .sf_cfg_win
            .clone()
            .into_iter()
            .filter(|item| item.visible)
            .collect();

        for cfg in self.sf_cfg_win.iter_mut() {
            let index = self.list.iter().position(|item| item.id == cfg.id());
            if let Some(index) = index {
                cfg.show(ui.ctx(), &mut self.list[index]);
            }
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            WidgetText::from(" \u{2795} ")
                                .text_style(egui::TextStyle::Name("monospace big".into())),
                        )
                        .on_hover_text("Add SoundFont(s)")
                        .clicked()
                    {
                        let sender = self.sf_picker.0.clone();
                        // TODO: Fix
                        //let last_location = state.last_location.clone();

                        thread::spawn(move || {
                            let midi_path = rfd::FileDialog::new()
                                .add_filter("Supported SoundFonts", &["sfz", "SFZ", "sf2", "SF2"])
                                //.set_directory(last_location.parent().unwrap_or(Path::new("./")))
                                .pick_file();

                            if let Some(midi_path) = midi_path {
                                sender.send(midi_path).unwrap_or_default();
                            }
                        });
                    }
                    if ui
                        .button(
                            WidgetText::from(" \u{2796} ")
                                .text_style(egui::TextStyle::Name("monospace big".into())),
                        )
                        .on_hover_text("Remove Selected")
                        .clicked()
                    {
                        self.remove_selected_items();
                    }
                    if ui
                        .button(
                            WidgetText::from(" \u{2716} ")
                                .text_style(egui::TextStyle::Name("monospace big".into())),
                        )
                        .on_hover_text("Clear List")
                        .clicked()
                    {
                        self.clear();
                    }
                    if ui
                        .button(
                            WidgetText::from(" \u{2705} ")
                                .text_style(egui::TextStyle::Name("monospace big".into())),
                        )
                        .on_hover_text("Apply SoundFont List")
                        .clicked()
                    {
                        synth.set_soundfonts(&settings.synth.soundfonts, loading_status);
                    }
                    // TODO: Rearrange list
                });
                ui.small("Loading order is top to bottom. Supported formats: SFZ, SF2");
            });

        egui::ScrollArea::both().show(ui, |ui| {
            let events = ui.input(|i| i.events.clone());
            for event in &events {
                if let egui::Event::Key {
                    key,
                    modifiers,
                    pressed,
                    ..
                } = event
                {
                    match *key {
                        egui::Key::A => {
                            if *pressed && modifiers.ctrl {
                                self.select_all();
                            }
                        }
                        egui::Key::Delete => {
                            self.remove_selected_items();
                        }
                        _ => {}
                    }
                }
            }

            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                ))
                .resizable(true)
                .column(Column::exact(20.0).resizable(false))
                .column(Column::remainder().at_least(50.0).clip(true))
                .columns(Column::auto().at_least(40.0).clip(true).resizable(false), 3)
                .header(20.0, |mut header| {
                    header.col(|_ui| {});
                    header.col(|ui| {
                        ui.strong("Filename");
                    });
                    header.col(|ui| {
                        ui.strong("Format");
                    });
                    header.col(|ui| {
                        ui.strong("Bank");
                    });
                    header.col(|ui| {
                        ui.strong("Preset");
                    });
                })
                .body(|mut body| {
                    let row_height = super::super::SPACING[1] * 3.0;
                    for item in self.list.iter_mut() {
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut item.item.enabled, "");
                            });
                            row.col(|ui| {
                                let selectable = if let Some(path) = item.item.path.to_str() {
                                    ui.selectable_label(item.selected, path)
                                } else {
                                    ui.selectable_label(item.selected, "error")
                                };

                                if selectable.clicked() {
                                    item.selected = !item.selected;
                                }
                                if selectable.double_clicked()
                                    && !self.sf_cfg_win.iter().any(|cfg| cfg.id() == item.id)
                                {
                                    self.sf_cfg_win.push(SoundfontConfigWindow::new(item.id))
                                }
                            });
                            row.col(|ui| {
                                ui.label(match item.format {
                                    SFFormat::Sfz => "SFZ",
                                    SFFormat::Sf2 => "SF2",
                                });
                            });

                            let bank_txt = if let Some(bank) = item.item.options.bank {
                                format!("{}", bank)
                            } else {
                                "None".to_owned()
                            };
                            row.col(|ui| {
                                ui.label(bank_txt.to_string());
                            });

                            let preset_txt = if let Some(preset) = item.item.options.preset {
                                format!("{}", preset)
                            } else {
                                "None".to_owned()
                            };
                            row.col(|ui| {
                                ui.label(preset_txt.to_string());
                            });
                        });
                    }
                });
            ui.allocate_space(ui.available_size());
        });

        settings.synth.soundfonts = self.as_vec();
    }
}
