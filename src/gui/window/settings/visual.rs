use egui::WidgetText;
use egui_extras::{Column, TableBuilder};

use crate::{settings::WasabiSettings, utils::NOTE_SPEED_RANGE};

use super::SettingsWindow;

impl SettingsWindow {
    pub fn show_visual_settings(
        &mut self,
        ui: &mut egui::Ui,
        settings: &mut WasabiSettings,
        width: f32,
    ) {
        ui.heading("General");
        egui::Grid::new("general_visual_settings_grid")
            .num_columns(2)
            .spacing(super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.label("Check for updates on launch:");
                ui.checkbox(&mut settings.gui.check_for_updates, "");
                ui.end_row();

                ui.label("Enable VSync:");
                ui.checkbox(&mut settings.gui.vsync, "");
                ui.end_row();

                ui.label("Skip Control:");
                ui.add(
                    egui::DragValue::new(&mut settings.gui.skip_control)
                        .speed(0.5)
                        .range(0.0..=f64::MAX),
                );
                ui.end_row();

                ui.label("Speed Control:");
                ui.add(
                    egui::DragValue::new(&mut settings.gui.speed_control)
                        .speed(0.5)
                        .range(0.0..=f64::MAX),
                );
                ui.end_row();
            });

        ui.add_space(super::CATEG_SPACE);
        ui.heading("Scene");

        egui::Grid::new("scene_visual_settings_grid")
            .num_columns(2)
            .spacing(super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.label("Background Color: ");
                ui.color_edit_button_srgba(&mut settings.scene.bg_color);
                ui.end_row();

                ui.label("Bar Color: ");
                ui.color_edit_button_srgba(&mut settings.scene.bar_color);
                ui.end_row();

                ui.label("Keyboard Range: ");
                let mut firstkey = *settings.scene.key_range.start();
                let mut lastkey = *settings.scene.key_range.end();
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut firstkey).speed(1).range(0..=253));
                    ui.add(
                        egui::DragValue::new(&mut lastkey)
                            .speed(1)
                            .range(firstkey + 1..=254),
                    );
                });
                ui.end_row();
                if firstkey != *settings.scene.key_range.start()
                    || lastkey != *settings.scene.key_range.end()
                {
                    settings.scene.key_range = firstkey..=lastkey;
                }

                ui.label("Note Speed: ");
                ui.spacing_mut().slider_width = width / 2.0 - 100.0;
                ui.add(
                    egui::Slider::new(&mut settings.scene.note_speed, NOTE_SPEED_RANGE)
                        .logarithmic(true),
                );
                ui.end_row();
            });

        ui.add_space(super::CATEG_SPACE);
        ui.heading("Statistics");

        egui::Grid::new("stats_visual_settings_grid")
            .num_columns(2)
            .spacing(super::SPACING)
            .striped(true)
            .min_col_width(width / 2.0)
            .show(ui, |ui| {
                ui.label("Floating:");
                ui.checkbox(&mut settings.scene.statistics.floating, "");
                ui.end_row();

                ui.label("Border:");
                ui.checkbox(&mut settings.scene.statistics.border, "");
                ui.end_row();

                ui.label("Background Opacity: ");
                ui.spacing_mut().slider_width = width / 2.0 - 100.0;
                ui.add(egui::Slider::new(
                    &mut settings.scene.statistics.opacity,
                    0.0..=1.0,
                ));
                ui.end_row();
            });

        ui.add_space(8.0);
        egui::Frame::default()
            .corner_radius(egui::CornerRadius::same(8))
            .stroke(ui.style().visuals.widgets.noninteractive.bg_stroke)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .striped(true)
                    .cell_layout(egui::Layout::centered_and_justified(
                        egui::Direction::LeftToRight,
                    ))
                    .resizable(true)
                    .column(Column::exact(40.0).resizable(false))
                    .column(Column::exact(width - 150.0).resizable(false))
                    .column(Column::exact(110.0).resizable(false))
                    .body(|mut body| {
                        let row_height = super::SPACING[1] * 3.0;
                        let mut temp = settings.scene.statistics.order.clone();
                        for (i, item) in settings.scene.statistics.order.iter_mut().enumerate() {
                            body.row(row_height, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(12.0);
                                        ui.checkbox(&mut temp[i].1, "");
                                    });
                                });
                                row.col(|ui| {
                                    ui.label(item.0.as_str());
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        if ui
                                            .button(WidgetText::from(" \u{2191} ").text_style(
                                                egui::TextStyle::Name("monospace big".into()),
                                            ))
                                            .clicked()
                                            && i > 0
                                        {
                                            temp.swap(i, i - 1);
                                        }

                                        if ui
                                            .button(WidgetText::from(" \u{2193} ").text_style(
                                                egui::TextStyle::Name("monospace big".into()),
                                            ))
                                            .clicked()
                                            && i < temp.len() - 1
                                        {
                                            temp.swap(i, i + 1);
                                        }
                                    });
                                });
                            });
                        }
                        settings.scene.statistics.order = temp;
                    });
            });
    }
}
