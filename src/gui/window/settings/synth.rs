use crate::{
    audio_playback::WasabiAudioPlayer,
    settings::{Synth, WasabiSettings},
    state::WasabiState,
};

use super::SettingsWindow;

mod kdmapi;
mod mididevice;
mod xsynth;

impl SettingsWindow {
    pub fn show_synth_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        state: &WasabiState,
        width: f32,
    ) {
        egui::Grid::new("synth_settings_grid")
            .num_columns(2)
            .spacing(super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                let synth_prev = settings.synth.synth;
                ui.label("Synthesizer:");
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("synth_select")
                        .selected_text(settings.synth.synth.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut settings.synth.synth,
                                Synth::XSynth,
                                Synth::XSynth.as_str(),
                            );
                            ui.selectable_value(
                                &mut settings.synth.synth,
                                Synth::Kdmapi,
                                Synth::Kdmapi.as_str(),
                            );
                            ui.selectable_value(
                                &mut settings.synth.synth,
                                Synth::MidiDevice,
                                Synth::MidiDevice.as_str(),
                            );
                            ui.selectable_value(
                                &mut settings.synth.synth,
                                Synth::None,
                                Synth::None.as_str(),
                            );
                        });

                    if ui
                        .button(
                            egui::WidgetText::from(" \u{1F503} ")
                                .text_style(egui::TextStyle::Name("monospace big".into())),
                        )
                        .on_hover_text("Reload Synth")
                        .clicked()
                    {
                        state.synth.switch(WasabiAudioPlayer::create_synth(
                            settings,
                            state.loading_status.clone(),
                            state.errors.clone(),
                        ));
                    }
                });
                ui.end_row();

                if settings.synth.synth != synth_prev {
                    let new_player = WasabiAudioPlayer::create_synth(
                        settings,
                        state.loading_status.clone(),
                        state.errors.clone(),
                    );
                    state.synth.switch(new_player);
                }
            });

        ui.add_space(8.0);
        ui.vertical_centered(|ui| {
            ui.small("Options marked with (*) will apply when the synth is reloaded.");
        });

        ui.add_space(super::CATEG_SPACE);
        ui.heading("Synth Settings");

        match settings.synth.synth {
            Synth::XSynth => self.show_xsynth_settings(ui, settings, state, width),
            Synth::Kdmapi => self.show_kdmapi_settings(ui, settings, state, width),
            Synth::MidiDevice => self.show_mididevice_settings(ui, settings, state, width),
            Synth::None => {
                ui.label("No Settings");
            }
        }
    }
}
