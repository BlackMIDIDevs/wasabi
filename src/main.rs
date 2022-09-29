#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]
#![feature(generators)]

mod audio_playback;
mod gui;
mod midi;
mod renderer;
mod scenes;

use egui_winit_vulkano::Gui;
use gui::{window::GuiWasabiWindow, GuiRenderer, GuiState};
use renderer::Renderer;
use vulkano::{
    format::Format,
    image::{ImageUsage, StorageImage},
    swapchain::PresentMode,
};
use vulkano_util::{
    context::{VulkanoConfig, VulkanoContext},
    renderer::{DeviceImageView, DEFAULT_IMAGE_FORMAT},
    window::{VulkanoWindows, WindowDescriptor},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub fn main() {
    // Winit event loop
    let event_loop = EventLoop::new();

    // Create renderer for our scene & ui
    let window_size = [1280, 720];
    let mut renderer = Renderer::new(
        &event_loop,
        window_size,
        PresentMode::Immediate,
        "Wholesome",
    );

    // Vulkano & Winit & egui integration
    let mut gui = Gui::new(
        &event_loop,
        renderer.surface(),
        Some(renderer.format()),
        renderer.queue(),
        false,
    );

    let mut gui_render_data = GuiRenderer {
        gui: &mut gui,
        device: renderer.device(),
        queue: renderer.queue(),
        format: renderer.format(),
    };

    let mut gui_state = GuiWasabiWindow::new(&mut gui_render_data);

    event_loop.run(move |event, _, control_flow| {
        // Update Egui integration so the UI works!
        match event {
            Event::WindowEvent { event, window_id }
                if window_id == renderer.surface().window().id() =>
            {
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(_) => {
                        renderer.resize();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize();
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if window_id == window_id => {
                renderer.render(|frame, future| {
                    // Generate egui layouts
                    gui.immediate_ui(|mut gui| {
                        let mut state = GuiState {
                            gui: &mut gui,
                            frame: &frame,
                        };
                        gui_state.layout(&mut state);
                    });

                    // Render the layouts
                    gui.draw_on_image(future, frame.image.clone())
                });
            }
            Event::MainEventsCleared => {
                renderer.window().request_redraw();
            }
            _ => (),
        }
    });
}
