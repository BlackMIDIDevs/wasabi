use egui_extras::{Column, TableBuilder};

use crate::{
    settings::{Colors, MidiParsing, WasabiSettings},
    state::WasabiState,
};

use super::SettingsWindow;

impl SettingsWindow {
    pub fn show_midi_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        state: &WasabiState,
        width: f32,
    ) {
        ui.vertical_centered(|ui| {
            ui.small("Changes to the settings below will be applied when a new MIDI is loaded.");
        });

        ui.heading("Settings");
        egui::Grid::new("midi_settings_grid")
            .num_columns(2)
            .spacing(super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("MIDI Parsing Algorithm:");
                    ui.monospace("\u{2139}").on_hover_text(
                        "\
                    - Cake\n\
                  \0    The most efficient loading and displaying algorithm.\n\
                  \0    The notes will be stored in binary trees and will be\n\
                  \0    displayed dynamically. This mode does not support\n\
                  \0    polyphony statistics.\n\
                    - Standard (RAM)\n\
                  \0    The MIDI will be loaded in the RAM and all the notes\n\
                  \0    will be rendered normally by the GPU.\n\
                    - Standard (Live)\n\
                  \0    The MIDI will be streamed live from the disk and all\n\
                  \0    the notes will be rendered normally by the GPU.\
                    ",
                    );
                });
                egui::ComboBox::from_id_salt("midi_parsing_select")
                    .selected_text(settings.midi.parsing.as_str())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut settings.midi.parsing,
                            MidiParsing::Cake,
                            MidiParsing::Cake.as_str(),
                        );
                        ui.selectable_value(
                            &mut settings.midi.parsing,
                            MidiParsing::Ram,
                            MidiParsing::Ram.as_str(),
                        );
                        ui.selectable_value(
                            &mut settings.midi.parsing,
                            MidiParsing::Live,
                            MidiParsing::Live.as_str(),
                        );
                    });
                ui.end_row();

                ui.label("Start Delay (s):");
                ui.add(
                    egui::DragValue::new(&mut settings.midi.start_delay)
                        .speed(1.0)
                        .range(0.0..=100.0),
                );
                ui.end_row();
            });

        ui.horizontal(|ui| ui.add_space(width + 40.0));
        ui.add_space(super::CATEG_SPACE);
        ui.horizontal(|ui| {
            ui.heading("Color Palette");
            ui.monospace("\u{2139}").on_hover_text(
                "\
                Each palette should be an image (PNG, JPEG, etc.)\n\
                with a 16 pixel width (one column for each MIDI\n\
                channel) and any height (one row for each track).\n\
                If the MIDI has more tracks than what is available\n\
                in the palette, the colours will loop.\
                ",
            );
        });
        egui::Frame::default()
            .rounding(egui::Rounding::same(8.0))
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    ))
                    .resizable(true)
                    .column(Column::exact(width).resizable(false))
                    .body(|mut body| {
                        let row_height = super::SPACING[1] * 3.0;
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                if ui
                                    .selectable_label(
                                        settings.midi.colors == Colors::Rainbow,
                                        Colors::Rainbow.as_str(),
                                    )
                                    .clicked()
                                {
                                    settings.midi.colors = Colors::Rainbow;
                                }
                            });
                        });
                        body.row(row_height, |mut row| {
                            row.col(|ui| {
                                if ui
                                    .selectable_label(
                                        settings.midi.colors == Colors::Random,
                                        Colors::Random.as_str(),
                                    )
                                    .clicked()
                                {
                                    settings.midi.colors = Colors::Random;
                                }
                            });
                        });
                        let mut temp = self.palettes.clone();
                        for i in temp.iter_mut() {
                            i.selected = false;
                        }
                        let mut changed = false;
                        for (i, palette) in self.palettes.iter_mut().enumerate() {
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    if ui
                                        .selectable_label(
                                            settings.midi.colors == Colors::Palette
                                                && palette.selected,
                                            palette
                                                .path
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_str()
                                                .unwrap_or_default(),
                                        )
                                        .clicked()
                                    {
                                        settings.midi.colors = Colors::Palette;
                                        temp[i].selected = true;
                                        settings.midi.palette_path = palette.path.clone();
                                        changed = true;
                                    }
                                });
                            });
                        }
                        if changed {
                            self.palettes = temp;
                        }
                    });
            });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button("Refresh List").clicked() {
                self.load_palettes(settings)
                    .unwrap_or_else(|e| state.errors.error(&e));
            }
            if ui.button("Open Palettes Directory").clicked() {
                open::that(WasabiSettings::get_palettes_dir()).unwrap_or_default();
            }
            ui.separator();
            ui.checkbox(&mut settings.midi.randomize_palette, " Randomize Palette")
                .on_hover_text("Does not affect \"Rainbow\" and \"Random\" palettes.");
        });
    }
}
