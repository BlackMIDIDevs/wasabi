use crate::gui::window::WIN_MARGIN;

use super::{SFFormat, SFListItem};
use egui::{Context, Window};
use xsynth_core::soundfont::Interpolator;

#[derive(Clone)]
pub struct SoundfontConfigWindow {
    pub visible: bool,
    id: usize,
}

impl SoundfontConfigWindow {
    pub fn new(id: usize) -> Self {
        Self { visible: true, id }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn show(&mut self, ctx: &Context, item: &mut SFListItem) {
        let title = if let Some(path) = item.item.path.file_name() {
            format!("Config for {path:?}")
        } else {
            format!("Config for {}", self.id)
        };

        let frame =
            egui::Frame::inner_margin(egui::Frame::window(ctx.style().as_ref()), WIN_MARGIN);

        Window::new(title)
            .id(egui::Id::new(self.id))
            .collapsible(false)
            .title_bar(true)
            .enabled(true)
            .frame(frame)
            .open(&mut self.visible)
            .show(ctx, |ui| {
                let col_width = 80.0;

                ui.heading("Instrument");
                ui.separator();
                egui::Grid::new("sfconfig_window_instr")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .show(ui, |ui| {
                        let mut modify = item.item.options.bank.is_some();

                        ui.label("Override Instrument: ");
                        let allow_override = !(item.format == SFFormat::Sfz);
                        ui.add_enabled(allow_override, egui::Checkbox::without_text(&mut modify));
                        ui.end_row();

                        if modify && item.item.options.bank.is_none() {
                            item.item.options.bank = Some(0);
                            item.item.options.preset = Some(0);
                        } else if !modify {
                            item.item.options.bank = None;
                            item.item.options.preset = None;
                        }

                        let mut bank = item.item.options.bank.unwrap_or(0);

                        ui.label("Bank: ");
                        ui.add_enabled(
                            modify,
                            egui::DragValue::new(&mut bank).speed(1).range(0..=128),
                        );
                        ui.end_row();

                        if bank != item.item.options.bank.unwrap_or(0) {
                            item.item.options.bank = Some(bank)
                        }

                        let mut preset = item.item.options.preset.unwrap_or(0);

                        ui.label("Preset: ");
                        ui.add_enabled(
                            modify,
                            egui::DragValue::new(&mut preset).speed(1).range(0..=127),
                        );
                        ui.end_row();

                        if preset != item.item.options.preset.unwrap_or(0) {
                            item.item.options.preset = Some(preset)
                        }
                    });

                ui.heading("Settings");
                ui.separator();
                egui::Grid::new("sfconfig_window_settings")
                    .num_columns(2)
                    .min_col_width(col_width)
                    .show(ui, |ui| {
                        // ui.label("Linear Release Envelope: ");
                        // ui.checkbox(&mut item.init.linear_release, "");
                        // ui.end_row();

                        let interp = ["Nearest Neighbor", "Linear"];
                        let mut interp_idx = item.item.options.interpolator as usize;

                        ui.label("Interpolation:");
                        egui::ComboBox::from_id_salt("interpolation").show_index(
                            ui,
                            &mut interp_idx,
                            interp.len(),
                            |i| interp[i].to_owned(),
                        );
                        ui.end_row();

                        if interp_idx != item.item.options.interpolator as usize {
                            match interp_idx {
                                0 => item.item.options.interpolator = Interpolator::Nearest,
                                1 => item.item.options.interpolator = Interpolator::Linear,
                                _ => item.item.options.interpolator = Interpolator::Nearest,
                            };
                        }
                    });
            });
    }
}
