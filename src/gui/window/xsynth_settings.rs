use egui::Context;

use std::ops::RangeInclusive;

use crate::{
    audio_playback::{
        xsynth::{convert_to_channel_init, convert_to_sf_init},
        AudioPlayerType,
    },
    gui::window::GuiWasabiWindow,
    settings::WasabiSettings,
    state::WasabiState,
};

pub fn draw_xsynth_settings(
    win: &mut GuiWasabiWindow,
    settings: &mut WasabiSettings,
    state: &mut WasabiState,
    ctx: &Context,
) {
    egui::Window::new("XSynth Settings")
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .scroll2([false, true])
        .enabled(true)
        .open(&mut state.xsynth_settings_visible)
        .show(ctx, |ui| {
            let col_width = 240.0;

            ui.heading("Synth");
            ui.separator();

            egui::Grid::new("synth_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .min_col_width(col_width)
                .show(ui, |ui| {
                    ui.label("Synth Render Buffer (ms)*: ");
                    ui.add(
                        egui::DragValue::new(&mut settings.synth.buffer_ms)
                            .speed(0.1)
                            .clamp_range(RangeInclusive::new(0.001, 1000.0)),
                    );
                    ui.end_row();

                    ui.label("SFZ Path: ");
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut settings.synth.sfz_path));

                        if ui.button("Browse...").clicked() {
                            // If windows, just use the native dialog
                            let sfz_path = rfd::FileDialog::new()
                                .add_filter("sfz/sf2", &["sfz", "sf2"])
                                .pick_file();

                            if let Some(sfz_path) = sfz_path {
                                if let Ok(path) = sfz_path.into_os_string().into_string() {
                                    settings.synth.sfz_path = path;
                                }
                            }
                        }

                        if ui.button("Load").clicked() {
                            win.synth.write().unwrap().set_soundfont(
                                &settings.synth.sfz_path,
                                convert_to_sf_init(settings),
                            );
                        }
                    });
                    ui.end_row();

                    ui.label("Limit Layers: ");
                    let layer_limit_prev = settings.synth.limit_layers;
                    ui.checkbox(&mut settings.synth.limit_layers, "");
                    ui.end_row();

                    ui.label("Synth Layer Count: ");
                    let layer_count_prev = settings.synth.layer_count;
                    ui.add_enabled_ui(settings.synth.limit_layers, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut settings.synth.layer_count)
                                .speed(1)
                                .clamp_range(RangeInclusive::new(1, 200)),
                        );
                    });
                    if settings.synth.layer_count != layer_count_prev
                        || layer_limit_prev != settings.synth.limit_layers
                    {
                        win.synth.write().unwrap().set_layer_count(
                            if settings.synth.limit_layers {
                                Some(settings.synth.layer_count)
                            } else {
                                None
                            },
                        );
                    }
                    ui.end_row();

                    ui.label("Ignore notes with velocities between*: ");
                    let mut lovel = *settings.synth.vel_ignore.start();
                    let mut hivel = *settings.synth.vel_ignore.end();
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
                    if lovel != *settings.synth.vel_ignore.start()
                        || hivel != *settings.synth.vel_ignore.end()
                    {
                        settings.synth.vel_ignore = lovel..=hivel;
                    }
                });

            ui.add_space(6.0);
            ui.heading("Engine");
            ui.separator();

            egui::Grid::new("engine_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .min_col_width(col_width)
                .show(ui, |ui| {
                    ui.label("Fade out voice when killing it*: ");
                    ui.checkbox(&mut settings.synth.fade_out_kill, "");
                    ui.end_row();

                    ui.label("Linear release envelope*: ");
                    ui.checkbox(&mut settings.synth.linear_envelope, "");
                    ui.end_row();

                    ui.label("Use Effects*: ");
                    ui.checkbox(&mut settings.synth.use_effects, "");
                    ui.end_row();

                    ui.label("Use Threadpool*: ");
                    ui.checkbox(&mut settings.synth.use_threadpool, "");
                    ui.end_row();
                });

            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Options marked with (*) will apply when the synth is reloaded.");
                if ui.button("Reload XSynth").clicked() {
                    win.synth
                        .write()
                        .unwrap()
                        .switch_player(AudioPlayerType::XSynth {
                            buffer: settings.synth.buffer_ms,
                            use_threadpool: settings.synth.use_threadpool,
                            ignore_range: settings.synth.vel_ignore.clone(),
                            options: convert_to_channel_init(settings),
                        });
                    win.synth
                        .write()
                        .unwrap()
                        .set_soundfont(&settings.synth.sfz_path, convert_to_sf_init(settings));
                    win.synth
                        .write()
                        .unwrap()
                        .set_layer_count(if settings.synth.limit_layers {
                            Some(settings.synth.layer_count)
                        } else {
                            None
                        });
                }
            });
        });
}
