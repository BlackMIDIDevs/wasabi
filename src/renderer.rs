pub mod swapchain;

use std::sync::Arc;

use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo,
    },
    format::Format,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::{PresentMode, Surface},
    sync::GpuFuture,
    Version, VulkanLibrary,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use self::swapchain::{ManagedSwapchain, SwapchainFrame};

pub struct Renderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface<Window>>,
    queue: Arc<Queue>,
    swap_chain: ManagedSwapchain,
}

impl Renderer {
    pub fn new(
        event_loop: &EventLoop<()>,
        window_size: [u32; 2],
        present_mode: PresentMode,
        name: &str,
    ) -> Self {
        // Why
        let library = VulkanLibrary::new().unwrap();

        // Add instance extensions based on needs
        let instance_extensions = InstanceExtensions {
            ..vulkano_win::required_extensions(&library)
        };

        // Create instance
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                application_version: Version::V1_2,
                enabled_extensions: instance_extensions,
                ..Default::default()
            },
        )
        .expect("Failed to create instance");

        // Create rendering surface along with window
        let surface = WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(window_size[0], window_size[1]))
            .with_title(name)
            .build_vk_surface(event_loop, instance.clone())
            .expect("Failed to create vulkan surface & window");

        // Get most performant physical device (device with most memory)
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let features = Features {
            geometry_shader: true,
            ..Features::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.graphics
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
                    })
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        // Create device
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                enabled_features: features,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        // Create swap chain & frame(s) to which we'll render
        let swap_chain = ManagedSwapchain::create(
            surface.clone(),
            physical_device.clone(),
            device.clone(),
            present_mode,
        );

        let queue = queues.next().unwrap();

        Self {
            _instance: instance,
            device,
            surface,
            queue,
            swap_chain,
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn surface(&self) -> Arc<Surface<Window>> {
        self.surface.clone()
    }

    pub fn window(&self) -> &Window {
        self.surface.window()
    }

    pub fn format(&self) -> Format {
        self.swap_chain.state().images_state.format
    }

    pub fn resize(&mut self) {
        self.swap_chain.resize();
    }

    pub fn render(
        &mut self,
        draw: impl FnOnce(&SwapchainFrame, Box<dyn GpuFuture>) -> Box<dyn GpuFuture>,
    ) {
        // Get the previous frame before starting a new one
        let previous_frame_future = self.swap_chain.take_previous_frame_end().unwrap();

        // Start a new frame
        let (frame, acquire_future) = self.swap_chain.acquire_frame();

        // Join the futures
        let future = previous_frame_future.join(acquire_future);

        // Call the passed-in renderer
        let after_future = draw(&frame, Box::new(future));

        // Finish render
        frame.present(&self.queue, after_future);
    }
}
