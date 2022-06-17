use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{Device, Queue},
    format::{ClearValue, Format},
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageViewAbstract},
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::{self, FenceSignalFuture, GpuFuture},
};

use crate::gui::{window::keyboard_layout::KeyboardView, GuiRenderer};

const NOTE_BUFFER_SIZE: u64 = 25000000;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct NoteVertex {
    pub start_length: [f32; 2],
    pub key_color: u32,
}
vulkano::impl_vertex!(NoteVertex, start_length, key_color);

impl NoteVertex {
    pub fn new(start: f32, len: f32, key: u8, color: u32) -> Self {
        Self {
            start_length: [start, len],
            key_color: key as u32 | (color << 8),
        }
    }
}

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

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
pub struct KeyPosition {
    left: f32,
    right: f32,
    _padding: [u8; 8],
}

pub struct NoteRenderPass {
    gfx_queue: Arc<Queue>,
    buffer_set: BufferSet,
    pipeline_clear: Arc<GraphicsPipeline>,
    pipeline_draw_over: Arc<GraphicsPipeline>,
    render_pass_clear: Arc<RenderPass>,
    render_pass_draw_over: Arc<RenderPass>,
    key_locations: Arc<CpuAccessibleBuffer<[[KeyPosition; 256]]>>,
    depth_buffer: Arc<ImageView<AttachmentImage>>,
}

impl NoteRenderPass {
    pub fn new(renderer: &GuiRenderer) -> NoteRenderPass {
        let gfx_queue = renderer.queue.clone();

        let render_pass_clear = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: renderer.format,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: Store,
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

        let render_pass_draw_over = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    load: DontCare,
                    store: Store,
                    format: renderer.format,
                    samples: 1,
                },
                depth: {
                    load: DontCare,
                    store: Store,
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

        let key_locations = CpuAccessibleBuffer::from_iter(
            gfx_queue.device().clone(),
            BufferUsage::all(),
            false,
            [[Default::default(); 256]].into_iter(),
        )
        .unwrap();

        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let gs = gs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        let pipeline_clear = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .vertex_input_state(BuffersDefinition::new().vertex::<NoteVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .geometry_shader(gs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(Subpass::from(render_pass_clear.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        let pipeline_draw_over = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .vertex_input_state(BuffersDefinition::new().vertex::<NoteVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .geometry_shader(gs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(Subpass::from(render_pass_draw_over.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        NoteRenderPass {
            gfx_queue,
            buffer_set: BufferSet::new(&renderer.device),
            pipeline_clear,
            pipeline_draw_over,
            render_pass_clear,
            render_pass_draw_over,
            depth_buffer,
            key_locations,
        }
    }

    pub fn draw(
        &mut self,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
        key_view: &KeyboardView,
        view_range: f32,
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

        {
            let mut keys = self.key_locations.write().unwrap();
            for (write, key) in keys[0].iter_mut().zip(key_view.iter_all_notes()) {
                *write = KeyPosition {
                    left: key.left,
                    right: key.right,
                    _padding: [0; 8],
                };
            }
        }

        let mut prev_future: Option<FenceSignalFuture<Box<dyn GpuFuture>>> = None;

        let mut status = NotePassStatus::HasMoreNotes;

        let mut first_pass = true;

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

            let (clears, pipeline, render_pass) = if first_pass {
                first_pass = false;
                (
                    vec![[0.0, 0.0, 0.0, 0.0].into(), 1.0f32.into()],
                    &self.pipeline_clear,
                    &self.render_pass_clear,
                )
            } else {
                (
                    vec![ClearValue::None, ClearValue::None],
                    &self.pipeline_draw_over,
                    &self.render_pass_draw_over,
                )
            };

            let framebuffer = Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![final_image.clone(), self.depth_buffer.clone()],
                    ..Default::default()
                },
            )
            .unwrap();

            let pipeline_layout = pipeline.layout();

            let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                desc_layout.clone(),
                [WriteDescriptorSet::buffer(0, self.key_locations.clone())],
            )
            .unwrap();

            command_buffer_builder
                .begin_render_pass(framebuffer.clone(), SubpassContents::Inline, clears)
                .unwrap();

            let push_constants = gs::ty::PushConstants {
                height_time: view_range,
                win_width: img_dims[0] as f32,
                win_height: img_dims[1] as f32,
            };

            command_buffer_builder
                .bind_pipeline_graphics(pipeline.clone())
                .set_viewport(
                    0,
                    [Viewport {
                        origin: [0.0, 0.0],
                        dimensions: [img_dims[0] as f32, img_dims[1] as f32],
                        depth_range: 0.0..1.0,
                    }],
                )
                .push_constants(pipeline_layout.clone().clone(), 0, push_constants)
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline_layout.clone(),
                    0,
                    set.clone(),
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
layout(location = 0) in vec2 start_length;
layout(location = 1) in uint key_color;

layout(location = 0) out vec2 v_start_length;
layout(location = 1) out uint v_key_color;

void main() {
    v_start_length = start_length;
    v_key_color = key_color;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/notes.frag"
    }
}
