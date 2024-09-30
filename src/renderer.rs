pub mod swapchain;

use std::sync::Arc;

use raw_window_handle::RawDisplayHandle;
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue,
        QueueCreateInfo, QueueFlags,
    },
    format::Format,
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::Surface,
    sync::GpuFuture,
    Version, VulkanLibrary,
};

use raw_window_handle::HasDisplayHandle;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    monitor::VideoModeHandle,
    window::{Fullscreen, Icon, Window, WindowAttributes},
};

use self::swapchain::{ManagedSwapchain, SwapchainFrame};

const ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.bitmap"));

pub struct Renderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    surface: Arc<Surface>,
    window: Arc<Window>,
    queue: Arc<Queue>,
    swap_chain: ManagedSwapchain,
}

impl Renderer {
    pub fn new(event_loop: &EventLoop<()>, name: &str) -> Self {
        // Why
        let library = VulkanLibrary::new().unwrap();

        // Add instance extensions based on needs
        let instance_extensions = InstanceExtensions {
            ..Surface::required_extensions(event_loop).unwrap()
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
        let win_attr = WindowAttributes::default()
            .with_window_icon(Some(Icon::from_rgba(ICON.to_vec(), 16, 16).unwrap()))
            .with_inner_size(crate::WINDOW_SIZE)
            .with_title(name);
        let window = event_loop
            .create_window(win_attr)
            .expect("Failed to create vulkan surface & window");

        let window = Arc::new(window);

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Failed to create surface");

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
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
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
            window.clone(),
            physical_device,
            device.clone(),
            #[cfg(target_os = "linux")]
            if matches!(
                event_loop.display_handle().unwrap().as_raw(),
                RawDisplayHandle::Wayland(..)
            ) {
                println!("Present Mode: {:?}", crate::WAYLAND_PRESENT_MODE);
                crate::WAYLAND_PRESENT_MODE
            } else {
                println!("Present Mode: {:?}", crate::PRESENT_MODE);
                crate::PRESENT_MODE
            },
            #[cfg(not(target_os = "linux"))]
            crate::PRESENT_MODE,
        );

        let queue = queues.next().unwrap();

        Self {
            _instance: instance,
            device,
            surface,
            queue,
            swap_chain,
            window,
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn format(&self) -> Format {
        self.swap_chain.state().images_state.format
    }

    pub fn resize(&mut self, size: Option<PhysicalSize<u32>>) {
        self.swap_chain.resize(size);
    }

    pub fn set_fullscreen(&self, mode: VideoModeHandle) {
        if self.window.fullscreen().is_none() {
            let fullscreen = if matches!(
                self.window.display_handle().unwrap().as_raw(),
                RawDisplayHandle::Wayland(..)
            ) {
                Some(Fullscreen::Borderless(None))
            } else {
                Some(Fullscreen::Exclusive(mode))
            };
            self.window.set_fullscreen(fullscreen);
        } else {
            self.window.set_fullscreen(None);
        }
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
