use std::{
    io,
    path::{Path, PathBuf},
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use egui::WidgetText;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    gui::window::WasabiError,
    settings::{WasabiSettings, WasabiSoundfont},
    state::WasabiState,
};

use super::show_sf_config;

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SFListItem {
    pub item: WasabiSoundfont,
    pub id: usize,
    pub selected: bool,
    pub config_visible: bool,
}

pub struct EguiSFList {
    list: Vec<SFListItem>,
    id_count: usize,

    sf_picker: (Sender<PathBuf>, Receiver<PathBuf>),
}

impl EguiSFList {
    pub fn new() -> Self {
        let sf_picker = crossbeam_channel::unbounded();

        Self {
            list: Vec::new(),
            id_count: 0,
            sf_picker,
        }
    }

    pub fn add_item(&mut self, sf: WasabiSoundfont, show_cfg: bool) {
        let item = SFListItem {
            item: sf,
            id: self.id_count,
            selected: false,
            config_visible: show_cfg,
        };
        self.list.push(item);
        self.id_count += 1;
    }

    fn add_path(&mut self, path: PathBuf) -> Result<(), WasabiError> {
        if !path.exists() {
            return Err(WasabiError::FilesystemError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{:?} not found.", &path),
            )));
        }

        let item = WasabiSoundfont {
            path,
            enabled: true,
            options: Default::default(),
        };

        self.add_item(item, true);

        Ok(())
    }

    fn select_all(&mut self) {
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

    fn remove_selected_items(&mut self) {
        self.list = self
            .list
            .clone()
            .into_iter()
            .filter(|item| !item.selected)
            .collect();
    }

    fn move_selected_down(&mut self) {
        let cloned = self.list.clone();
        for (i, item) in cloned.iter().enumerate() {
            if i != self.list.len() - 1 && item.selected {
                self.list.swap(i, i + 1);
            }
        }
    }

    fn move_selected_up(&mut self) {
        let cloned = self.list.clone();
        for (i, item) in cloned.iter().enumerate() {
            if i != 0 && item.selected {
                self.list.swap(i, i - 1);
            }
        }
    }

    fn clear(&mut self) {
        self.list.clear();
    }

    fn as_vec(&self) -> Vec<WasabiSoundfont> {
        self.list.iter().map(|sf| sf.item.clone()).collect()
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        state: &mut WasabiState,
    ) {
        // Set keyboard shortcuts
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

        // Check for paths sent by the file picker
        {
            let recv = self.sf_picker.1.clone();
            if !recv.is_empty() {
                if let Some(path) = recv.into_iter().next() {
                    state.last_sf_location = path.clone();
                    if path.is_file() {
                        if let Err(err) = self.add_path(path.clone()) {
                            state
                                .errors
                                .warning(format!("Error adding SoundFont to the list: {}", err));
                        }
                    }
                }
            }
        }

        // Show config windows
        for sf in self.list.iter_mut() {
            if sf.config_visible {
                show_sf_config(ui.ctx(), sf);
            }
        }

        // Render action buttons
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.add_space(5.0);
                ui.columns(2, |columns| {
                    columns[0].horizontal(|ui| {
                        if ui
                            .button(
                                WidgetText::from(" \u{2795} ")
                                    .text_style(egui::TextStyle::Name("monospace big".into())),
                            )
                            .on_hover_text("Add SoundFont(s)")
                            .clicked()
                        {
                            let sender = self.sf_picker.0.clone();
                            let last_sf_location = state.last_sf_location.clone();

                            thread::spawn(move || {
                                let midi_path = rfd::FileDialog::new()
                                    .add_filter(
                                        "Supported SoundFonts",
                                        &["sfz", "SFZ", "sf2", "SF2"],
                                    )
                                    .set_title("Pick SoundFonts...")
                                    .set_directory(
                                        last_sf_location.parent().unwrap_or(Path::new("./")),
                                    )
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
                                WidgetText::from(" \u{2191} ")
                                    .text_style(egui::TextStyle::Name("monospace big".into())),
                            )
                            .on_hover_text("Move Selected Up")
                            .clicked()
                        {
                            self.move_selected_up();
                        }
                        if ui
                            .button(
                                WidgetText::from(" \u{2193} ")
                                    .text_style(egui::TextStyle::Name("monospace big".into())),
                            )
                            .on_hover_text("Move Selected Down")
                            .clicked()
                        {
                            self.move_selected_down();
                        }
                    });

                    columns[1].with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        if ui
                            .button(
                                WidgetText::from(" \u{2705} ")
                                    .text_style(egui::TextStyle::Name("monospace big".into())),
                            )
                            .on_hover_text("Apply SoundFont List")
                            .clicked()
                        {
                            state.synth.set_soundfonts(
                                &settings.synth.soundfonts,
                                state.loading_status.clone(),
                                state.errors.clone(),
                            );
                        }
                    });
                });
                ui.vertical_centered(|ui| {
                    ui.small(
                        "Loading order is bottom to top. \
                        Double click on a soundfont to modify its options. \
                        Supported formats: SFZ, SF2",
                    );
                });
            });

        // Render the list
        egui::ScrollArea::both().show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .cell_layout(egui::Layout::centered_and_justified(
                    egui::Direction::LeftToRight,
                ))
                .resizable(true)
                .column(Column::exact(20.0).resizable(false))
                .column(Column::remainder().at_least(50.0).clip(true))
                .columns(Column::auto().at_least(40.0).clip(true).resizable(false), 2)
                .header(20.0, |mut header| {
                    header.col(|_ui| {});
                    header.col(|ui| {
                        ui.strong("Filename");
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
                                if selectable.double_clicked() {
                                    item.config_visible = true;
                                }
                            });

                            let bank_txt = if let Some(bank) = item.item.options.bank {
                                format!("{}", bank)
                            } else {
                                "-".to_owned()
                            };
                            row.col(|ui| {
                                ui.label(bank_txt.to_string());
                            });

                            let preset_txt = if let Some(preset) = item.item.options.preset {
                                format!("{}", preset)
                            } else {
                                "-".to_owned()
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
