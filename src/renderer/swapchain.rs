use std::sync::Arc;

use vulkano::{
    device::{physical::PhysicalDevice, Device, Queue},
    format::Format,
    image::{view::ImageView, ImageUsage, SwapchainImage},
    swapchain::{
        AcquireError, PresentMode, Surface, Swapchain, SwapchainAcquireFuture, SwapchainCreateInfo,
        SwapchainCreationError, SwapchainPresentInfo,
    },
    sync::{self, FlushError, GpuFuture},
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    swap_chain: Arc<Swapchain>,
    image_views: Vec<Arc<ImageView<SwapchainImage>>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    device: Arc<Device>,
    recreate_on_next_frame: bool,
}

impl ManagedSwapchain {
    pub fn create(
        surface: Arc<Surface>,
        window: Arc<Window>,
        physical: Arc<PhysicalDevice>,
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
        let image_extent = window.inner_size().into();

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count,
                image_format,
                image_extent,
                image_usage: ImageUsage {
                    color_attachment: true,
                    ..ImageUsage::empty()
                },
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
            previous_frame_end: Some(sync::now(device.clone()).boxed()),
            device,
            recreate_on_next_frame: false,
        }
    }

    pub fn state(&self) -> &SwapchainState {
        &self.state
    }

    pub fn resize(&mut self, size: Option<PhysicalSize<u32>>) {
        self.recreate_on_next_frame = true;
        if let Some(s) = size {
            self.state.size = s.into();
        }
    }

    pub fn recreate(&mut self) {
        let (new_swapchain, new_images) = match self.swap_chain.recreate(SwapchainCreateInfo {
            image_extent: self.state.size,
            ..self.swap_chain.create_info()
        }) {
            Ok(r) => r,
            Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
            Err(e) => panic!("Failed to recreate swapchain: {e:?}"),
        };
        self.swap_chain = new_swapchain;
        let new_images = new_images
            .into_iter()
            .map(|image| ImageView::new_default(image).unwrap())
            .collect::<Vec<_>>();

        self.image_views = new_images;
    }

    pub fn take_previous_frame_end(&mut self) -> Option<Box<dyn GpuFuture>> {
        self.previous_frame_end.take()
    }

    pub fn acquire_frame(&mut self) -> (SwapchainFrame, SwapchainAcquireFuture) {
        if self.recreate_on_next_frame {
            self.recreate();
            self.recreate_on_next_frame = false;
        }

        let mut tries = 0;
        loop {
            tries += 1;
            if tries > 10 {
                panic!("Failed to acquire next image after 10 tries");
            }

            let next = vulkano::swapchain::acquire_next_image(self.swap_chain.clone(), None);

            let (image_num, suboptimal, acquire_future) = match next {
                Ok(r) => r,
                // TODO: Handle more errors, e.g. DeviceLost, by re-creating the entire graphics chain
                Err(AcquireError::OutOfDate) => {
                    self.recreate();
                    continue;
                }
                Err(e) => panic!("Failed to acquire next image: {e:?}"),
            };

            if suboptimal {
                self.recreate();
                continue;
            }

            let frame = SwapchainFrame {
                presented: false,
                image_num,
                image: self.image_views[image_num as usize].clone(),
                managed_swap_chain: self,
            };

            return (frame, acquire_future);
        }
    }
}

pub struct SwapchainFrame<'a> {
    presented: bool,

    pub image_num: u32,
    pub image: Arc<ImageView<SwapchainImage>>,

    managed_swap_chain: &'a mut ManagedSwapchain,
}

impl<'a> SwapchainFrame<'a> {
    pub fn present(mut self, queue: &Arc<Queue>, after_future: Box<dyn GpuFuture>) {
        self.presented = true;

        let sc = &mut self.managed_swap_chain;

        let present_info =
            SwapchainPresentInfo::swapchain_image_index(sc.swap_chain.clone(), self.image_num);

        let future = after_future
            .then_swapchain_present(queue.clone(), present_info)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // FIXME: A hack to prevent OutOfMemory error on Nvidia
                // https://github.com/vulkano-rs/vulkano/issues/627
                match future.wait(None) {
                    Ok(x) => x,
                    Err(err) => println!("err: {err:?}"),
                }
                sc.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                sc.recreate_on_next_frame = true;
                sc.previous_frame_end = Some(sync::now(sc.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {e:?}");
                sc.previous_frame_end = Some(sync::now(sc.device.clone()).boxed());
            }
        }
    }

    pub fn swap_chain_state(&self) -> &SwapchainState {
        &self.managed_swap_chain.state
    }
}

impl<'a> std::ops::Drop for SwapchainFrame<'a> {
    fn drop(&mut self) {
        if !self.presented {
            panic!("SwapchainFrame not presented.")
        }
    }
}
