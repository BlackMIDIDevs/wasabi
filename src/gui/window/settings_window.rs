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

include!("../../help/help_settings.rs");

macro_rules! with_tooltip {
    { $ui:expr;$short_help:expr,$long_help:expr,$shift:expr } => {{
        let element = $ui;
        if $shift {
            element.on_hover_text_at_pointer(concat!($long_help, "\n\nHold H for more info"));
        } else {
            element.on_hover_text_at_pointer(concat!($short_help, "\n\nHold H for more info"));
        }
    }};
}

pub fn draw_settings(
    win: &mut GuiWasabiWindow,
    settings: &mut WasabiSettings,
    state: &mut WasabiState,
    ctx: &Context,
) {
    let is_shift = ctx.input().key_down(egui::Key::H);
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
                    let synth = ["XSynth", "KDMAPI"];
                    with_tooltip! {
                        egui::ComboBox::from_id_source("synth_select").show_index(
                            ui,
                            unsafe {
                                std::mem::transmute::<&mut Synth, &mut usize>(&mut settings.synth.synth)
                            },
                            synth.len(),
                            |i| synth[i].to_owned(),
                        );
                        synth_short_help!(), synth_long_help!(), is_shift
                    }
                    if settings.synth.synth != synth_prev {
                        match settings.synth.synth {
                            Synth::Kdmapi => {
                                win.synth
                                    .player
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::Kdmapi);
                            }
                            Synth::XSynth => {
                                win.synth.player.write().unwrap().switch_player(
                                    AudioPlayerType::XSynth {
                                        buffer: settings.synth.buffer_ms,
                                        ignore_range: settings.synth.vel_ignore.clone(),
                                        options: convert_to_channel_init(settings),
                                    },
                                );
                                win.synth.player.write().unwrap().set_soundfont(
                                    &settings.synth.sfz_path,
                                    convert_to_sf_init(settings),
                                );
                                win.synth.player.write().unwrap().set_layer_count(
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
                    with_tooltip! {
                        ui.add(egui::Slider::new(
                            &mut settings.midi.note_speed,
                            2.0..=0.001,
                        ));
                        note_speed_short_help!(), note_speed_long_help!(), is_shift
                    }
                    ui.end_row();

                    ui.label("Random Track Colors*: ");
                    with_tooltip! {
                        ui.checkbox(&mut settings.midi.random_colors, "");
                        random_colors_short_help!(), random_colors_long_help!(), is_shift
                    }
                    ui.end_row();

                    ui.label("Keyboard Range: ");
                    let mut firstkey = *settings.midi.key_range.start();
                    let mut lastkey = *settings.midi.key_range.end();
                    with_tooltip! {
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
                        }).response;
                        key_range_short_help!(), key_range_long_help!(), is_shift
                    }
                    ui.end_row();
                    if firstkey != *settings.midi.key_range.start()
                        || lastkey != *settings.midi.key_range.end()
                    {
                        settings.midi.key_range = firstkey..=lastkey;
                    }

                    ui.label("MIDI Loading*: ");
                    let midi_loading = ["In RAM", "Live"];
                    with_tooltip! {
                        egui::ComboBox::from_id_source("midiload_select").show_index(
                            ui,
                            unsafe {
                                std::mem::transmute::<&mut MidiLoading, &mut usize>(
                                    &mut settings.midi.midi_loading,
                                )
                            },
                            midi_loading.len(),
                            |i| midi_loading[i].to_owned(),
                        );
                        midi_loading_short_help!(), midi_loading_long_help!(), is_shift
                    }
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
                    with_tooltip! {
                        ui.color_edit_button_srgba(&mut settings.visual.bg_color);
                        bg_color_short_help!(), bg_color_long_help!(), is_shift
                    }
                    ui.end_row();

                    ui.label("Bar Color: ");
                    with_tooltip! {
                        ui.color_edit_button_srgba(&mut settings.visual.bar_color);
                        bar_color_short_help!(), bar_color_long_help!(), is_shift
                    }
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
