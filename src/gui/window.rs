use egui::Visuals;
use vulkano::format::Format;

use super::{scene::GuiRenderScene, GuiRenderer, GuiState};

pub struct GuiWasabiWindow {
    show_texture_window1: bool,
    show_texture_window2: bool,
    show_texture_window3: bool,
    image_texture_id1: egui::TextureId,
    image_texture_id2: egui::TextureId,
    render_scene: GuiRenderScene,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer) -> GuiWasabiWindow {
        // tree.png asset is from https://github.com/sotrh/learn-wgpu/tree/master/docs/beginner/tutorial5-textures
        let image_texture_id1 = renderer.gui.register_user_image(
            include_bytes!("../../assets/tree.png"),
            Format::R8G8B8A8_UNORM,
        );
        let image_texture_id2 = renderer.gui.register_user_image(
            include_bytes!("../../assets/doge2.png"),
            Format::R8G8B8A8_UNORM,
        );

        GuiWasabiWindow {
            show_texture_window1: true,
            show_texture_window2: true,
            show_texture_window3: true,
            image_texture_id1,
            image_texture_id2,
            render_scene: GuiRenderScene::new(renderer),
        }
    }

    /// Defines the layout of our UI
    pub fn layout(&mut self, state: &mut GuiState) {
        let egui_context = state.gui.context();

        egui_context.set_visuals(Visuals::dark());
        egui::SidePanel::left("Side Panel")
            .default_width(150.0)
            .show(&egui_context, |ui| {
                ui.heading("Hello Tree");
                ui.separator();
                ui.checkbox(&mut self.show_texture_window1, "Show Tree");
                ui.checkbox(&mut self.show_texture_window2, "Show Doge");
                ui.checkbox(&mut self.show_texture_window3, "Show Scene");
            });

        egui::Window::new("Mah Tree")
            .resizable(true)
            .vscroll(true)
            .open(&mut self.show_texture_window1)
            .show(&egui_context, |ui| {
                ui.image(self.image_texture_id1, [256.0, 256.0]);
            });

        egui::Window::new("Mah Doge")
            .resizable(true)
            .vscroll(true)
            .open(&mut self.show_texture_window2)
            .show(&egui_context, |ui| {
                ui.image(self.image_texture_id2, [300.0, 200.0]);
            });

        egui::Window::new("Mah Scene")
            .resizable(true)
            .vscroll(true)
            .open(&mut self.show_texture_window3)
            .show(&egui_context, |mut ui| {
                self.render_scene.layout(state, &mut ui)
            });
    }
}
