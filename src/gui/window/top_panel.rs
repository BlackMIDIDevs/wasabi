use egui::{Context, Frame};

use std::time::Duration;

use crate::{
    gui::window::GuiWasabiWindow, midi::MIDIFileBase, settings::WasabiSettings, state::WasabiState,
};

pub fn draw_panel(
    win: &mut GuiWasabiWindow,
    settings: &mut WasabiSettings,
    state: &mut WasabiState,
    ctx: &Context,
) {
    let panel_frame = Frame::default()
        .inner_margin(egui::style::Margin::same(10.0))
        .fill(egui::Color32::from_rgb(42, 42, 42));

    egui::TopBottomPanel::top("Top panel")
        .frame(panel_frame)
        .show_separator_line(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    win.open_midi_dialog(state);
                }

                if let Some(midi_file) = win.midi_file.as_mut() {
                    if ui.button("Unload").clicked() {
                        midi_file.timer_mut().pause();
                        win.synth.write().unwrap().reset();
                        win.midi_file = None;
                    }
                }

                ui.add_space(10.0);

                if ui.button("Settings").clicked() {
                    match state.settings_visible {
                        true => state.settings_visible = false,
                        false => state.settings_visible = true,
                    }
                }

                ui.add_space(10.0);

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

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Note speed: ");
                    ui.add(
                        egui::Slider::new(&mut settings.note_speed, 2.0..=0.001).show_value(false),
                    );
                })
            });

            ui.spacing_mut().slider_width = ctx.available_rect().width() - 20.0;
            let mut empty_slider =
                || ui.add(egui::Slider::new(&mut 0.0, 0.0..=1.0).show_value(false));
            if let Some(midi_file) = win.midi_file.as_mut() {
                if let Some(length) = midi_file.midi_length() {
                    let mut time = midi_file.timer().get_time().as_secs_f64();
                    let time_prev = time;

                    ui.add(egui::Slider::new(&mut time, 0.0..=length).show_value(false));
                    if (time_prev != time)
                        && (midi_file.allows_seeking_backward() || time_prev < time)
                    {
                        midi_file.timer_mut().seek(Duration::from_secs_f64(time));
                    }
                } else {
                    empty_slider();
                }
            } else {
                empty_slider();
            }
        });
}
