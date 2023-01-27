use std::sync::Arc;

use vulkano::{
    device::Device,
    image::{view::ImageView, AttachmentImage},
    memory::allocator::StandardMemoryAllocator,
};

use crate::{gui::GuiState, renderer::swapchain::ImagesState};

pub struct SceneImage {
    pub image: Arc<ImageView<AttachmentImage>>,
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
                state.gui.unregister_user_image(image.id);
            }

            let allocator = StandardMemoryAllocator::new_default(self.device.clone());

            // Create new images
            for _ in 0..image_state.count {
                let image = ImageView::new_default(
                    AttachmentImage::sampled_input_attachment(&allocator, size, image_state.format)
                        .expect("Failed to create scene image"),
                )
                .expect("Failed to create scene image view");

                let id = state.gui.register_user_image_view(image.clone());

                self.scene_images.push(SceneImage { image, id });
            }

            self.image_state = Some(image_state);
            self.scene_view_size = size;
        }

        &self.scene_images[state.frame.image_num as usize]
    }
}
