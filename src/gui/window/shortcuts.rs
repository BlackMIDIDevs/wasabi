use crate::state::WasabiState;

use super::GuiWasabiWindow;

impl GuiWasabiWindow {
    pub fn show_shortcuts(&mut self, ctx: &egui::Context, state: &mut WasabiState) {
        let frame =
            egui::Frame::inner_margin(egui::Frame::window(ctx.style().as_ref()), super::WIN_MARGIN);
        let size = [400.0, 210.0];

        egui::Window::new("Keyboard Shortcuts")
            .collapsible(false)
            .title_bar(true)
            .scroll([false, true])
            .enabled(true)
            .frame(frame)
            .fixed_size(size)
            .open(&mut state.show_shortcuts)
            .show(ctx, |ui| {
                let col_width = size[0] / 2.0;
                egui::Grid::new("shortcuts_grid")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Play / Pause Playback");
                        ui.label("Space");
                        ui.end_row();

                        ui.label("Skip Forward");
                        ui.label("Right Arrow");
                        ui.end_row();

                        ui.label("Go Back");
                        ui.label("Left Arrow");
                        ui.end_row();

                        ui.label("Slower Note Speed");
                        ui.label("Up Arrow");
                        ui.end_row();

                        ui.label("Faster Note Speed");
                        ui.label("Down Arrow");
                        ui.end_row();

                        ui.label("Toggle Fullscreen");
                        ui.label("Alt + Enter");
                        ui.end_row();

                        ui.label("Toggle Panel");
                        ui.label("Ctrl + F");
                        ui.end_row();

                        ui.label("Toggle Statistics");
                        ui.label("Ctrl + G");
                        ui.end_row();

                        ui.label("Open MIDI");
                        ui.label("Ctrl + O");
                        ui.end_row();

                        ui.label("Reset Synthesizer");
                        ui.label("Insert");
                        ui.end_row();
                    });
            });
    }
}
