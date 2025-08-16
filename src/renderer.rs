pub mod swapchain;

use std::sync::Arc;

use egui_winit_vulkano::{Gui, GuiConfig};
use raw_window_handle::RawDisplayHandle;
use vulkano::{
    device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures,
        Queue, QueueCreateInfo, QueueFlags,
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
    event_loop::ActiveEventLoop,
    monitor::VideoModeHandle,
    window::{Fullscreen, Window},
};

use crate::{
    gui::{window::GuiWasabiWindow, GuiRenderer, GuiState},
    settings::WasabiSettings,
    state::WasabiState,
};

use self::swapchain::ManagedSwapchain;

pub struct Renderer {
    _instance: Arc<Instance>,
    device: Arc<Device>,
    window: Arc<Window>,
    queue: Arc<Queue>,
    swap_chain: ManagedSwapchain,

    gui: Gui,
    gui_window: GuiWasabiWindow,
}

impl Renderer {
    pub fn new(
        event_loop: &ActiveEventLoop,
        window: Window,
        settings: &mut WasabiSettings,
        state: &WasabiState,
    ) -> Self {
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

        let window = Arc::new(window);

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Failed to create surface");

        // Get most performant physical device (device with most memory)
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };
        let features = DeviceFeatures {
            geometry_shader: true,
            ..DeviceFeatures::empty()
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

        // Vulkano & Winit & egui integration
        let mut gui = Gui::new(
            &event_loop,
            surface.clone(),
            queue.clone(),
            swap_chain.state().images_state.format,
            GuiConfig {
                is_overlay: true,
                ..Default::default()
            },
        );

        let mut gui_render_data = GuiRenderer {
            gui: &mut gui,
            device: device.clone(),
            queue: queue.clone(),
            format: swap_chain.state().images_state.format,
        };

        let gui_window = GuiWasabiWindow::new(&mut gui_render_data, settings, state);

        Self {
            _instance: instance,
            device,
            queue,
            swap_chain,
            window,
            gui,
            gui_window,
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
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

    pub fn set_vsync(&mut self, enable_vsync: bool) {
        if enable_vsync {
            self.swap_chain.set_present_mode(crate::VSYNC_PRESENT_MODE);
        } else if matches!(
            self.window.display_handle().unwrap().as_raw(),
            RawDisplayHandle::Wayland(..)
        ) {
            self.swap_chain
                .set_present_mode(crate::WAYLAND_PRESENT_MODE);
        } else {
            self.swap_chain.set_present_mode(crate::PRESENT_MODE);
        }
    }

    pub fn gui(&mut self) -> &mut Gui {
        &mut self.gui
    }

    pub fn gui_window(&mut self) -> &mut GuiWasabiWindow {
        &mut self.gui_window
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

    pub fn render(&mut self, settings: &mut WasabiSettings, state: &mut WasabiState) {
        let device = self.device();
        let queue = self.queue();
        let format = self.format();

        // Get the previous frame before starting a new one
        let previous_frame_future = self.swap_chain.take_previous_frame_end().unwrap();

        // Start a new frame
        let (frame, acquire_future) = self.swap_chain.acquire_frame();

        // Join the futures
        let future = previous_frame_future.join(acquire_future);

        self.gui.immediate_ui(|gui| {
            let mut gui_render_data = GuiRenderer {
                gui,
                device,
                queue,
                format,
            };

            let mut gui_state = GuiState {
                renderer: &mut gui_render_data,
                frame: &frame,
            };
            egui_extras::install_image_loaders(&gui_state.renderer.gui.context());
            self.gui_window.layout(&mut gui_state, settings, state);
        });

        // Render the layouts
        let after_future = self
            .gui
            .draw_on_image(Box::new(future), frame.image.clone());

        // Finish render
        frame.present(&self.queue, after_future);
    }
}
