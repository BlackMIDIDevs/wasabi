mod cake_system;
mod note_list_system;

use egui::{Image, Ui};

use crate::{
    midi::{MIDIColor, MIDIFileUnion},
    scenes::SceneSwapchain,
};

use self::{cake_system::CakeRenderer, note_list_system::NoteRenderer};

use super::{keyboard_layout::KeyboardView, GuiRenderer, GuiState};

enum CurrentRenderer {
    Note(NoteRenderer),
    Cake(CakeRenderer),
    None,
}

impl CurrentRenderer {
    fn get_note_renderer(&mut self, renderer: &GuiRenderer) -> &mut NoteRenderer {
        match self {
            CurrentRenderer::Note(renderer) => renderer,
            _ => {
                let renderer = NoteRenderer::new(renderer);
                *self = CurrentRenderer::Note(renderer);
                match self {
                    CurrentRenderer::Note(renderer) => renderer,
                    _ => unreachable!(),
                }
            }
        }
    }

    fn get_cake_renderer(&mut self, renderer: &GuiRenderer) -> &mut CakeRenderer {
        match self {
            CurrentRenderer::Cake(renderer) => renderer,
            _ => {
                let renderer = CakeRenderer::new(renderer);
                *self = CurrentRenderer::Cake(renderer);
                match self {
                    CurrentRenderer::Cake(renderer) => renderer,
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub struct GuiRenderScene {
    swap_chain: SceneSwapchain,
    draw_system: CurrentRenderer,
}

pub struct RenderResultData {
    pub notes_rendered: u64,
    pub key_colors: Vec<Option<MIDIColor>>,
}

impl GuiRenderScene {
    pub fn new(renderer: &GuiRenderer) -> Self {
        Self {
            swap_chain: SceneSwapchain::new(renderer.device.clone()),
            draw_system: CurrentRenderer::None,
        }
    }

    pub fn draw(
        &mut self,
        state: &mut GuiState,
        ui: &mut Ui,
        key_view: &KeyboardView,
        midi_file: &mut MIDIFileUnion,
        view_range: f64,
    ) -> RenderResultData {
        let size = ui.available_size();
        let size = [size.x as u32, size.y as u32];

        let scene_image = self.swap_chain.get_next_image(state, size);
        let frame = scene_image.image.clone();

        let result = match midi_file {
            MIDIFileUnion::InRam(file) => self
                .draw_system
                .get_note_renderer(state.renderer)
                .draw(key_view, frame, file, view_range),

            MIDIFileUnion::Live(file) => self
                .draw_system
                .get_note_renderer(state.renderer)
                .draw(key_view, frame, file, view_range),

            MIDIFileUnion::Cake(file) => self
                .draw_system
                .get_cake_renderer(state.renderer)
                .draw(key_view, frame, file, view_range),
        };

        let img = Image::new((scene_image.id, [size[0] as f32, size[1] as f32].into()));
        ui.add(img);

        result
    }
}
