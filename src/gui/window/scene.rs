mod draw_system;

use egui::Ui;

use crate::{midi::MIDIFileViewsUnion, scenes::SceneSwapchain};

use self::draw_system::{NoteRenderer, RenderResultData};

use super::{keyboard_layout::KeyboardView, GuiRenderer, GuiState};

pub struct GuiRenderScene {
    swap_chain: SceneSwapchain,
    draw_system: NoteRenderer,
}

impl GuiRenderScene {
    pub fn new(renderer: &GuiRenderer) -> Self {
        let draw_system = NoteRenderer::new(renderer);
        Self {
            swap_chain: SceneSwapchain::new(renderer.device.clone()),
            draw_system,
        }
    }

    pub fn draw(
        &mut self,
        state: &mut GuiState,
        ui: &mut Ui,
        key_view: &KeyboardView,
        views: &MIDIFileViewsUnion,
    ) -> RenderResultData {
        let size = ui.available_size();
        let size = [size.x as u32, size.y as u32];

        let scene_image = self.swap_chain.get_next_image(state, size);
        let frame = scene_image.image.clone();

        let result = match views {
            MIDIFileViewsUnion::InRam(views) => self.draw_system.draw(key_view, frame, views),
        };

        ui.image(scene_image.id, [size[0] as f32, size[1] as f32]);

        result
    }
}
