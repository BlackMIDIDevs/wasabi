use std::sync::Arc;

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

const NOTE_BUFFER_SIZE: u64 = 250000;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct NoteVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
vulkano::impl_vertex!(NoteVertex, position, color);

struct BufferSet {
    vertex_buffers: Vec<Arc<CpuAccessibleBuffer<[NoteVertex]>>>,
    index: usize,
}

fn get_buffer(device: &Arc<Device>) -> Arc<CpuAccessibleBuffer<[NoteVertex]>> {
    unsafe {
        CpuAccessibleBuffer::uninitialized_array(
            device.clone(),
            NOTE_BUFFER_SIZE,
            BufferUsage::all(),
            false,
        )
        .expect("failed to create buffer")
    }
}

impl BufferSet {
    fn new(device: &Arc<Device>) -> Self {
        Self {
            vertex_buffers: vec![get_buffer(device), get_buffer(device)],
            index: 0,
        }
    }

    fn next(&mut self) -> &Arc<CpuAccessibleBuffer<[NoteVertex]>> {
        self.index = (self.index + 1) % self.vertex_buffers.len();
        &self.vertex_buffers[self.index]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotePassStatus {
    Finished { remaining: u32 },
    HasMoreNotes,
}

pub struct NoteRenderPass {
    gfx_queue: Arc<Queue>,
    buffer_set: BufferSet,
    pipeline: Arc<GraphicsPipeline>,
    render_pass: Arc<RenderPass>,
    depth_buffer: Arc<ImageView<AttachmentImage>>,
}

impl NoteRenderPass {
    pub fn new(renderer: &GuiRenderer) -> NoteRenderPass {
        let gfx_queue = renderer.queue.clone();

        let render_pass = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: renderer.format,
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

        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let gs = gs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        let pipeline = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .vertex_input_state(BuffersDefinition::new().vertex::<NoteVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .geometry_shader(gs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        NoteRenderPass {
            gfx_queue,
            buffer_set: BufferSet::new(&renderer.device),
            pipeline,
            render_pass,
            depth_buffer,
        }
    }

    pub fn draw(
        &mut self,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
        mut fill_buffer: impl FnMut(&Arc<CpuAccessibleBuffer<[NoteVertex]>>) -> NotePassStatus,
    ) {
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

        let mut prev_future: Option<FenceSignalFuture<Box<dyn GpuFuture>>> = None;

        let mut status = NotePassStatus::HasMoreNotes;

        while status == NotePassStatus::HasMoreNotes {
            let buffer = self.buffer_set.next();

            status = fill_buffer(buffer);

            let items_to_render = match status {
                NotePassStatus::Finished { remaining } => {
                    assert!(remaining <= buffer.len() as u32);
                    remaining
                }
                NotePassStatus::HasMoreNotes => buffer.len() as u32,
            };

            let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                self.gfx_queue.device().clone(),
                self.gfx_queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();
            command_buffer_builder
                .begin_render_pass(
                    framebuffer.clone(),
                    SubpassContents::Inline,
                    vec![[0.0, 0.0, 0.0, 0.0].into(), 1.0f32.into()],
                )
                .unwrap();

            command_buffer_builder
                .bind_pipeline_graphics(self.pipeline.clone())
                .set_viewport(
                    0,
                    [Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [img_dims[0] as f32, img_dims[1] as f32],
                        depth_range: 0.0..1.0,
                    }],
                )
                .bind_vertex_buffers(0, buffer.clone())
                .draw(items_to_render, 1, 0, 0)
                .unwrap();

            command_buffer_builder.end_render_pass().unwrap();
            let command_buffer = command_buffer_builder.build().unwrap();

            if let Some(prev_future) = prev_future.take() {
                match prev_future.wait(None) {
                    Ok(x) => x,
                    Err(err) => println!("err: {:?}", err),
                }
            }

            let future = sync::now(self.gfx_queue.device().clone()).boxed();
            let after_main_cb = future
                .then_execute(self.gfx_queue.clone(), command_buffer)
                .unwrap();

            let future = after_main_cb
                .boxed()
                .then_signal_fence_and_flush()
                .expect("Failed to signal fence and flush");

            prev_future = Some(future);
        }

        if let Some(prev_future) = prev_future {
            match prev_future.wait(None) {
                Ok(x) => x,
                Err(err) => println!("err: {:?}", err),
            }
        }
    }
}

mod gs {
    vulkano_shaders::shader! {
        ty: "geometry",
        path: "shaders/notes.geom"
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec2 v_position;
layout(location = 1) out vec4 v_color;

void main() {
    v_position = position;
    v_color = color;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
#version 450
layout(location = 0) in vec4 v_color;

layout(location = 0) out vec4 f_color;

void main() {
    f_color = v_color;
}"
    }
}
