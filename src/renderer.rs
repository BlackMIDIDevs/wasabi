mod swapchain;

use std::sync::Arc;

use egui_winit_vulkano::Gui;
use vulkano::{
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo,
    },
    image::{view::ImageView, AttachmentImage, ImageUsage, ImageViewAbstract, SwapchainImage},
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::{
        AcquireError, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainCreationError,
    },
    sync,
    sync::{FlushError, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use self::swapchain::{ManagedSwapchain, SwapchainState};

pub struct Renderer {
    instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,
    swap_chain: ManagedSwapchain,
    image_num: usize,
}

pub struct RenderState<'a> {
    swapchain: SwapchainState,
    selected_image: usize,
    gui: &'a mut Gui,
}

impl Renderer {
    pub fn new(
        event_loop: &EventLoop<()>,
        window_size: [u32; 2],
        present_mode: PresentMode,
        name: &str,
    ) -> Self {
        // Add instance extensions based on needs
        let instance_extensions = InstanceExtensions {
            ..vulkano_win::required_extensions()
        };
        // Create instance
        let instance = Instance::new(InstanceCreateInfo {
            application_version: Version::V1_2,
            enabled_extensions: instance_extensions,
            ..Default::default()
        })
        .expect("Failed to create instance");
        // Get most performant device (physical)
        let physical = PhysicalDevice::enumerate(&instance)
            .fold(None, |acc, val| {
                if acc.is_none() {
                    Some(val)
                } else if acc.unwrap().properties().max_compute_shared_memory_size
                    >= val.properties().max_compute_shared_memory_size
                {
                    acc
                } else {
                    Some(val)
                }
            })
            .expect("No physical device found");
        println!("Using device {}", physical.properties().device_name);
        // Create rendering surface along with window
        let surface = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(window_size[0], window_size[1]))
            .with_title(name)
            .build_vk_surface(event_loop, instance.clone())
            .expect("Failed to create vulkan surface & window");
        // Create device
        let (device, queue) = Self::create_device(physical, surface.clone());
        // Create swap chain & frame(s) to which we'll render

        let swap_chain =
            ManagedSwapchain::create(surface.clone(), physical, device.clone(), present_mode);

        Self {
            instance,
            device,
            surface,
            queue,
            swap_chain,
            image_num: 0,
        }
    }

    /// Creates vulkan device with required queue families and required extensions
    fn create_device(
        physical: PhysicalDevice,
        surface: Arc<Surface<Window>>,
    ) -> (Arc<Device>, Arc<Queue>) {
        let queue_family = physical
            .queue_families()
            .find(|&q| q.supports_graphics() && q.supports_surface(&surface).unwrap_or(false))
            .expect("couldn't find a graphical queue family");
        // Add device extensions based on needs
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        // Add device features
        let features = Features {
            geometry_shader: true,
            ..Features::none()
        };
        let (device, mut queues) = {
            Device::new(
                physical,
                DeviceCreateInfo {
                    enabled_extensions: physical.required_extensions().union(&device_extensions),
                    enabled_features: features,
                    queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                    _ne: Default::default(),
                },
            )
            .expect("failed to create device")
        };
        (device, queues.next().unwrap())
    }

    fn create_swap_chain(
        surface: Arc<Surface<Window>>,
        physical: PhysicalDevice,
        device: Arc<Device>,
        present_mode: PresentMode,
    ) -> (
        Arc<Swapchain<Window>>,
        Vec<Arc<ImageView<SwapchainImage<Window>>>>,
    ) {
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
        (swapchain, images)
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn surface(&self) -> Arc<Surface<Window>> {
        self.surface.clone()
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    pub fn resize(&mut self) {
        self.swap_chain.resize();
    }

    /// Renders scene onto scene images using frame system and finally draws UI on final
    /// swapchain images
    pub fn render(&mut self, gui: &mut Gui, layout: impl FnOnce(usize, &mut Gui) -> ()) {
        let previous_frame_future = self.swap_chain.previous_frame_end().unwrap();

        let (frame, acquire_future) = self.swap_chain.acquire_frame();

        layout(frame.image_num, gui);

        // Finally render GUI on our swapchain color image attachments
        let future = previous_frame_future.join(acquire_future);
        let after_future = gui.draw_on_image(future, frame.image.clone());
        // Finish render

        frame.present(&self.queue, after_future);
    }
}
