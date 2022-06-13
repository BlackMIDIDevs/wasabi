use std::sync::Arc;

use vulkano::{
    device::{physical::PhysicalDevice, Device},
    format::Format,
    image::{view::ImageView, ImageUsage, SwapchainImage},
    swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError},
};
use winit::window::Window;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ImagesState {
    pub count: usize,
    pub format: Format,
}

pub struct SwapchainState {
    pub size: [u32; 2],
    pub images_state: ImagesState,
}

pub struct ManagedSwapchain {
    state: SwapchainState,
    swap_chain: Arc<Swapchain<Window>>,
    image_views: Vec<Arc<ImageView<SwapchainImage<Window>>>>,
}

impl ManagedSwapchain {
    pub fn create(
        surface: Arc<Surface<Window>>,
        physical: PhysicalDevice,
        device: Arc<Device>,
        present_mode: PresentMode,
    ) -> Self {
        let surface_capabilities = physical
            .surface_capabilities(&surface, Default::default())
            .unwrap();
        let image_format = Some(
            physical
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0,
        );
        let image_extent = surface.window().inner_size().into();

        let (swapchain, images) = Swapchain::new(
            device,
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent,
                image_usage: ImageUsage::color_attachment(),
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .iter()
                    .next()
                    .unwrap(),
                present_mode,
                ..Default::default()
            },
        )
        .unwrap();
        let images = images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect::<Vec<_>>();

        Self {
            state: SwapchainState {
                size: image_extent,
                images_state: ImagesState {
                    count: images.len(),
                    format: image_format.unwrap(),
                },
            },
            swap_chain: swapchain,
            image_views: images,
        }
    }

    pub fn recreate(&mut self, surface: Arc<Surface<Window>>) {
        let dimensions: [u32; 2] = surface.window().inner_size().into();
        let (new_swapchain, new_images) = match self.swap_chain.recreate(SwapchainCreateInfo {
            image_extent: dimensions,
            ..self.swap_chain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
        };
        self.swap_chain = new_swapchain;
        let new_images = new_images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect::<Vec<_>>();

        self.image_views = new_images;
    }
}
