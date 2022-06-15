mod notes_render_pass;

use std::{sync::Arc};

use vulkano::{buffer::TypedBufferAccess, image::ImageViewAbstract};

use crate::gui::{window::keyboard_layout::KeyboardView, GuiRenderer};

use self::notes_render_pass::{NotePassStatus, NoteRenderPass, NoteVertex};

pub struct ChikaraShaderTest {
    render_pass: NoteRenderPass,
}

impl ChikaraShaderTest {
    pub fn new(renderer: &GuiRenderer) -> ChikaraShaderTest {
        ChikaraShaderTest {
            render_pass: NoteRenderPass::new(renderer),
        }
    }

    fn iter_verts(len: usize) -> impl Iterator<Item = NoteVertex> {
        let color = 0xFF00FF;

        [
            NoteVertex::new(0.0, 0.5, 6, color),
            NoteVertex::new(0.1, 0.4, 64, color),
            NoteVertex::new(0.0, 0.5, 80, color),
        ]
        .into_iter()
        .cycle()
        .take(len)
    }

    pub fn draw(
        &mut self,
        key_view: &KeyboardView,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
    ) {
        let mut i = 0;

        self.render_pass.draw(final_image, key_view, |buffer| {
            let new_verts =
                ChikaraShaderTest::iter_verts(buffer.len() as usize).enumerate();

            {
                let mut verts = buffer.write().unwrap();
                for (i, v) in new_verts {
                    verts[i] = v;
                }
            }

            let value = match i {
                0 => NotePassStatus::HasMoreNotes,
                1 => NotePassStatus::Finished {
                    remaining: buffer.len() as u32,
                },
                _ => unreachable!(),
            };

            i += 1;
            value
        });
    }
}
