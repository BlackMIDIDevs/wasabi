// Copyright (c) 2017 The vulkano developers <=== !
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

// This is a simplified version of the example. See that for commented version of this code.
// https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/deferred/frame/system.rs
// Egui drawing could be its own pass or it could be a deferred subpass

use std::{convert::TryFrom, sync::Arc};

use vulkano::{
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        SecondaryCommandBuffer, SubpassContents,
    },
    device::Queue,
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageViewAbstract},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::GpuFuture,
};

/// System that contains the necessary facilities for rendering a single frame.
pub struct FrameSystem {
    gfx_queue: Arc<Queue>,
    render_pass: Arc<RenderPass>,
    depth_buffer: Arc<ImageView<AttachmentImage>>,
}

impl FrameSystem {
    pub fn new(gfx_queue: Arc<Queue>, final_output_format: Format) -> FrameSystem {
        let render_pass = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: final_output_format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            passes: [
                {
                    color: [final_color],
                    depth_stencil: {depth},
                    input: []
                }
            ]
        )
        .unwrap();
        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient_input_attachment(
                gfx_queue.device().clone(),
                [1, 1],
                Format::D16_UNORM,
            )
            .unwrap(),
        )
        .unwrap();
        FrameSystem {
            gfx_queue,
            render_pass,
            depth_buffer,
        }
    }

    #[inline]
    pub fn deferred_subpass(&self) -> Subpass {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }

    pub fn draw_frame(
        &mut self,
        before_future: impl GpuFuture + 'static,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
        render: impl FnOnce(DrawPass) -> (),
    ) -> Box<dyn GpuFuture> {
        let img_dims = final_image.image().dimensions().width_height();
        if self.depth_buffer.image().dimensions().width_height() != img_dims {
            self.depth_buffer = ImageView::new_default(
                AttachmentImage::transient_input_attachment(
                    self.gfx_queue.device().clone(),
                    img_dims,
                    Format::D16_UNORM,
                )
                .unwrap(),
            )
            .unwrap();
        }
        let framebuffer = Framebuffer::new(
            self.render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![final_image, self.depth_buffer.clone()],
                ..Default::default()
            },
        )
        .unwrap();
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        command_buffer_builder
            .begin_render_pass(
                framebuffer.clone(),
                SubpassContents::SecondaryCommandBuffers,
                vec![[0.0, 0.0, 0.0, 0.0].into(), 1.0f32.into()],
            )
            .unwrap();

        let mut frame = Frame {
            system: self,
            before_main_cb_future: Some(Box::new(before_future)),
            framebuffer,
            num_pass: 0,
            command_buffer_builder: Some(command_buffer_builder),
        };

        render(DrawPass { frame: &mut frame });

        frame
            .command_buffer_builder
            .as_mut()
            .unwrap()
            .end_render_pass()
            .unwrap();
        let command_buffer = frame
            .command_buffer_builder
            .take()
            .unwrap()
            .build()
            .unwrap();
        let after_main_cb = frame
            .before_main_cb_future
            .take()
            .unwrap()
            .then_execute(frame.system.gfx_queue.clone(), command_buffer)
            .unwrap();

        Box::new(after_main_cb)
    }
}

pub struct Frame<'a> {
    system: &'a mut FrameSystem,
    num_pass: u8,
    before_main_cb_future: Option<Box<dyn GpuFuture>>,
    framebuffer: Arc<Framebuffer>,
    command_buffer_builder: Option<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>>,
}

impl<'a> Frame<'a> {
}

pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {
    #[inline]
    pub fn execute<C>(&mut self, command_buffer: C)
    where
        C: SecondaryCommandBuffer + Send + Sync + 'static,
    {
        self.frame
            .command_buffer_builder
            .as_mut()
            .unwrap()
            .execute_commands(command_buffer)
            .unwrap();
    }

    #[allow(dead_code)]
    #[inline]
    pub fn viewport_dimensions(&self) -> [u32; 2] {
        self.frame.framebuffer.extent()
    }
}
