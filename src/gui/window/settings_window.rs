use egui::Context;

use std::ops::RangeInclusive;

use crate::{
    audio_playback::{AudioPlayerType, xsynth::{convert_to_sf_init, convert_to_channel_init}},
    gui::window::GuiWasabiWindow,
    settings::{WasabiPermanentSettings, WasabiTemporarySettings},
};

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

            egui::Grid::new("synth_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Synth: ");
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
                                win.synth
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::Kdmapi);
                            }
                            _ => {
                                win.synth
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::XSynth {
                                        buffer: perm_settings.buffer_ms,
                                        ignore_range: perm_settings.vel_ignore.clone(),
                                        options: convert_to_channel_init(perm_settings)
                                    });
                                win.synth
                                    .write()
                                    .unwrap()
                                    .set_soundfont(
                                        &perm_settings.sfz_path,
                                        convert_to_sf_init(perm_settings)
                                    );
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

                    ui.label("Configure:");
                    if ui.button("Open Synth Settings").clicked() {
                        temp_settings.xsynth_settings_visible = true;
                    }
                    ui.end_row();
                });

            // MIDI settings section
            ui.add_space(6.0);
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
            ui.add_space(6.0);
            ui.heading("Visual");
            ui.separator();

            egui::Grid::new("visual_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Fullscreen: ");
                    ui.checkbox(&mut temp_settings.fullscreen, "");
                    ui.end_row();

                    ui.label("Background Color: ");
                    ui.color_edit_button_srgba(&mut perm_settings.bg_color);
                    ui.end_row();

                    ui.label("Bar Color: ");
                    ui.color_edit_button_srgba(&mut perm_settings.bar_color);
                    ui.end_row();
                });

            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Options marked with (*) will apply when a new MIDI is loaded.");
                if ui.button("Save").clicked() {
                    perm_settings.save_to_file();
                }
            });
        });
}
