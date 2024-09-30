use std::sync::{Arc, RwLock};

use xsynth_realtime::ThreadCount;

use crate::{audio_playback::WasabiAudioPlayer, settings::WasabiSettings};

use super::SettingsWindow;

impl SettingsWindow {
    pub fn show_xsynth_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        width: f32,
        synth: Arc<RwLock<WasabiAudioPlayer>>,
    ) {
        egui::Grid::new("xsynth_settings_grid")
            .num_columns(2)
            .spacing(super::super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                let layer_limit_prev = settings.synth.xsynth.limit_layers;

                ui.label("Enable Layer Limiting: ");
                ui.checkbox(&mut settings.synth.xsynth.limit_layers, "");
                ui.end_row();

                let layer_count_prev = settings.synth.xsynth.layers;

                ui.horizontal(|ui| {
                    ui.label("Layer Limit:");
                    ui.monospace("\u{2139}")
                        .on_hover_text("One layer is one voice per key per channel.");
                });
                ui.add_enabled(
                    settings.synth.xsynth.limit_layers,
                    egui::DragValue::new(&mut settings.synth.xsynth.layers)
                        .speed(1)
                        .range(1..=usize::MAX),
                );
                ui.end_row();

                if settings.synth.xsynth.layers != layer_count_prev
                    || layer_limit_prev != settings.synth.xsynth.limit_layers
                {
                    synth.write().unwrap().configure(&settings.synth);
                }

                let buffer_prev = settings.synth.xsynth.config.render_window_ms;
                ui.label("Render Buffer (ms):");
                ui.add(
                    egui::DragValue::new(&mut settings.synth.xsynth.config.render_window_ms)
                        .speed(0.1)
                        .range(0.0001..=1000.0),
                );
                ui.end_row();
                if settings.synth.xsynth.config.render_window_ms != buffer_prev {
                    synth.write().unwrap().configure(&settings.synth);
                }

                ui.label("Ignore velocities between:");
                let mut lovel = *settings.synth.xsynth.config.ignore_range.start();
                let mut hivel = *settings.synth.xsynth.config.ignore_range.end();
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut lovel).speed(1).range(0..=127));
                    ui.label("and");
                    ui.add(egui::DragValue::new(&mut hivel).speed(1).range(lovel..=127));
                });
                ui.end_row();
                if lovel != *settings.synth.xsynth.config.ignore_range.start()
                    || hivel != *settings.synth.xsynth.config.ignore_range.end()
                {
                    settings.synth.xsynth.config.ignore_range = lovel..=hivel;
                    synth.write().unwrap().configure(&settings.synth);
                }

                ui.label("Fade out voice when killing it*: ");
                ui.checkbox(
                    &mut settings
                        .synth
                        .xsynth
                        .config
                        .channel_init_options
                        .fade_out_killing,
                    "",
                );
                ui.end_row();

                let mut threading =
                    settings.synth.xsynth.config.multithreading == ThreadCount::Auto;
                ui.label("Enable multithreading*: ");
                ui.checkbox(&mut threading, "");
                ui.end_row();
                if threading {
                    settings.synth.xsynth.config.multithreading = ThreadCount::Auto;
                } else {
                    settings.synth.xsynth.config.multithreading = ThreadCount::None;
                }
            });
    }
}
