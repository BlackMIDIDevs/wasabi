use std::sync::{Arc, RwLock};

use egui::Context;

use crate::utils;

#[derive(Default, Clone)]
struct StatusInfoHolder {
    title: String,
    message: String,
}

pub enum LoadingType {
    Midi,
    SoundFont,
}

impl std::fmt::Display for LoadingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingType::Midi => write!(f, "Loading MIDI..."),
            LoadingType::SoundFont => write!(f, "Loading SoundFont..."),
        }
    }
}

pub struct LoadingStatus(RwLock<Option<StatusInfoHolder>>);

impl LoadingStatus {
    pub fn new() -> Arc<Self> {
        Arc::new(Self(RwLock::new(None)))
    }

    pub fn create(&self, loading_type: LoadingType, message: String) {
        *self.0.write().unwrap() = Some(StatusInfoHolder {
            title: loading_type.to_string(),
            message,
        });
    }

    pub fn is_loading(&self) -> bool {
        self.0.read().unwrap().is_some()
    }

    pub fn update_message(&self, new_message: String) {
        if let Some(info) = self.0.write().unwrap().as_mut() {
            info.message = new_message.clone();
        }
    }

    pub fn clear(&self) {
        *self.0.write().unwrap() = None;
    }

    pub fn show(&self, ctx: &Context) {
        if let Some(info) = self.0.read().unwrap().as_ref() {
            let frame = utils::create_window_frame(ctx);

            egui::Window::new(&info.title)
                .frame(frame)
                .resizable(false)
                .collapsible(false)
                .title_bar(true)
                .enabled(true)
                .movable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        let rotation = ui.input(|i| i.time) as f32;
                        ui.add(
                            egui::Image::new(egui::include_image!("../../../assets/logo.svg"))
                                .rotate(rotation, egui::Vec2::splat(0.5))
                                .fit_to_exact_size([56.0, 56.0].into()),
                        );
                        ui.label(&info.message);
                    });
                });
        }
    }
}
