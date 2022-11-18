use egui::Context;

use std::{
    ops::RangeInclusive,
    sync::{Arc, RwLock},
};

use crate::{
    audio_playback::{AudioPlayerType, SimpleTemporaryPlayer},
    gui::window::GuiWasabiWindow,
    settings::{WasabiPermanentSettings, WasabiTemporarySettings},
};

use rfd::FileDialog;

pub fn draw_settings(
    win: &mut GuiWasabiWindow,
    perm_settings: &mut WasabiPermanentSettings,
    temp_settings: &mut WasabiTemporarySettings,
    ctx: &Context,
) {
    egui::Window::new("Settings")
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .scroll2([false, true])
        .enabled(true)
        .open(&mut temp_settings.settings_visible)
        .show(ctx, |ui| {
            // Synth settings section
            ui.heading("Synth");
            ui.separator();

            let sfz_path_prev = perm_settings.sfz_path.clone();
            egui::Grid::new("synth_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Synth*: ");
                    let synth_prev = perm_settings.synth;
                    let synth = ["XSynth", "KDMAPI"];
                    egui::ComboBox::from_id_source("synth_select").show_index(
                        ui,
                        &mut perm_settings.synth,
                        synth.len(),
                        |i| synth[i].to_owned(),
                    );
                    if perm_settings.synth != synth_prev {
                        match perm_settings.synth {
                            1 => {
                                win.synth = Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                                    AudioPlayerType::Kdmapi,
                                )));
                            }
                            _ => {
                                win.synth = Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                                    AudioPlayerType::XSynth(perm_settings.buffer_ms),
                                )));
                                win.synth
                                    .write()
                                    .unwrap()
                                    .set_soundfont(&perm_settings.sfz_path);
                                win.synth.write().unwrap().set_layer_count(
                                    match perm_settings.layer_count {
                                        0 => None,
                                        _ => Some(perm_settings.layer_count),
                                    },
                                );
                            }
                        }
                    }
                    ui.end_row();

                    if perm_settings.synth == 0 {
                        ui.label("SFZ Path: ");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut perm_settings.sfz_path));
                            if ui.button("Browse...").clicked() {
                                let sfz_path = FileDialog::new()
                                    .add_filter("sfz", &["sfz"])
                                    .set_directory("/")
                                    .pick_file();

                                if let Some(sfz_path) = sfz_path {
                                    if let Ok(path) = sfz_path.into_os_string().into_string() {
                                        perm_settings.sfz_path = path;
                                    }
                                }
                            }
                        });
                        ui.end_row();

                        ui.label("Synth Layer Count: ");
                        let layer_count_prev = perm_settings.layer_count;
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut perm_settings.layer_count)
                                    .speed(1)
                                    .clamp_range(RangeInclusive::new(0, 1024)),
                            );
                            ui.label("(0 = Infinite)");
                        });
                        if perm_settings.layer_count != layer_count_prev {
                            win.synth.write().unwrap().set_layer_count(
                                match perm_settings.layer_count {
                                    0 => None,
                                    _ => Some(perm_settings.layer_count),
                                },
                            );
                        }
                        ui.end_row();

                        ui.label("Synth Render Buffer (ms)*: ");
                        ui.add(
                            egui::DragValue::new(&mut perm_settings.buffer_ms)
                                .speed(0.1)
                                .clamp_range(RangeInclusive::new(1.0, 1000.0)),
                        );
                        ui.end_row();
                    }
                });

            // MIDI settings section
            ui.add_space(5.0);
            ui.heading("MIDI");
            ui.separator();

            egui::Grid::new("midi_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Note speed: ");
                    ui.spacing_mut().slider_width = 150.0;
                    ui.add(egui::Slider::new(
                        &mut perm_settings.note_speed,
                        2.0..=0.001,
                    ));
                    ui.end_row();

                    ui.label("Random Track Colors*: ");
                    ui.checkbox(&mut perm_settings.random_colors, "");
                    ui.end_row();

                    ui.label("Keyboard Range: ");
                    let mut firstkey = *perm_settings.key_range.start();
                    let mut lastkey = *perm_settings.key_range.end();
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut firstkey)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(0, 253)),
                        );
                        ui.add(
                            egui::DragValue::new(&mut lastkey)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(firstkey + 1, 254)),
                        );
                    });
                    ui.end_row();
                    if firstkey != *perm_settings.key_range.start()
                        || lastkey != *perm_settings.key_range.end()
                    {
                        perm_settings.key_range = firstkey..=lastkey;
                    }

                    ui.label("MIDI Loading*: ");
                    let midi_loading = ["In RAM", "Live"];
                    egui::ComboBox::from_id_source("midiload_select").show_index(
                        ui,
                        &mut perm_settings.midi_loading,
                        midi_loading.len(),
                        |i| midi_loading[i].to_owned(),
                    );
                });

            // Visual settings section
            ui.add_space(5.0);
            ui.heading("Visual");
            ui.separator();

            egui::Grid::new("visual_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Background Color: ");
                    ui.color_edit_button_srgba(&mut perm_settings.bg_color);
                    ui.end_row();

                    ui.label("Bar Color: ");
                    ui.color_edit_button_srgba(&mut perm_settings.bar_color);
                    ui.end_row();
                });

            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Options marked with (*) require a restart.");
                if ui.button("Save").clicked() {
                    perm_settings.save_to_file();
                    if sfz_path_prev != perm_settings.sfz_path && perm_settings.synth == 0 {
                        win.synth
                            .write()
                            .unwrap()
                            .set_soundfont(&perm_settings.sfz_path);
                    }
                }
            });
        });
}
