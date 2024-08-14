use egui::{Context, Frame, Pos2};

use crate::{gui::window::GuiWasabiWindow, midi::MIDIFileBase};

pub struct GuiMidiStats {
    time_passed: f64,
    time_total: f64,
    notes_on_screen: u64,
    voice_count: u64,
}

impl GuiMidiStats {
    pub fn empty() -> GuiMidiStats {
        GuiMidiStats {
            time_passed: 0.0,
            time_total: 0.0,
            notes_on_screen: 0,
            voice_count: 0,
        }
    }

    pub fn set_voice_count(&mut self, voices: u64) {
        self.voice_count = voices;
    }

    pub fn set_rendered_note_count(&mut self, notes: u64) {
        self.notes_on_screen = notes;
    }
}

pub fn draw_stats(win: &mut GuiWasabiWindow, ctx: &Context, pos: Pos2, mut stats: GuiMidiStats) {
    let onepx = ctx.pixels_per_point();

    let stats_frame = Frame::default()
        .inner_margin(egui::style::Margin::same(7.0))
        .fill(egui::Color32::from_rgba_unmultiplied(7, 7, 7, 200))
        .stroke(egui::Stroke::new(
            onepx,
            egui::Color32::from_rgb(50, 50, 50),
        ))
        .rounding(egui::Rounding::same(6.0));

    egui::Window::new("Stats")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .scroll2([false, false])
        .enabled(true)
        .frame(stats_frame)
        .fixed_pos(pos)
        .fixed_size(egui::Vec2::new(200.0, 128.0))
        .show(ctx, |ui| {
            let mut time_millis: u64 = 0;
            let mut time_sec: u64 = 0;
            let mut time_min: u64 = 0;
            let mut length_millis: u64 = 0;
            let mut length_sec: u64 = 0;
            let mut length_min: u64 = 0;

            let mut note_stats = Default::default();

            if let Some(midi_file) = win.midi_file.as_mut() {
                stats.time_total = midi_file.midi_length().unwrap_or(0.0);
                let time = midi_file.timer().get_time().as_secs_f64();

                length_millis = (stats.time_total * 10.0) as u64 % 10;
                length_sec = stats.time_total as u64 % 60;
                length_min = stats.time_total as u64 / 60;

                if time > stats.time_total {
                    stats.time_passed = stats.time_total;
                } else {
                    stats.time_passed = time;
                }

                time_millis = (stats.time_passed * 10.0) as u64 % 10;
                time_sec = stats.time_passed as u64 % 60;
                time_min = stats.time_passed as u64 / 60;

                note_stats = midi_file.stats();
            }

            ui.horizontal(|ui| {
                ui.monospace("Time:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(format!(
                        "{:0width$}:{:0width$}.{} / {:0width$}:{:0width$}.{}",
                        time_min,
                        time_sec,
                        time_millis,
                        length_min,
                        length_sec,
                        length_millis,
                        width = 2
                    ));
                });
            });

            ui.horizontal(|ui| {
                ui.monospace("FPS:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(format!("{}", win.fps.get_fps().round()));
                });
            });

            ui.horizontal(|ui| {
                ui.monospace("Voice Count:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(format!("{}", stats.voice_count));
                });
            });

            ui.horizontal(|ui| {
                ui.monospace("Rendered:");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.monospace(format!("{}", stats.notes_on_screen));
                });
            });

            fn num_or_q(num: Option<impl ToString>) -> String {
                if let Some(num) = num {
                    num.to_string()
                } else {
                    "?".to_string()
                }
            }

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.monospace(format!(
                    "{} / {}",
                    num_or_q(note_stats.passed_notes),
                    num_or_q(note_stats.total_notes)
                ));
            });
        });
}
