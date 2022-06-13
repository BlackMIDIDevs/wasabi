use std::sync::Arc;

use egui::Ui;
use vulkano::{
    device::Device,
    sync::{self, GpuFuture},
};

use crate::{
    old::{draw_system::ChikaraShaderTest, frame_system::FrameSystem},
    scenes::SceneSwapchain,
};

use super::{GuiRenderer, GuiState};

pub struct GuiRenderScene {
    swap_chain: SceneSwapchain,
    frame_system: FrameSystem,
    draw_system: ChikaraShaderTest,
    device: Arc<Device>,
}

impl GuiRenderScene {
    pub fn new(renderer: &GuiRenderer) -> Self {
        let frame_system = FrameSystem::new(renderer.queue.clone(), renderer.format);
        let draw_system =
            ChikaraShaderTest::new(renderer.queue.clone(), frame_system.deferred_subpass());
        Self {
            swap_chain: SceneSwapchain::new(renderer.device.clone()),
            frame_system,
            draw_system,
            device: renderer.device.clone(),
        }
    }

    pub fn layout(&mut self, state: &mut GuiState, ui: &mut Ui) {
        let size = ui.available_size();
        let size = [size.x as u32, size.y as u32];

        let scene_image = self.swap_chain.get_next_image(state, size);

        let future = sync::now(self.device.clone()).boxed();

        let after_future = self.frame_system.draw_frame(
            future,
            // Notice that final image is now scene image
            scene_image.image.clone(),
            |mut draw_pass| {
                let cb = self.draw_system.draw(size);
                draw_pass.execute(cb);
            },
        );

        // Wait on our future
        let future = after_future
            .then_signal_fence_and_flush()
            .expect("Failed to signal fence and flush");
        match future.wait(None) {
            Ok(x) => x,
            Err(err) => println!("err: {:?}", err),
        }

        ui.image(scene_image.id, [size[0] as f32, size[1] as f32]);
    }
}
