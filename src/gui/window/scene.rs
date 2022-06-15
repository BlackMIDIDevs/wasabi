mod draw_system;

use egui::Ui;

use crate::scenes::SceneSwapchain;

use self::draw_system::ChikaraShaderTest;

use super::{GuiRenderer, GuiState, keyboard_layout::KeyboardView};

pub struct GuiRenderScene {
    swap_chain: SceneSwapchain,
    draw_system: ChikaraShaderTest,
}

impl GuiRenderScene {
    pub fn new(renderer: &GuiRenderer) -> Self {
        let draw_system = ChikaraShaderTest::new(renderer);
        Self {
            swap_chain: SceneSwapchain::new(renderer.device.clone()),
            draw_system,
        }
    }

    pub fn layout(&mut self, state: &mut GuiState, ui: &mut Ui, key_view: &KeyboardView) {
        let size = ui.available_size();
        let size = [size.x as u32, size.y as u32];

        let scene_image = self.swap_chain.get_next_image(state, size);

        self.draw_system.draw(key_view, scene_image.image.clone());

        ui.image(scene_image.id, [size[0] as f32, size[1] as f32]);
    }
}
