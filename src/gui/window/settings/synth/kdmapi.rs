use crate::settings::WasabiSettings;

use super::SettingsWindow;

impl SettingsWindow {
    pub fn show_kdmapi_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        width: f32,
    ) {
        egui::Grid::new("kdmapi_settings_grid")
            .num_columns(2)
            .spacing(super::super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.label("Use the driver's soundfont list*:");
                ui.checkbox(&mut settings.synth.kdmapi.use_om_sflist, "");
                ui.end_row();
            });
    }
}
