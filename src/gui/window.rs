use egui::Visuals;

use super::{scene::GuiRenderScene, GuiRenderer, GuiState};

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer) -> GuiWasabiWindow {
        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
        }
    }

    /// Defines the layout of our UI
    pub fn layout(&mut self, state: &mut GuiState) {
        let egui_context = state.gui.context();

        egui_context.set_visuals(Visuals::dark());

        egui::TopBottomPanel::top("Top panel")
            .height_range(100.0..=100.0)
            .show(&egui_context, |ui| ui.heading("Settings or whatever here"));

        egui::TopBottomPanel::bottom("Keyboard panel")
            .height_range(100.0..=100.0)
            .show(&egui_context, |ui| ui.heading("Keyboard here"));

        egui::CentralPanel::default().show(&egui_context, |mut ui| {
            self.render_scene.layout(state, &mut ui)
        });
    }
}
