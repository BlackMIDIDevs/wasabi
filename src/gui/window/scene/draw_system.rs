mod notes_render_pass;

use std::{sync::Arc, time::Instant};

use vulkano::{buffer::TypedBufferAccess, image::ImageViewAbstract};

use crate::gui::{window::keyboard_layout::KeyboardView, GuiRenderer};

use self::notes_render_pass::{NotePassStatus, NoteRenderPass, NoteVertex};

pub struct ChikaraShaderTest {
    render_pass: NoteRenderPass,
    start_time: Instant,
}

impl ChikaraShaderTest {
    pub fn new(renderer: &GuiRenderer) -> ChikaraShaderTest {
        ChikaraShaderTest {
            start_time: Instant::now(),
            render_pass: NoteRenderPass::new(renderer),
        }
    }

    fn iter_verts(start_time: Instant, len: usize) -> impl Iterator<Item = NoteVertex> {
        [
            NoteVertex {
                start_length: [0.0, 0.5],
                key_color: 6,
            },
            NoteVertex {
                start_length: [0.0, 0.5],
                key_color: 64,
            },
            NoteVertex {
                start_length: [0.0, 0.5],
                key_color: 80,
            },
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

        let start_time = self.start_time;

        self.render_pass.draw(final_image, key_view, |buffer| {
            let new_verts =
                ChikaraShaderTest::iter_verts(start_time, buffer.len() as usize).enumerate();

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
