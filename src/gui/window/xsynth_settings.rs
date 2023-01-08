use egui::Context;

use std::ops::RangeInclusive;

use crate::{
    audio_playback::{
        xsynth::{convert_to_channel_init, convert_to_sf_init},
        AudioPlayerType,
    },
    gui::window::GuiWasabiWindow,
    settings::{WasabiPermanentSettings, WasabiTemporarySettings},
};

use rfd::FileDialog;

pub fn draw_xsynth_settings(
    win: &mut GuiWasabiWindow,
    perm_settings: &mut WasabiPermanentSettings,
    temp_settings: &mut WasabiTemporarySettings,
    ctx: &Context,
) {
    egui::Window::new("XSynth Settings")
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .scroll2([false, true])
        .enabled(true)
        .open(&mut temp_settings.xsynth_settings_visible)
        .show(ctx, |ui| {
            ui.heading("Synth");
            ui.separator();

            egui::Grid::new("synth_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Synth Render Buffer (ms)*: ");
                    ui.add(
                        egui::DragValue::new(&mut perm_settings.buffer_ms)
                            .speed(0.1)
                            .clamp_range(RangeInclusive::new(0.001, 1000.0)),
                    );
                    ui.end_row();

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
                        if ui.button("Load").clicked() {
                            win.synth.write().unwrap().set_soundfont(
                                &perm_settings.sfz_path,
                                convert_to_sf_init(perm_settings),
                            );
                        }
                    });
                    ui.end_row();

                    ui.label("Limit Layers: ");
                    ui.checkbox(&mut perm_settings.limit_layers, "");
                    ui.end_row();

                    ui.label("Synth Layer Count: ");
                    ui.add_enabled_ui(perm_settings.limit_layers, |ui| {
                        let layer_count_prev = perm_settings.layer_count;
                        ui.add(
                            egui::DragValue::new(&mut perm_settings.layer_count)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(0, 200)),
                        );
                        if perm_settings.layer_count != layer_count_prev {
                            win.synth.write().unwrap().set_layer_count(
                                if perm_settings.limit_layers {
                                    Some(perm_settings.layer_count)
                                } else {
                                    None
                                },
                            );
                        }
                    });
                    ui.end_row();

                    ui.label("Ignore notes with velocities between*: ");
                    let mut lovel = *perm_settings.vel_ignore.start();
                    let mut hivel = *perm_settings.vel_ignore.end();
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut lovel)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(0, 127)),
                        );
                        ui.label("and");
                        ui.add(
                            egui::DragValue::new(&mut hivel)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(lovel, 127)),
                        );
                    });
                    ui.end_row();
                    if lovel != *perm_settings.vel_ignore.start()
                        || hivel != *perm_settings.vel_ignore.end()
                    {
                        perm_settings.vel_ignore = lovel..=hivel;
                    }
                });

            ui.add_space(6.0);
            ui.heading("Engine");
            ui.separator();

            egui::Grid::new("engine_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Fade out voice when killing it*: ");
                    ui.checkbox(&mut perm_settings.fade_out_kill, "");
                    ui.end_row();

                    ui.label("Linear release envelope*: ");
                    ui.checkbox(&mut perm_settings.linear_envelope, "");
                    ui.end_row();
                });

            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Options marked with (*) will apply when the synth is reloaded.");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        perm_settings.save_to_file();
                    }

                    if ui.button("Reload").clicked() {
                        win.synth
                            .write()
                            .unwrap()
                            .switch_player(AudioPlayerType::XSynth {
                                buffer: perm_settings.buffer_ms,
                                ignore_range: perm_settings.vel_ignore.clone(),
                                options: convert_to_channel_init(perm_settings),
                            });
                        win.synth.write().unwrap().set_soundfont(
                            &perm_settings.sfz_path,
                            convert_to_sf_init(perm_settings),
                        );
                        win.synth.write().unwrap().set_layer_count(
                            match perm_settings.layer_count {
                                0 => None,
                                _ => Some(perm_settings.layer_count),
                            },
                        );
                    }
                });
            });
        });
}
