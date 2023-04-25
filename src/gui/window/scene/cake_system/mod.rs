use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        RenderPassBeginInfo, SubpassContents,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{Device, Queue},
    format::Format,
    image::{view::ImageView, AttachmentImage, ImageAccess, ImageViewAbstract},
    memory::allocator::StandardMemoryAllocator,
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
    sync::{self, GpuFuture},
};

use crate::{
    gui::{
        window::keyboard_layout::{KeyPosition, KeyboardView},
        GuiRenderer,
    },
    midi::{CakeBlock, CakeMIDIFile, CakeSignature, IntVector4},
};

use super::RenderResultData;

const BUFFER_ARRAY_LEN: u64 = 256;

struct CakeBuffer {
    data: Arc<CpuAccessibleBuffer<[IntVector4]>>,
    start: i32,
    end: i32,
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

#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C)]
struct CakeVertex {
    left: f32,
    right: f32,
    start: i32,
    end: i32,
    buffer_index: i32,
}
vulkano::impl_vertex!(CakeVertex, left, right, start, end, buffer_index);

impl BufferSet {
    fn new(_device: &Arc<Device>) -> Self {
        Self { buffers: vec![] }
    }

    fn add_buffer(
        &mut self,
        allocator: &StandardMemoryAllocator,
        block: &CakeBlock,
        _key: &KeyPosition,
    ) {
        let data = CpuAccessibleBuffer::from_iter(
            allocator,
            BUFFER_USAGE,
            false,
            block.tree.iter().copied(),
        )
        .unwrap();

        let buffer = CakeBuffer {
            data,
            start: block.start_time as i32,
            end: block.end_time as i32,
        };

        self.buffers.push(buffer);
    }

    fn clear(&mut self) {
        self.buffers.clear();
    }
}

pub struct CakeRenderer {
    gfx_queue: Arc<Queue>,
    buffers: BufferSet,
    pipeline_clear: Arc<GraphicsPipeline>,
    render_pass_clear: Arc<RenderPass>,
    allocator: StandardMemoryAllocator,
    depth_buffer: Arc<ImageView<AttachmentImage>>,
    cb_allocator: StandardCommandBufferAllocator,
    sd_allocator: StandardDescriptorSetAllocator,
    buffers_init: Arc<CpuAccessibleBuffer<[CakeVertex]>>,
    current_file_signature: Option<CakeSignature>,
}

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

        let depth_buffer = ImageView::new_default(
            AttachmentImage::transient_input_attachment(&allocator, [1, 1], Format::D16_UNORM)
                .unwrap(),
        )
        .unwrap();

        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let gs = gs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        let pipeline_base = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .vertex_input_state(BuffersDefinition::new().vertex::<CakeVertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .geometry_shader(gs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .depth_stencil_state(DepthStencilState::simple_depth_test());

        let pipeline_clear = pipeline_base
            .clone()
            .render_pass(Subpass::from(render_pass_clear.clone(), 0).unwrap())
            .build(gfx_queue.device().clone())
            .unwrap();

        let buffers = unsafe {
            CpuAccessibleBuffer::uninitialized_array(
                &allocator,
                BUFFER_ARRAY_LEN,
                BUFFER_USAGE,
                false,
            )
            .unwrap()
        };

        CakeRenderer {
            gfx_queue,
            buffers: BufferSet::new(&renderer.device),
            pipeline_clear,
            render_pass_clear,
            depth_buffer,
            allocator,
            cb_allocator: StandardCommandBufferAllocator::new(
                renderer.device.clone(),
                Default::default(),
            ),
            sd_allocator: StandardDescriptorSetAllocator::new(renderer.device.clone()),
            buffers_init: buffers,
            current_file_signature: None,
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
        if self.depth_buffer.image().dimensions().width_height() != img_dims {
            self.depth_buffer = ImageView::new_default(
                AttachmentImage::transient_input_attachment(
                    &self.allocator,
                    img_dims,
                    Format::D16_UNORM,
                )
                .unwrap(),
            )
            .unwrap();
        }

        let curr_signature = midi_file.cake_signature();
        if self.current_file_signature.as_ref() != Some(&curr_signature) {
            self.current_file_signature = Some(curr_signature);
            self.buffers.clear();
            for (i, block) in midi_file.key_blocks().iter().enumerate() {
                let key = key_view.key(i);
                self.buffers.add_buffer(&self.allocator, block, &key);
            }
        }

        let midi_time = midi_file.current_time().as_secs_f64();
        let screen_start = (midi_time * midi_file.ticks_per_second() as f64) as i32;
        let screen_end = ((midi_time + view_range) * midi_file.ticks_per_second() as f64) as i32;

        let push_constants = gs::ty::PushConstants {
            start_time: screen_start,
            end_time: screen_end,
            screen_width: img_dims[0] as i32,
            screen_height: img_dims[1] as i32,
        };

        let mut buffer_instances = self.buffers_init.write().unwrap();
        // Black keys first
        let mut written_instances = 0;
        for (i, buffer) in self.buffers.buffers.iter().enumerate() {
            let key = key_view.note(i);
            if key.black {
                buffer_instances[written_instances] = CakeVertex {
                    buffer_index: i as i32,
                    start: buffer.start,
                    end: buffer.end,
                    left: key.left,
                    right: key.right,
                };
                written_instances += 1;
            }
        }
        // Then white keys
        for (i, buffer) in self.buffers.buffers.iter().enumerate() {
            let key = key_view.note(i);
            if !key.black {
                buffer_instances[written_instances] = CakeVertex {
                    buffer_index: i as i32,
                    start: buffer.start,
                    end: buffer.end,
                    left: key.left,
                    right: key.right,
                };
                written_instances += 1;
            }
        }
        drop(buffer_instances);

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary(
            &self.cb_allocator,
            self.gfx_queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let (clears, pipeline, render_pass) = (
            vec![Some([0.0, 0.0, 0.0, 0.0].into()), Some(1.0f32.into())],
            &self.pipeline_clear,
            &self.render_pass_clear,
        );

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
        let data_descriptor = PersistentDescriptorSet::new(
            &self.sd_allocator,
            desc_layout.clone(),
            [WriteDescriptorSet::buffer_array(
                0,
                0,
                self.buffers
                    .buffers
                    .iter()
                    .map(|b| b.data.clone() as Arc<dyn BufferAccess>),
            )],
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
            .push_constants(pipeline_layout.clone(), 0, push_constants)
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                pipeline_layout.clone(),
                0,
                data_descriptor,
            )
            .bind_vertex_buffers(0, self.buffers_init.clone())
            .draw(written_instances as u32, 1, 0, 0)
            .unwrap();

        command_buffer_builder.end_render_pass().unwrap();
        let command_buffer = command_buffer_builder.build().unwrap();

        let now = sync::now(self.gfx_queue.device().clone()).boxed();
        let render_future = now
            .then_execute(self.gfx_queue.clone(), command_buffer)
            .unwrap()
            .boxed();

        // Calculate the metadata before awaiting the future
        // to keep this more efficient
        let colors = midi_file
            .key_blocks()
            .iter()
            .map(|block| block.get_note_at(screen_start as u32).map(|n| n.color))
            .collect();
        let rendered_notes = midi_file
            .key_blocks()
            .iter()
            .map(|block| {
                let passed = block.get_notes_passed_at(screen_end as u32)
                    - block.get_notes_passed_at(screen_start as u32);

                if block.get_note_at(screen_start as u32).is_some() {
                    passed as u64 + 1
                } else {
                    passed as u64
                }
            })
            .sum();

        render_future
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        RenderResultData {
            notes_rendered: rendered_notes,
            key_colors: colors,
        }
    }
}

mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "shaders/cake/cake.vert"
    }
}

mod gs {
    vulkano_shaders::shader! {
        ty: "geometry",
        path: "shaders/cake/cake.geom",
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
