use egui::{Frame, Context};

use std::time::Duration;

use crate::{
    gui::window::GuiWasabiWindow,
    midi::{InRamMIDIFile, LiveLoadMIDIFile, MIDIFileBase, MIDIFileUnion},
    settings::{WasabiPermanentSettings, WasabiTemporarySettings},
};

use rfd::FileDialog;

pub fn draw_panel(
    win: &mut GuiWasabiWindow,
    perm_settings: &mut WasabiPermanentSettings,
    temp_settings: &mut WasabiTemporarySettings,
    ctx: &Context,
    height: f32,
) {
    let panel_frame = Frame::default()
        .inner_margin(egui::style::Margin::same(10.0))
        .fill(egui::Color32::from_rgb(42, 42, 42));

    egui::TopBottomPanel::top("Top panel")
        .height_range(0.0..=height)
        .frame(panel_frame)
        .show(&ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open MIDI").clicked() {
                    let midi_path = FileDialog::new()
                        .add_filter("midi", &["mid"])
                        .set_directory("/")
                        .pick_file();

                    if let Some(midi_path) = midi_path {
                        if let Some(midi_file) = win.midi_file.as_mut() {
                            midi_file.timer_mut().pause();
                        }
                        win.reset_synth();
                        win.midi_file = None;

                        if let Ok(path) = midi_path.into_os_string().into_string() {
                            match perm_settings.midi_loading {
                                0 => {
                                    let mut midi_file =
                                        MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
                                            &path,
                                            win.synth.clone(),
                                            perm_settings.random_colors,
                                        ));
                                    midi_file.timer_mut().play();
                                    win.midi_file = Some(midi_file);
                                }
                                1 => {
                                    let mut midi_file =
                                        MIDIFileUnion::Live(LiveLoadMIDIFile::load_from_file(
                                            &path,
                                            win.synth.clone(),
                                            perm_settings.random_colors,
                                        ));
                                    midi_file.timer_mut().play();
                                    win.midi_file = Some(midi_file);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if let Some(midi_file) = win.midi_file.as_mut() {
                    if ui.button("Close MIDI").clicked() {
                        midi_file.timer_mut().pause();
                        win.reset_synth();
                        win.midi_file = None;
                    }
                }
                if ui.button("Play").clicked() {
                    if let Some(midi_file) = win.midi_file.as_mut() {
                        midi_file.timer_mut().play();
                    }
                }
                if ui.button("Pause").clicked() {
                    if let Some(midi_file) = win.midi_file.as_mut() {
                        midi_file.timer_mut().pause();
                    }
                }
                if ui.button("Settings").clicked() {
                    match temp_settings.settings_visible {
                        true => temp_settings.settings_visible = false,
                        false => temp_settings.settings_visible = true,
                    }
                }
                ui.horizontal(|ui| {
                    ui.label("Note speed: ");
                    ui.add(
                        egui::Slider::new(&mut perm_settings.note_speed, 2.0..=0.001)
                            .show_value(false),
                    );
                })
            });

            if let Some(midi_file) = win.midi_file.as_mut() {
                if let Some(length) = midi_file.midi_length() {
                    let time = midi_file.timer().get_time().as_secs_f64();
                    let mut progress = time / length;
                    let progress_prev = progress;
                    let slider = egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false);
                    ui.spacing_mut().slider_width = ctx.available_rect().width() - 20.0;
                    ui.add(slider);
                    if (progress_prev != progress)
                        && (midi_file.allows_seeking_backward() || progress_prev < progress)
                    {
                        let position = Duration::from_secs_f64(progress * length);
                        midi_file.timer_mut().seek(position);
                    }
                }
            } else {
                let mut progress = 0.0;
                let slider = egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false);
                ui.spacing_mut().slider_width = ctx.available_rect().width() - 20.0;
                ui.add(slider);
            }
        });
}
