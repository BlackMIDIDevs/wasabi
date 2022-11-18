use egui::{Frame, Pos2, Context};

use crate::{
    gui::window::GuiWasabiWindow,
    midi::MIDIFileBase,
};

pub struct GuiMidiStats {
    time_passed: f64,
    time_total: f64,
    notes_total: usize,
    notes_on_screen: u64,
    voice_count: u64,
}

impl GuiMidiStats {
    pub fn empty() -> GuiMidiStats {
        GuiMidiStats {
            time_passed: 0.0,
            time_total: 0.0,
            notes_total: 0,
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

pub fn draw_stats(
    win: &mut GuiWasabiWindow,
    ctx: &Context,
    pos: Pos2,
    mut stats: GuiMidiStats,
) {
    let stats_frame = Frame::default()
        .inner_margin(egui::style::Margin::same(6.0))
        .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 175))
        .rounding(egui::Rounding::same(6.0));

    egui::Window::new("Stats")
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .scroll2([false, false])
        .enabled(true)
        .frame(stats_frame)
        .fixed_pos(pos)
        .show(&ctx, |ui| {
            let mut time_sec: u64 = 0;
            let mut time_min: u64 = 0;
            let mut length_sec: u64 = 0;
            let mut length_min: u64 = 0;

            if let Some(midi_file) = win.midi_file.as_mut() {
                stats.time_total = if let Some(length) = midi_file.midi_length() {
                    length
                } else {
                    0.0
                };
                let time = midi_file.timer().get_time().as_secs_f64();
                let length_u64 = stats.time_total as u64;
                length_sec = length_u64 % 60;
                length_min = (length_u64 / 60) % 60;
                if time > stats.time_total {
                    stats.time_passed = stats.time_total;
                } else {
                    stats.time_passed = time;
                }
                let time_u64 = stats.time_passed as u64;
                time_sec = time_u64 % 60;
                time_min = (time_u64 / 60) % 60;

                stats.notes_total = midi_file.stats().total_notes;
            }
            ui.monospace(format!(
                "Time: {:0width$}:{:0width$}/{:0width$}:{:0width$}",
                time_min,
                time_sec,
                length_min,
                length_sec,
                width = 2
            ));
            ui.monospace(format!("FPS: {}", win.fps.get_fps().round()));
            ui.monospace(format!("Total Notes: {}", stats.notes_total));
            ui.monospace(format!("Voice Count: {}", stats.voice_count));
            ui.monospace(format!("Rendered: {}", stats.notes_on_screen));
        });
}
