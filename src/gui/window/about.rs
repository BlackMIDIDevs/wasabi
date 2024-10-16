use crate::{state::WasabiState, utils};
use std::env::consts::{ARCH, OS};

use super::{GuiWasabiWindow, WasabiError};

impl GuiWasabiWindow {
    pub fn show_about(&mut self, ctx: &egui::Context, state: &mut WasabiState) {
        let frame = utils::create_window_frame(ctx);
        let size = [600.0, 460.0];

        let mut updcheck = false;

        egui::Window::new("About Wasabi")
            .resizable(true)
            .collapsible(false)
            .title_bar(true)
            .scroll([false, true])
            .enabled(true)
            .frame(frame)
            .fixed_size(size)
            .open(&mut state.show_about)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let image_size = 84.0;

                    let title_size = 40.0;
                    let titleid = egui::FontId {
                        size: title_size,
                        ..Default::default()
                    };

                    let title_text = "Wasabi";
                    let title_galley = ui.painter().layout_no_wrap(
                        title_text.to_owned(),
                        titleid,
                        egui::Color32::WHITE,
                    );

                    let logo_width =
                        image_size + ui.spacing().item_spacing.x + title_galley.size().x;
                    let space = ui.available_width() / 2.0 - (logo_width + 4.0) / 2.0;

                    ui.add_space(space);
                    ui.add(
                        egui::Image::new(egui::include_image!("../../../assets/logo.svg"))
                            .fit_to_exact_size(egui::Vec2::new(image_size, image_size)),
                    );
                    ui.add_space(4.0);

                    ui.vertical(|ui| {
                        let text_height = title_galley.size().y;
                        let space = (image_size - text_height) / 2.0;
                        ui.add_space(space);

                        ui.label(egui::RichText::new(title_text).size(title_size));
                    })
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                let col_width = size[0] / 2.0;
                ui.heading("Build Information");
                egui::Grid::new("buildinfo_grid")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Version:");
                        ui.label(env!("CARGO_PKG_VERSION"));
                        ui.end_row();

                        ui.label("Operating System:");
                        ui.label(OS.to_string());
                        ui.end_row();

                        ui.label("Architecture:");
                        ui.label(ARCH.to_string());
                        ui.end_row();
                    });

                ui.add_space(20.0);

                ui.heading("Libraries");
                egui::Grid::new("libraries_grid")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Vulkano Version:");
                        ui.label("0.34");
                        ui.end_row();

                        ui.label("Egui Version:");
                        ui.label("0.29");
                        ui.end_row();

                        ui.label("Winit Version:");
                        ui.label("0.30");
                        ui.end_row();

                        ui.label("XSynth Version:");
                        ui.label("0.3.1");
                        ui.end_row();

                        ui.label("MIDI Toolkit Version:");
                        ui.label("0.1.0");
                        ui.end_row();
                    });

                let gh_text = "\u{1F310} GitHub";
                let gh_galley = ui.painter().layout_no_wrap(
                    gh_text.to_owned(),
                    ctx.style()
                        .text_styles
                        .iter()
                        .find(|v| v.0 == &egui::TextStyle::Button)
                        .unwrap()
                        .1
                        .clone(),
                    egui::Color32::WHITE,
                );

                let upd_text = "\u{1F310} Check for updates";
                let upd_galley = ui.painter().layout_no_wrap(
                    upd_text.to_owned(),
                    ctx.style()
                        .text_styles
                        .iter()
                        .find(|v| v.0 == &egui::TextStyle::Button)
                        .unwrap()
                        .1
                        .clone(),
                    egui::Color32::WHITE,
                );

                let mut h = ui.available_height();

                let button_height = ui.spacing().button_padding.y * 2.0 + gh_galley.size().y;
                h -= button_height;
                ui.add_space(h);

                ui.horizontal(|ui| {
                    let w = ui.available_width();

                    let button_width = gh_galley.size().x
                        + upd_galley.size().x
                        + ui.spacing().button_padding.x * 4.0;
                    let w = w / 2.0 - button_width / 2.0;
                    ui.add_space(w);

                    if ui.button(gh_text).clicked() {
                        open::that("https://github.com/BlackMIDIDevs/wasabi").unwrap_or_else(|e| {
                            state.errors.error(&WasabiError::Other(e.to_string()))
                        });
                    }
                    if ui.button(upd_text).clicked() {
                        updcheck = true;
                    }
                });
            });

        if updcheck {
            crate::utils::check_for_updates(state);
        }
    }
}
