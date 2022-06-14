// Copyright (c) 2017 The vulkano developers <=== !
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

// Slightly modified version from
// https://github.com/vulkano-rs/vulkano-examples/blob/master/src/bin/deferred/triangle_draw_system.rs
// To simplify this wholesome example :)

use std::{sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SecondaryAutoCommandBuffer},
    device::Queue,
    pipeline::{
        graphics::{
            depth_stencil::DepthStencilState,
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            vertex_input::BuffersDefinition,
            viewport::{Viewport, ViewportState},
        },
        GraphicsPipeline,
    },
    render_pass::Subpass,
};

pub struct ChikaraShaderTest {
    gfx_queue: Arc<Queue>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pipeline: Arc<GraphicsPipeline>,
    start_time: Instant,
}

impl ChikaraShaderTest {
    pub fn new(gfx_queue: Arc<Queue>, subpass: Subpass) -> ChikaraShaderTest {
        let vertex_buffer = {
            CpuAccessibleBuffer::from_iter(
                gfx_queue.device().clone(),
                BufferUsage::all(),
                false,
                [
                    Vertex {
                        position: [-0.5, -0.25],
                        color: [1.0, 0.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.0, 0.5],
                        color: [0.0, 1.0, 0.0, 1.0],
                    },
                    Vertex {
                        position: [0.25, -0.1],
                        color: [0.0, 0.0, 1.0, 1.0],
                    },
                ]
                .iter()
                .cloned(),
            )
            .expect("failed to create buffer")
        };

        let vs = vs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let fs = fs::load(gfx_queue.device().clone()).expect("failed to create shader module");
        let gs = gs::load(gfx_queue.device().clone()).expect("failed to create shader module");

        let pipeline = GraphicsPipeline::start()
            .input_assembly_state(InputAssemblyState::new().topology(PrimitiveTopology::PointList))
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .geometry_shader(gs.entry_point("main").unwrap(), ())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            .render_pass(subpass)
            .build(gfx_queue.device().clone())
            .unwrap();

        ChikaraShaderTest {
            gfx_queue,
            vertex_buffer,
            pipeline,
            start_time: Instant::now(),
        }
    }

    fn iter_verts(&mut self) -> impl Iterator<Item = Vertex> {
        let angle = self.start_time.elapsed().as_secs_f32() as f32 * 10.0;

        [
            Vertex {
                position: [angle.cos() * 0.5, angle.sin() * 0.5],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [(angle + 0.25).cos() * 0.5, (angle + 0.25).sin() * 0.5],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [(angle + 0.5).cos() * 0.5, (angle + 0.5).sin() * 0.5],
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ]
        .into_iter()
    }

    pub fn draw(&mut self, viewport_dimensions: [u32; 2]) -> SecondaryAutoCommandBuffer {
        {
            let new_verts = self.iter_verts().enumerate();
            let mut verts = self.vertex_buffer.write().unwrap();
            for (i, v) in new_verts {
                verts[i] = v;
            }
        }

        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.gfx_queue.device().clone(),
            self.gfx_queue.family(),
            CommandBufferUsage::MultipleSubmit,
            self.pipeline.subpass().clone(),
        )
        .unwrap();
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .set_viewport(
                0,
                [Viewport {
                    origin: [0.0, 0.0],
                    dimensions: [viewport_dimensions[0] as f32, viewport_dimensions[1] as f32],
                    depth_range: 0.0..1.0,
                }],
            )
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap();
        builder.build().unwrap()
    }
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Zeroable, Pod)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, position, color);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct UniformBufferObject {
    time: f32,
    pre_time: f32,
    keyboard_height: f32,
    width: f32,
    height: f32,
}

// mod vs
// {
//   vulkano_shaders::shader! {
//       ty: "vertex",
//       path: "src/shaders/notes.vert"
//   }
// }

// mod fs
// {
//   vulkano_shaders::shader! {
//       ty: "fragment",
//       path: "src/shaders/notes.frag"
//   }
// }

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
