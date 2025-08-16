use std::{collections::VecDeque, time::Instant};

use egui::{Context, Frame, Pos2};

use crate::{
    gui::window::GuiWasabiWindow,
    midi::{MIDIFileBase, MIDIFileStats},
    settings::{Statistics, WasabiSettings},
    utils::convert_seconds_to_time_string,
};

pub struct GuiMidiStats {
    time_passed: f64,
    time_total: f64,
    notes_on_screen: u64,
    polyphony: Option<u64>,
    voice_count: Option<u64>,
}

impl GuiMidiStats {
    pub fn empty() -> GuiMidiStats {
        GuiMidiStats {
            time_passed: 0.0,
            time_total: 0.0,
            notes_on_screen: 0,
            polyphony: None,
            voice_count: None,
        }
    }

    pub fn set_voice_count(&mut self, voices: Option<u64>) {
        self.voice_count = voices;
    }

    pub fn set_rendered_note_count(&mut self, notes: u64) {
        self.notes_on_screen = notes;
    }

    pub fn set_polyphony(&mut self, polyphony: Option<u64>) {
        self.polyphony = polyphony;
    }
}

fn num_or_q(num: Option<impl ToString>) -> String {
    if let Some(num) = num {
        num.to_string()
    } else {
        "-".to_string()
    }
}

impl GuiWasabiWindow {
    pub fn draw_stats(
        &mut self,
        ctx: &Context,
        pos: Pos2,
        mut stats: GuiMidiStats,
        settings: &WasabiSettings,
    ) {
        // Prepare frame based on settings
        let opacity = settings.scene.statistics.opacity.clamp(0.0, 1.0);
        let alpha = (u8::MAX as f32 * opacity).round() as u8;

        let round = 8;

        let mut stats_frame = Frame::default()
            .inner_margin(egui::Margin::same(7))
            .fill(egui::Color32::from_black_alpha(alpha));

        if settings.scene.statistics.floating {
            stats_frame = stats_frame.corner_radius(egui::CornerRadius::same(round));
        } else {
            stats_frame = stats_frame.corner_radius(egui::CornerRadius {
                ne: 0,
                nw: 0,
                sw: 0,
                se: round,
            });
        }

        if settings.scene.statistics.border {
            stats_frame =
                stats_frame.stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 50, 50)));
        }

        // Render statistics in a window
        egui::Window::new("Stats")
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .scroll([false, false])
            .interactable(false)
            .frame(stats_frame)
            .fixed_pos(pos)
            .fixed_size(egui::Vec2::new(200.0, 128.0))
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = 16.0;

                let mut note_stats = MIDIFileStats::default();
                if let Some(midi_file) = self.midi_file.as_mut() {
                    stats.time_total = midi_file.midi_length().unwrap_or(0.0);
                    let time = midi_file.timer().get_time().as_seconds_f64();

                    if time > stats.time_total {
                        stats.time_passed = stats.time_total;
                    } else {
                        stats.time_passed = time;
                    }

                    note_stats = midi_file.stats();
                }

                for i in settings.scene.statistics.order.iter().filter(|i| i.1) {
                    match i.0 {
                        Statistics::Time => {
                            ui.horizontal(|ui| {
                                ui.monospace("Time:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.monospace(format!(
                                            "{} / {}",
                                            convert_seconds_to_time_string(stats.time_passed),
                                            convert_seconds_to_time_string(stats.time_total)
                                        ));
                                    },
                                );
                            });
                        }
                        Statistics::Fps => {
                            ui.horizontal(|ui| {
                                ui.monospace("FPS:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.monospace(format!("{}", self.fps.get_fps()));
                                    },
                                );
                            });
                        }
                        Statistics::VoiceCount => {
                            if let Some(voice_count) = stats.voice_count {
                                ui.horizontal(|ui| {
                                    ui.monospace("Voice Count:");
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.monospace(format!("{}", voice_count));
                                        },
                                    );
                                });
                            }
                        }
                        Statistics::Rendered => {
                            ui.horizontal(|ui| {
                                ui.monospace("Rendered:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.monospace(format!("{}", stats.notes_on_screen));
                                    },
                                );
                            });
                        }
                        Statistics::NoteCount => {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.monospace(format!(
                                    "{} / {}",
                                    num_or_q(note_stats.passed_notes),
                                    num_or_q(note_stats.total_notes)
                                ));
                            });
                        }
                        Statistics::Nps => {
                            ui.horizontal(|ui| {
                                ui.monospace("NPS:");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        self.nps.tick(note_stats.passed_notes.unwrap_or(0) as i64);
                                        ui.monospace(format!("{}", self.nps.read()));
                                    },
                                );
                            });
                        }
                        Statistics::Polyphony => {
                            if let Some(poly) = stats.polyphony {
                                ui.horizontal(|ui| {
                                    ui.monospace("Polyphony:");
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.monospace(format!("{}", poly));
                                        },
                                    );
                                });
                            }
                        }
                    };
                }
            });
    }
}

#[derive(Default)]
pub struct NpsCounter {
    ticks: VecDeque<(Instant, i64)>,
}

impl NpsCounter {
    const NPS_WINDOW: f64 = 0.5;

    pub fn tick(&mut self, passed: i64) {
        self.ticks.push_back((Instant::now(), passed));
        while let Some((front_time, _passed)) = self.ticks.front() {
            if front_time.elapsed().as_secs_f64() > Self::NPS_WINDOW {
                self.ticks.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn read(&self) -> u32 {
        let old = if let Some((_time, front_passed)) = self.ticks.front() {
            *front_passed as f64
        } else {
            0.0
        };

        let last = if let Some((_time, back_passed)) = self.ticks.back() {
            *back_passed as f64
        } else {
            0.0
        };

        ((last - old).max(0.0) / Self::NPS_WINDOW).round() as u32
    }
}
