use std::time::Duration;

use egui::{popup_below_widget, PopupCloseBehavior};

use super::{GuiWasabiWindow, SPACE, WIN_MARGIN};
use crate::{midi::MIDIFileBase, state::WasabiState, utils::convert_seconds_to_time_string};

impl GuiWasabiWindow {
    pub fn show_playback_panel(&mut self, ctx: &egui::Context, state: &mut WasabiState) {
        let mut mouse_over_panel = false;
        if let Some(mouse) = ctx.pointer_latest_pos() {
            if mouse.y < 60.0 {
                mouse_over_panel = true;
            }
        }
        let button_size = egui::Vec2::new(26.0, 26.0);
        let icon_color = ctx.style().visuals.strong_text_color();
        let button_rounding = 8.0;

        let is_popup_open = ctx.memory(|mem| mem.is_popup_open(state.panel_popup_id));

        let frame = egui::Frame::side_top_panel(&ctx.style()).inner_margin(WIN_MARGIN);
        egui::TopBottomPanel::top("panel")
            .frame(frame)
            .show_separator_line(false)
            .show_animated(
                ctx,
                state.panel_pinned || mouse_over_panel || is_popup_open,
                |ui| {
                    ui.horizontal(|ui| {
                        let folder_img =
                            egui::Image::new(egui::include_image!("../../../assets/folder.svg"))
                                .fit_to_exact_size(button_size)
                                .tint(icon_color)
                                .rounding(button_rounding);

                        if ui
                            .add(egui::ImageButton::new(folder_img))
                            .on_hover_text("Open MIDI")
                            .clicked()
                        {
                            self.open_midi_dialog(state);
                        }

                        let stop_img =
                            egui::Image::new(egui::include_image!("../../../assets/stop.svg"))
                                .fit_to_exact_size(button_size)
                                .tint(icon_color)
                                .rounding(button_rounding);

                        if ui
                            .add(egui::ImageButton::new(stop_img))
                            .on_hover_text("Unload")
                            .clicked()
                        {
                            if let Some(midi) = self.midi_file.take().as_mut() {
                                midi.timer_mut().pause();
                                self.synth.write().unwrap().reset();
                            }
                        }

                        let playing = if let Some(midi) = self.midi_file.as_ref() {
                            !midi.timer().is_paused()
                        } else {
                            false
                        };

                        // FIXME
                        if playing {
                            let pause_img =
                                egui::Image::new(egui::include_image!("../../../assets/pause.svg"))
                                    .fit_to_exact_size(button_size)
                                    .tint(icon_color)
                                    .rounding(button_rounding);

                            if ui
                                .add(egui::ImageButton::new(pause_img))
                                .on_hover_text("Pause")
                                .clicked()
                            {
                                if let Some(midi_file) = self.midi_file.as_mut() {
                                    midi_file.timer_mut().pause();
                                }
                            }
                        } else {
                            let play_img =
                                egui::Image::new(egui::include_image!("../../../assets/play.svg"))
                                    .fit_to_exact_size(button_size)
                                    .tint(icon_color)
                                    .rounding(button_rounding);

                            if ui
                                .add(egui::ImageButton::new(play_img))
                                .on_hover_text("Play")
                                .clicked()
                            {
                                if let Some(midi_file) = self.midi_file.as_mut() {
                                    midi_file.timer_mut().play();
                                }
                            }
                        }

                        ui.add_space(SPACE);
                        ui.separator();
                        ui.add_space(SPACE);

                        let mut time_passed = 0.0;
                        let mut time_total = 0.0;

                        if let Some(midi) = self.midi_file.as_ref() {
                            time_total = midi.midi_length().unwrap_or(0.0);
                            time_passed = midi.timer().get_time().as_secs_f64();
                        }

                        let mut timeid = ui
                            .style()
                            .text_styles
                            .get(&egui::TextStyle::Monospace)
                            .unwrap()
                            .clone();
                        timeid.size = 16.0;
                        let time_text = convert_seconds_to_time_string(time_passed);
                        let time_galley = ui.painter().layout_no_wrap(
                            time_text.clone(),
                            timeid.clone(),
                            egui::Color32::WHITE,
                        );

                        let length_text = convert_seconds_to_time_string(time_total);
                        let length_galley = ui.painter().layout_no_wrap(
                            length_text.clone(),
                            timeid.clone(),
                            egui::Color32::WHITE,
                        );

                        ui.spacing_mut().slider_width = ui.available_width()
                            - time_galley.size().x
                            - length_galley.size().x
                            - ui.spacing().item_spacing.x * 8.0
                            - button_size.x * 2.0
                            - ui.spacing().button_padding.x * 2.0
                            - SPACE;

                        ui.label(egui::RichText::new(time_text).font(timeid.clone()));
                        let mut empty_slider =
                            || ui.add(egui::Slider::new(&mut 0.0, 0.0..=1.0).show_value(false));
                        if let Some(midi_file) = self.midi_file.as_mut() {
                            if let Some(length) = midi_file.midi_length() {
                                let mut time = midi_file.timer().get_time().as_secs_f64();
                                let time_prev = time;

                                ui.add(
                                    egui::Slider::new(&mut time, 0.0..=length).show_value(false),
                                );
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
                        ui.label(egui::RichText::new(length_text).font(timeid.clone()));

                        ui.add_space(SPACE);
                        ui.separator();
                        ui.add_space(SPACE);

                        let options_img =
                            egui::Image::new(egui::include_image!("../../../assets/options.svg"))
                                .fit_to_exact_size(button_size)
                                .tint(icon_color)
                                .rounding(button_rounding);

                        let options = ui.add(egui::ImageButton::new(options_img));

                        if options.clicked() {
                            ui.memory_mut(|mem| mem.toggle_popup(state.panel_popup_id));
                        }
                        popup_below_widget(
                            ui,
                            state.panel_popup_id,
                            &options,
                            PopupCloseBehavior::CloseOnClick,
                            |ui| {
                                ui.set_min_width(130.0);

                                if ui.button("Settings").clicked() {
                                    state.show_settings = true;
                                }
                                if ui.button("Shortcuts").clicked() {
                                    state.show_shortcuts = true;
                                }
                                if ui.button("About").clicked() {
                                    state.show_about = true;
                                }
                            },
                        );

                        let arrow_img =
                            egui::Image::new(egui::include_image!("../../../assets/pin.svg"))
                                .fit_to_exact_size(button_size)
                                .tint(icon_color)
                                .rounding(button_rounding);

                        if ui
                            .add(egui::ImageButton::new(arrow_img).selected(state.panel_pinned))
                            .on_hover_text("Pin Panel")
                            .clicked()
                        {
                            state.panel_pinned = !state.panel_pinned;
                        }
                    });
                },
            );
    }
}
