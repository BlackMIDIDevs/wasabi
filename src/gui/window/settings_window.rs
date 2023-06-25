use egui::Context;

use std::ops::RangeInclusive;

use crate::{
    audio_playback::{
        xsynth::{convert_to_channel_init, convert_to_sf_init},
        AudioPlayerType,
    },
    gui::window::GuiWasabiWindow,
    settings::{MidiLoading, Synth, WasabiSettings},
    state::WasabiState,
};

pub fn draw_settings(
    win: &mut GuiWasabiWindow,
    settings: &mut WasabiSettings,
    state: &mut WasabiState,
    ctx: &Context,
) {
    egui::Window::new("Settings")
        .resizable(true)
        .collapsible(true)
        .title_bar(true)
        .scroll2([false, true])
        .enabled(true)
        .open(&mut state.settings_visible)
        .show(ctx, |ui| {
            let col_width = 160.0;

            // Synth settings section
            ui.heading("Synth");
            ui.separator();

            egui::Grid::new("synth_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .min_col_width(col_width)
                .show(ui, |ui| {
                    ui.label("Synth: ");
                    let synth_prev = settings.synth.synth;
                    egui::ComboBox::from_id_source("synth_select")
                        .selected_text(settings.synth.synth.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut settings.synth.synth, Synth::XSynth, "XSynth");
                            ui.selectable_value(&mut settings.synth.synth, Synth::Kdmapi, "KDMAPI");
                        });
                    if settings.synth.synth != synth_prev {
                        match settings.synth.synth {
                            Synth::Kdmapi => {
                                win.synth
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::Kdmapi);
                            }
                            Synth::XSynth => {
                                win.synth
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::XSynth {
                                        buffer: settings.synth.buffer_ms,
                                        ignore_range: settings.synth.vel_ignore.clone(),
                                        options: convert_to_channel_init(settings),
                                    });
                                win.synth.write().unwrap().set_soundfont(
                                    &settings.synth.sfz_path,
                                    convert_to_sf_init(settings),
                                );
                                win.synth.write().unwrap().set_layer_count(
                                    match settings.synth.layer_count {
                                        0 => None,
                                        _ => Some(settings.synth.layer_count),
                                    },
                                );
                            }
                        }
                    }
                    ui.end_row();

                    ui.label("Configure:");
                    if ui.button("Open Synth Settings").clicked() {
                        state.xsynth_settings_visible = true;
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
                .min_col_width(col_width)
                .show(ui, |ui| {
                    ui.label("Note speed: ");
                    ui.spacing_mut().slider_width = 150.0;
                    ui.add(egui::Slider::new(
                        &mut settings.midi.note_speed,
                        2.0..=0.001,
                    ));
                    ui.end_row();

                    ui.label("Random Track Colors*: ");
                    ui.checkbox(&mut settings.midi.random_colors, "");
                    ui.end_row();

                    ui.label("Keyboard Range: ");
                    let mut firstkey = *settings.midi.key_range.start();
                    let mut lastkey = *settings.midi.key_range.end();
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
                    if firstkey != *settings.midi.key_range.start()
                        || lastkey != *settings.midi.key_range.end()
                    {
                        settings.midi.key_range = firstkey..=lastkey;
                    }

                    ui.label("MIDI Loading*: ");
                    egui::ComboBox::from_id_source("midiload_select")
                        .selected_text(settings.midi.midi_loading.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut settings.midi.midi_loading,
                                MidiLoading::Ram,
                                "In RAM",
                            );
                            ui.selectable_value(
                                &mut settings.midi.midi_loading,
                                MidiLoading::Live,
                                "Live",
                            );
                            ui.selectable_value(
                                &mut settings.midi.midi_loading,
                                MidiLoading::Cake,
                                "Cake",
                            );
                        });
                });

            // Visual settings section
            ui.add_space(6.0);
            ui.heading("Visual");
            ui.separator();

            egui::Grid::new("visual_settings_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .min_col_width(col_width)
                .show(ui, |ui| {
                    if ui.button("Toggle Fullscreen").clicked() {
                        state.fullscreen = true;
                    }
                    ui.end_row();

                    ui.label("Background Color: ");
                    ui.color_edit_button_srgba(&mut settings.visual.bg_color);
                    ui.end_row();

                    ui.label("Bar Color: ");
                    ui.color_edit_button_srgba(&mut settings.visual.bar_color);
                    ui.end_row();
                });

            ui.separator();
            ui.vertical_centered(|ui| {
                ui.label("Options marked with (*) will apply when a new MIDI is loaded.");
                if ui.button("Save").clicked() {
                    settings.save_to_file();
                }
            });
        });
}
