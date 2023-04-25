use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageViewAbstract},
    memory::allocator::StandardMemoryAllocator,
    pipeline::{
        graphics::{
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline, Pipeline, PipelineBindPoint,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    sync::{self, FenceSignalFuture, GpuFuture},
};

use crate::{
    gui::{
        window::keyboard_layout::{KeyPosition, KeyboardView},
        GuiRenderer,
    },
    midi::{self, CakeBlock, CakeMIDIFile, IntVector4, MIDIColor, MIDIFile, MIDIFileBase},
};

use super::RenderResultData;

struct CakeBuffer {
    verts: Arc<CpuAccessibleBuffer<[CakeVertex; 4]>>,
    index: Arc<CpuAccessibleBuffer<[u32]>>,
    data: Arc<CpuAccessibleBuffer<[IntVector4]>>,
}

struct BufferSet {
    buffers: Vec<CakeBuffer>,
}

const BUFFER_USAGE: BufferUsage = BufferUsage {
    transfer_src: true,
    transfer_dst: true,
    uniform_texel_buffer: true,
    storage_texel_buffer: true,
    uniform_buffer: true,
    storage_buffer: true,
    index_buffer: true,
    vertex_buffer: true,
    indirect_buffer: true,
    shader_device_address: true,
    ..BufferUsage::empty()
};

impl BufferSet {
    fn new(device: &Arc<Device>) -> Self {
        Self { buffers: vec![] }
    }

    fn add_buffer(
        &mut self,
        allocator: &StandardMemoryAllocator,
        block: &CakeBlock,
        key: &KeyPosition,
    ) {
        let buffer_data = [
            CakeVertex {
                uv: [1.0, 0.0],
                left_right: [key.left, key.right],
                start: block.start_time as i32,
                end: block.end_time as i32,
                x: key.right,
            },
            CakeVertex {
                uv: [0.0, 0.0],
                left_right: [key.left, key.right],
                start: block.start_time as i32,
                end: block.end_time as i32,
                x: key.left,
            },
            CakeVertex {
                uv: [1.0, 1.0],
                left_right: [key.left, key.right],
                start: block.start_time as i32,
                end: block.end_time as i32,
                x: key.left,
            },
            CakeVertex {
                uv: [0.0, 1.0],
                left_right: [key.left, key.right],
                start: block.start_time as i32,
                end: block.end_time as i32,
                x: key.right,
            },
        ];

        let verts =
            CpuAccessibleBuffer::from_data(allocator, BUFFER_USAGE, false, buffer_data).unwrap();

        let index =
            CpuAccessibleBuffer::from_iter(allocator, BUFFER_USAGE, false, [0, 1, 2, 0, 2, 3])
                .unwrap();

        dbg!(block.tree.len());

        let data = CpuAccessibleBuffer::from_iter(
            allocator,
            BUFFER_USAGE,
            false,
            block.tree.iter().copied(),
        )
        .unwrap();

        let buffer = CakeBuffer { verts, data, index };

        self.buffers.push(buffer);
    }
}

pub struct CakeRenderer {
    gfx_queue: Arc<Queue>,
    buffers: BufferSet,
    pipeline_clear: Arc<GraphicsPipeline>,
    pipeline_draw_over: Arc<GraphicsPipeline>,
    render_pass_clear: Arc<RenderPass>,
    render_pass_draw_over: Arc<RenderPass>,
    allocator: StandardMemoryAllocator,
    cb_allocator: StandardCommandBufferAllocator,
    sd_allocator: StandardDescriptorSetAllocator,
}

#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
struct CakeVertex {
    uv: [f32; 2],
    left_right: [f32; 2],
    start: i32,
    end: i32,
    x: f32,
}
vulkano::impl_vertex!(CakeVertex, uv, left_right, start, end, x);

impl CakeRenderer {
    pub fn new(renderer: &GuiRenderer) -> CakeRenderer {
        let allocator = StandardMemoryAllocator::new_default(renderer.device.clone());

        let gfx_queue = renderer.queue.clone();

        let render_pass_clear = vulkano::ordered_passes_renderpass!(gfx_queue.device().clone(),
            attachments: {
                final_color: {
                    load: Clear,
                    store: Store,
                    format: renderer.format,
                    samples: 1,
                }
            },
            passes: [
                {
                    color: [final_color],
                    depth_stencil: {},
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
                }
            },
            passes: [
                {
                    color: [final_color],
                    depth_stencil: {},
                    input: []
                }
            ]
        )
        .unwrap();

        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        let pipeline_base = GraphicsPipeline::start()
            .input_assembly_state(
                InputAssemblyState::new().topology(PrimitiveTopology::TriangleList),
            )
            .vertex_input_state(BuffersDefinition::new().vertex::<CakeVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant());

        let pipeline_clear = pipeline_base
            .clone()
            .render_pass(Subpass::from(render_pass_clear.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        let pipeline_draw_over = pipeline_base
            .render_pass(Subpass::from(render_pass_draw_over.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        CakeRenderer {
            gfx_queue,
            buffers: BufferSet::new(&renderer.device),
            pipeline_clear,
            pipeline_draw_over,
            render_pass_clear,
            render_pass_draw_over,
            allocator,
            cb_allocator: StandardCommandBufferAllocator::new(
                renderer.device.clone(),
                Default::default(),
            ),
            sd_allocator: StandardDescriptorSetAllocator::new(renderer.device.clone()),
        }
    }

    pub fn draw(
        &mut self,
        key_view: &KeyboardView,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
        midi_file: &mut CakeMIDIFile,
        view_range: f64,
    ) -> RenderResultData {
        let img_dims = final_image.image().dimensions().width_height();

        let mut prev_future = sync::now(self.gfx_queue.device().clone()).boxed();

        if self.buffers.buffers.is_empty() {
            for (i, block) in midi_file.key_blocks().iter().enumerate() {
                if block.tree.len() > 0 {
                    let key = key_view.key(i);
                    self.buffers.add_buffer(&self.allocator, block, &key);
                }
            }
        }

        let mut first = true;

        let midi_time = midi_file.current_time().as_secs_f64();
        let screen_start = (midi_time * midi_file.ticks_per_second() as f64) as i32;
        let screen_end = ((midi_time + view_range) * midi_file.ticks_per_second() as f64) as i32;

        let push_constants = vs::ty::PushConstants {
            start_time: screen_start,
            end_time: screen_end,
            screen_width: img_dims[0] as i32,
            screen_height: img_dims[1] as i32,
        };

        for buffer in self.buffers.buffers.iter() {
            let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
                &self.cb_allocator,
                self.gfx_queue.queue_family_index(),
                CommandBufferUsage::OneTimeSubmit,
            )
            .unwrap();

            let (clears, pipeline, render_pass) = if first {
                first = false;
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
                    attachments: vec![final_image.clone()],
                    ..Default::default()
                },
            )
            .unwrap();

            let pipeline_layout = pipeline.layout();

            let desc_layout = pipeline_layout.set_layouts().get(0).unwrap();
            let data_descriptor = PersistentDescriptorSet::new(
                &self.sd_allocator,
                desc_layout.clone(),
                [WriteDescriptorSet::buffer(0, buffer.data.clone())],
            )
            .unwrap();

            command_buffer_builder
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: clears,
                        ..RenderPassBeginInfo::framebuffer(framebuffer)
                    },
                    SubpassContents::Inline,
                )
                .unwrap();

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
                    data_descriptor.clone(),
                )
                .bind_vertex_buffers(0, buffer.verts.clone())
                .bind_index_buffer(buffer.index.clone())
                .draw_indexed(6, 1, 0, 0, 0)
                .unwrap();

            command_buffer_builder.end_render_pass().unwrap();
            let command_buffer = command_buffer_builder.build().unwrap();

            prev_future = prev_future
                .then_execute(self.gfx_queue.clone(), command_buffer)
                .unwrap()
                .boxed();

            // prev_future
            //     .then_signal_fence_and_flush()
            //     .unwrap()
            //     .wait(None)
            //     .unwrap();

            // prev_future = sync::now(self.gfx_queue.device().clone()).boxed();
        }

        prev_future
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let colors = midi_file
            .key_blocks()
            .iter()
            .map(|block| block.get_note_at(screen_start as u32).map(|n| n.color))
            .collect();

        RenderResultData {
            notes_rendered: 0,
            key_colors: colors,
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/cake/cake.vert",
        types_meta: {
            use bytemuck::{Pod, Zeroable};

            #[derive(Clone, Copy, Zeroable, Pod)]
        },
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "shaders/cake/cake.frag"
    }
}
