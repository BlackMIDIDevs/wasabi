use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::DepthStencilState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            subpass::PipelineSubpassType,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::{self, future::FenceSignalFuture, GpuFuture},
};

use crate::gui::{window::keyboard_layout::KeyboardView, GuiRenderer};

const NOTE_BUFFER_SIZE: u64 = 25000000;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod, Vertex)]
pub struct NoteVertex {
    #[format(R32G32_SFLOAT)]
    pub start_length: [f32; 2],
    #[format(R32_UINT)]
    pub key_color: u32,
    #[format(R32_UINT)]
    pub border_width: u32,
}

impl NoteVertex {
    pub fn new(start: f32, len: f32, key: u8, color: u32, border_width: u32) -> Self {
        Self {
            start_length: [start, len],
            key_color: key as u32 | (color << 8),
            border_width,
        }
    }
}

struct BufferSet {
    vertex_buffers: [Subbuffer<[NoteVertex]>; 2],
    index: usize,
}

fn get_buffer(device: &Arc<Device>) -> (Subbuffer<[NoteVertex]>, Subbuffer<[NoteVertex]>) {
    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    Buffer::new_slice(
        allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        NOTE_BUFFER_SIZE * 2,
    )
    .expect("failed to create buffer")
    .split_at(NOTE_BUFFER_SIZE)
}

impl BufferSet {
    fn new(device: &Arc<Device>) -> Self {
        let buffer = get_buffer(device);
        Self {
            vertex_buffers: [buffer.0, buffer.1],
            index: 0,
        }
    }

    fn next(&mut self) -> &Subbuffer<[NoteVertex]> {
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
    key_locations: Subbuffer<[[KeyPosition; 256]]>,
    depth_buffer: Arc<ImageView>,
    allocator: Arc<StandardMemoryAllocator>,
    cb_allocator: StandardCommandBufferAllocator,
    sd_allocator: StandardDescriptorSetAllocator,
}

impl NoteRenderPass {
    pub fn new(renderer: &GuiRenderer) -> NoteRenderPass {
        let allocator = Arc::new(StandardMemoryAllocator::new_default(
            renderer.device.clone(),
        ));

        let gfx_queue = renderer.queue.clone();

        let render_pass_clear = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    format: renderer.format,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
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
                    format: renderer.format,
                    samples: 1,
                    load_op: DontCare,
                    store_op: Store,
                },
                depth: {
                    format: Format::D16_UNORM,
                    samples: 1,
                    load_op: DontCare,
                    store_op: Store,
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
            Image::new(
                allocator.clone(),
                ImageCreateInfo {
                    extent: [1, 1, 1],
                    format: Format::D16_UNORM,
                    usage: ImageUsage::SAMPLED | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                    ..Default::default()
                },
                Default::default(),
            )
            .unwrap(),
        )
        .unwrap();

        let key_locations = Buffer::from_iter(
            allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            [[Default::default(); 256]],
        )
        .unwrap();

        let vs = vs::load(gfx_queue.device().clone())
            .expect("failed to create shader module")
            .entry_point("main")
            .unwrap();
        let fs = fs::load(gfx_queue.device().clone())
            .expect("failed to create shader module")
            .entry_point("main")
            .unwrap();
        let gs = gs::load(gfx_queue.device().clone())
            .expect("failed to create shader module")
            .entry_point("main")
            .unwrap();

        let vertex_input_state = NoteVertex::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
            PipelineShaderStageCreateInfo::new(gs),
        ];
        let layout = PipelineLayout::new(
            renderer.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(renderer.device.clone())
                .unwrap(),
        )
        .unwrap();
        let subpass = Subpass::from(render_pass_clear.clone(), 0).unwrap();

        let mut create_info = GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::PointList,
                ..Default::default()
            }),
            viewport_state: Some(ViewportState {
                viewports: [Viewport {
                    offset: [0.0, 0.0],
                    extent: [1280.0, 720.0],
                    depth_range: 0.0..=1.0,
                }]
                .into_iter()
                .collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            depth_stencil_state: Some(DepthStencilState::default()),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        let pipeline_clear =
            GraphicsPipeline::new(renderer.device.clone(), None, create_info.clone()).unwrap();

        create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(
            Subpass::from(render_pass_draw_over.clone(), 0).unwrap(),
        ));
        let pipeline_draw_over =
            GraphicsPipeline::new(renderer.device.clone(), None, create_info).unwrap();

        NoteRenderPass {
            gfx_queue,
            buffer_set: BufferSet::new(&renderer.device),
            pipeline_clear,
            pipeline_draw_over,
            render_pass_clear,
            render_pass_draw_over,
            depth_buffer,
            key_locations,
            allocator,
            cb_allocator: StandardCommandBufferAllocator::new(
                renderer.device.clone(),
                Default::default(),
            ),
            sd_allocator: StandardDescriptorSetAllocator::new(
                renderer.device.clone(),
                Default::default(),
            ),
        }
    }

    pub fn draw(
        &mut self,
        final_image: Arc<ImageView>,
        key_view: &KeyboardView,
        view_range: f32,
        mut fill_buffer: impl FnMut(&Subbuffer<[NoteVertex]>) -> NotePassStatus,
    ) {
        let img_dims = final_image.image().extent();
        if self.depth_buffer.image().extent() != img_dims {
            self.depth_buffer = ImageView::new_default(
                Image::new(
                    self.allocator.clone(),
                    ImageCreateInfo {
                        extent: [img_dims[0], img_dims[1], 1],
                        format: Format::D16_UNORM,
                        usage: ImageUsage::SAMPLED | ImageUsage::DEPTH_STENCIL_ATTACHMENT,
                        ..Default::default()
                    },
                    Default::default(),
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
                &self.cb_allocator,
                self.gfx_queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            let (clears, pipeline, render_pass) = if first_pass {
                first_pass = false;
                (
                    vec![Some([0.0, 0.0, 0.0, 0.0].into()), Some(1.0f32.into())],
                    &self.pipeline_clear,
                    &self.render_pass_clear,
                )
            } else {
                (
                    vec![None, None],
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

            let desc_layout = pipeline_layout.set_layouts().first().unwrap();
            let write_descriptor_set = WriteDescriptorSet::buffer(0, self.key_locations.clone());
            let set = PersistentDescriptorSet::new(
                &self.sd_allocator,
                desc_layout.clone(),
                [write_descriptor_set],
                [],
            )
            .unwrap();

            let mut subpassbegininfo = SubpassBeginInfo::default();
            subpassbegininfo.contents = SubpassContents::Inline;

            command_buffer_builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: clears,
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    subpassbegininfo,
                )
                .unwrap();

            let push_constants = gs::PushConstants {
                height_time: view_range,
                win_width: img_dims[0] as f32,
                win_height: img_dims[1] as f32,
            };

            command_buffer_builder
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .push_constants(pipeline_layout.clone().clone(), 0, push_constants)
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline_layout.clone(),
                    0,
                    set.clone(),
                )
                .unwrap()
                .bind_vertex_buffers(0, buffer.clone())
                .unwrap()
                .draw(items_to_render, 1, 0, 0)
                .unwrap();

            command_buffer_builder
                .end_render_pass(Default::default())
                .unwrap();
            let command_buffer = command_buffer_builder.build().unwrap();

            if let Some(prev_future) = prev_future.take() {
                match prev_future.wait(None) {
                    Ok(x) => x,
                    Err(err) => println!("err: {err:?}"),
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
                Err(err) => println!("err: {err:?}"),
            }
        }
    }
}

mod gs {
    vulkano_shaders::shader! {
        ty: "geometry",
        path: "shaders/notes/notes.geom",
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
#version 450
layout(location = 0) in vec2 start_length;
layout(location = 1) in uint key_color;
layout(location = 2) in uint border_width;

layout(location = 0) out vec2 v_start_length;
layout(location = 1) out uint v_key_color;
layout(location = 2) out uint v_border_width;

void main() {
    v_start_length = start_length;
    v_key_color = key_color;
    v_border_width = border_width;
}"
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/notes/notes.frag"
    }
}
