use std::sync::Arc;

use vulkano::{
    device::Device,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    memory::allocator::StandardMemoryAllocator,
};

use crate::{gui::GuiState, renderer::swapchain::ImagesState};

pub struct SceneImage {
    pub image: Arc<ImageView>,
    pub id: egui::TextureId,
}

pub struct SceneSwapchain {
    device: Arc<Device>,
    scene_images: Vec<SceneImage>,
    image_state: Option<ImagesState>,
    scene_view_size: [u32; 2],
}

impl SceneSwapchain {
    pub fn new(device: Arc<Device>) -> Self {
        Self {
            device,
            scene_images: Vec::new(),
            image_state: None,
            scene_view_size: [0, 0],
        }
    }

    pub fn get_next_image(&mut self, state: &mut GuiState, size: [u32; 2]) -> &SceneImage {
        let image_state = state.frame.swap_chain_state().images_state;

        if Some(image_state) != self.image_state || self.scene_view_size != size {
            // Remove existing images
            for image in self.scene_images.drain(..) {
                state.renderer.gui.unregister_user_image(image.id);
            }

            let allocator = Arc::new(StandardMemoryAllocator::new_default(self.device.clone()));
            let mut create_info: ImageCreateInfo = Default::default();
            create_info.format = image_state.format;
            create_info.extent = [size[0], size[1], 1];
            create_info.usage = ImageUsage::SAMPLED | ImageUsage::COLOR_ATTACHMENT;

            // Create new images
            for _ in 0..image_state.count {
                let image = ImageView::new_default(
                    Image::new(allocator.clone(), create_info.clone(), Default::default())
                        .expect("Failed to create scene image"),
                )
                .expect("Failed to create scene image view");

                let id = state
                    .renderer
                    .gui
                    .register_user_image_view(image.clone(), Default::default());

                self.scene_images.push(SceneImage { image, id });
            }

            self.image_state = Some(image_state);
            self.scene_view_size = size;
        }

        &self.scene_images[state.frame.image_num as usize]
    }
}
