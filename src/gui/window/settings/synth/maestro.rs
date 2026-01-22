use crate::settings::WasabiSettings;

use super::SettingsWindow;

impl SettingsWindow {
    pub fn show_maestro_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        width: f32,
    ) {
        egui::Grid::new("maestro_settings_grid")
            .num_columns(2)
            .spacing(super::super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Enable the functionality of MIDI ports*:");
                    ui.monospace("\u{2139}")
                        .on_hover_text("When enabled, the MIDI port of each track will be obeyed.\nElse all events will be sent to a single port.");
                });
                ui.checkbox(&mut settings.synth.maestro.use_ports, "");
                ui.end_row();

                ui.label("Number of active MIDI ports*:");
                ui.add_enabled(
                    settings.synth.maestro.use_ports,
                    egui::DragValue::new(&mut settings.synth.maestro.num_ports)
                        .speed(1)
                        .range(1..=16),
                );
                ui.end_row();
            });
    }
}
