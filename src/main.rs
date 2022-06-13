mod gui;
mod renderer;
mod scenes;
mod window;

use egui_winit_vulkano::Gui;
use renderer::Renderer;
use vulkano::swapchain::PresentMode;
use window::GuiWasabiWindow;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

#[path = "./old/main.rs"]
mod old;

pub fn main() {
    // Winit event loop
    let event_loop = EventLoop::new();

    // Create renderer for our scene & ui
    let window_size = [1280, 720];
    let mut renderer = Renderer::new(&event_loop, window_size, PresentMode::Mailbox, "Wholesome");

    // Vulkano & Winit & egui integration
    let mut gui = Gui::new(renderer.surface(), renderer.queue(), false);

    let mut gui_state = GuiWasabiWindow::new(&mut gui);
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
                    gui.immediate_ui(|gui| {
                        let ctx = gui.context();
                        gui_state.layout(ctx);
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
