mod swapchain;

use std::sync::Arc;

use vulkano::{
    device::{
        physical::PhysicalDevice, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo,
    },
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::{PresentMode, Surface},
    sync::GpuFuture,
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use self::swapchain::{ManagedSwapchain, SwapchainFrame};

pub struct Renderer {
    _instance: Arc<Instance>,
    _device: Arc<Device>,
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

        // Get most performant physical device (device with most memory)
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
            _instance: instance,
            _device: device,
            surface,
            queue,
            swap_chain,
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
