#![feature(type_alias_impl_trait)]
#![feature(generators)]
#![feature(impl_trait_in_assoc_type)]

mod audio_playback;
mod gui;
mod midi;
mod renderer;
mod scenes;
mod settings;
mod state;

use egui_winit_vulkano::Gui;
use gui::{window::GuiWasabiWindow, GuiRenderer, GuiState};
use renderer::Renderer;
use vulkano::swapchain::PresentMode;

use settings::WasabiSettings;
use state::WasabiState;
use winit::{
    dpi::{LogicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

pub const WINDOW_SIZE: Size = Size::Logical(LogicalSize {
    width: 1280.0,
    height: 720.0,
});

pub const PRESENT_MODE: PresentMode = PresentMode::Immediate;
pub const WAYLAND_PRESENT_MODE: PresentMode = PresentMode::Mailbox;

pub fn main() {
    // Winit event loop
    let event_loop = EventLoop::new();

    // Load the settings values
    let mut settings = WasabiSettings::new_or_load();
    let mut wasabi_state = WasabiState::default();

    // Create renderer for our scene & ui
    let mut renderer = Renderer::new(&event_loop, "Wasabi");

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

    let mut gui_state = GuiWasabiWindow::new(&mut gui_render_data, &mut settings);

    let monitor = event_loop
        .available_monitors()
        .next()
        .expect("no monitor found!");

    let mode = monitor.video_modes().next().expect("no mode found");

    event_loop.run(move |event, _, control_flow| {
        // Update Egui integration so the UI works!
        match event {
            Event::WindowEvent { event, window_id } if window_id == renderer.window().id() => {
                let _pass_events_to_game = !gui.update(&event);
                match event {
                    WindowEvent::Resized(size) => {
                        renderer.resize(Some(size));
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        renderer.resize(None);
                    }
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    }
                    WindowEvent::DroppedFile(path) => {
                        gui_state.load_midi(&mut settings, path);
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(_) => {
                renderer.render(|frame, future| {
                    // Generate egui layouts
                    gui.immediate_ui(|gui| {
                        let mut state = GuiState { gui, frame };
                        gui_state.layout(&mut state, &mut settings, &mut wasabi_state);
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

        if wasabi_state.fullscreen {
            renderer.set_fullscreen(mode.clone());
            wasabi_state.fullscreen = false;
        }
    });
}
