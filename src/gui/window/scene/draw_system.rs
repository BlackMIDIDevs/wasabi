mod notes_render_pass;

use std::{sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageViewAbstract},
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::{self, FenceSignalFuture, GpuFuture},
};

use crate::gui::GuiRenderer;

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
        let angle = start_time.elapsed().as_secs_f32() as f32 * 10.0;

        [
            NoteVertex {
                position: [angle.cos() * 0.5, angle.sin() * 0.5],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            NoteVertex {
                position: [(angle + 0.25).cos() * 0.5, (angle + 0.25).sin() * 0.5],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            NoteVertex {
                position: [(angle + 0.5).cos() * 0.5, (angle + 0.5).sin() * 0.5],
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ]
        .into_iter()
        .cycle()
        .take(len)
    }

    pub fn draw(&mut self, final_image: Arc<dyn ImageViewAbstract + 'static>) {
        let mut i = 0;

        let start_time = self.start_time;

        self.render_pass.draw(final_image, |buffer| {
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
