use egui::{Context, Visuals};
use egui_winit_vulkano::Gui;
use vulkano::format::Format;

pub struct GuiWasabiWindow {
    show_texture_window1: bool,
    show_texture_window2: bool,
    image_texture_id1: egui::TextureId,
    image_texture_id2: egui::TextureId,
}

impl GuiWasabiWindow {
    pub fn new(gui: &mut Gui) -> GuiWasabiWindow {
        // tree.png asset is from https://github.com/sotrh/learn-wgpu/tree/master/docs/beginner/tutorial5-textures
        let image_texture_id1 = gui.register_user_image(
            include_bytes!("./old/assets/tree.png"),
            Format::R8G8B8A8_UNORM,
        );
        let image_texture_id2 = gui.register_user_image(
            include_bytes!("./old/assets/doge2.png"),
            Format::R8G8B8A8_UNORM,
        );

        GuiWasabiWindow {
            show_texture_window1: true,
            show_texture_window2: true,
            image_texture_id1,
            image_texture_id2,
        }
    }

    /// Defines the layout of our UI
    pub fn layout(&mut self, egui_context: Context) {
        egui_context.set_visuals(Visuals::dark());
        egui::SidePanel::left("Side Panel")
            .default_width(150.0)
            .show(&egui_context, |ui| {
                ui.heading("Hello Tree");
                ui.separator();
                ui.checkbox(&mut self.show_texture_window1, "Show Tree");
                ui.checkbox(&mut self.show_texture_window2, "Show Doge");
            });
        let show_texture_window1 = &mut self.show_texture_window1;
        let show_texture_window2 = &mut self.show_texture_window2;
        let image_texture_id1 = self.image_texture_id1;
        egui::Window::new("Mah Tree")
            .resizable(true)
            .vscroll(true)
            .open(show_texture_window1)
            .show(&egui_context, |ui| {
                ui.image(image_texture_id1, [256.0, 256.0]);
            });
        let image_texture_id2 = self.image_texture_id2;
        egui::Window::new("Mah Doge")
            .resizable(true)
            .vscroll(true)
            .open(show_texture_window2)
            .show(&egui_context, |ui| {
                ui.image(image_texture_id2, [300.0, 200.0]);
            });
    }
}
