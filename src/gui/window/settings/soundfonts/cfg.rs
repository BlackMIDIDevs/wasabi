use crate::gui::window::WIN_MARGIN;

use super::SFListItem;
use egui::{Context, Window};
use xsynth_core::soundfont::{EnvelopeCurveType, Interpolator};

pub fn show_sf_config(ctx: &Context, item: &mut SFListItem) {
    let title = if let Some(path) = item.item.path.file_name() {
        format!("Config for {path:?}")
    } else {
        format!("Config for {}", item.id)
    };

    let frame = egui::Frame::inner_margin(egui::Frame::window(ctx.style().as_ref()), WIN_MARGIN);

    Window::new(title)
        .id(egui::Id::new(item.id))
        .collapsible(false)
        .title_bar(true)
        .enabled(true)
        .frame(frame)
        .open(&mut item.config_visible)
        .scroll([false, true])
        .default_height(300.0)
        .show(ctx, |ui| {
            let col_width = 220.0;

            ui.heading("Instrument");
            ui.separator();
            egui::Grid::new("sfconfig_window_instr")
                .num_columns(2)
                .min_col_width(col_width)
                .spacing(super::super::SPACING)
                .striped(true)
                .show(ui, |ui| {
                    let mut modify = item.item.options.bank.is_some();

                    ui.label("Override Instrument: ");
                    ui.add(egui::Checkbox::without_text(&mut modify));
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

            ui.add_space(super::super::CATEG_SPACE);
            ui.heading("Settings");
            ui.separator();
            egui::Grid::new("sfconfig_window_settings")
                .num_columns(2)
                .min_col_width(col_width)
                .spacing(super::super::SPACING)
                .striped(true)
                .show(ui, |ui| {
                    // Effects option
                    ui.label("Apply DSP (cutoff filter etc.):");
                    ui.checkbox(&mut item.item.options.use_effects, "");
                    ui.end_row();

                    // Interpolation option
                    ui.label("Interpolation Algorithm:");
                    egui::ComboBox::from_id_salt("interpolation_select")
                        .selected_text(format!("{:?}", item.item.options.interpolator))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut item.item.options.interpolator,
                                Interpolator::Nearest,
                                format!("{:?}", Interpolator::Nearest),
                            );
                            ui.selectable_value(
                                &mut item.item.options.interpolator,
                                Interpolator::Linear,
                                format!("{:?}", Interpolator::Linear),
                            );
                        });
                    ui.end_row();
                });

            ui.add_space(super::super::CATEG_SPACE);
            ui.horizontal(|ui| {
                ui.heading("Volume Envelope");
                ui.monospace("\u{2139}")
                    .on_hover_text("Curves are set in dB units.");
            });
            ui.separator();
            egui::Grid::new("sfconfig_window_envelope")
                .num_columns(2)
                .min_col_width(col_width)
                .spacing(super::super::SPACING)
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Attack Curve:");
                    egui::ComboBox::from_id_salt("env_attack_select")
                        .selected_text(format!(
                            "{:?}",
                            item.item.options.vol_envelope_options.attack_curve
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.attack_curve,
                                EnvelopeCurveType::Exponential,
                                format!("{:?}", EnvelopeCurveType::Exponential),
                            );
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.attack_curve,
                                EnvelopeCurveType::Linear,
                                format!("{:?}", EnvelopeCurveType::Linear),
                            );
                        });
                    ui.end_row();

                    ui.label("Decay Curve:");
                    egui::ComboBox::from_id_salt("env_decay_select")
                        .selected_text(format!(
                            "{:?}",
                            item.item.options.vol_envelope_options.decay_curve
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.decay_curve,
                                EnvelopeCurveType::Exponential,
                                format!("{:?}", EnvelopeCurveType::Exponential),
                            );
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.decay_curve,
                                EnvelopeCurveType::Linear,
                                format!("{:?}", EnvelopeCurveType::Linear),
                            );
                        });
                    ui.end_row();

                    ui.label("Release Curve:");
                    egui::ComboBox::from_id_salt("env_release_select")
                        .selected_text(format!(
                            "{:?}",
                            item.item.options.vol_envelope_options.release_curve
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.release_curve,
                                EnvelopeCurveType::Exponential,
                                format!("{:?}", EnvelopeCurveType::Exponential),
                            );
                            ui.selectable_value(
                                &mut item.item.options.vol_envelope_options.release_curve,
                                EnvelopeCurveType::Linear,
                                format!("{:?}", EnvelopeCurveType::Linear),
                            );
                        });
                    ui.end_row();
                });
        });
}
