#![feature(type_alias_impl_trait)]
#![feature(generators)]

mod audio_playback;
mod gui;
mod midi;
mod renderer;
mod scenes;
mod settings;

use egui_winit_vulkano::Gui;
use gui::{window::GuiWasabiWindow, GuiRenderer, GuiState};
use renderer::Renderer;
use vulkano::swapchain::PresentMode;

use settings::{WasabiPermanentSettings, WasabiTemporarySettings};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Fullscreen,
};

pub fn main() {
    // Winit event loop
    let event_loop = EventLoop::new();

    // Load the settings values
    let mut perm_settings = WasabiPermanentSettings::new_or_load();
    let mut temp_settings = WasabiTemporarySettings::default();

    // Create renderer for our scene & ui
    let window_size = [1280, 720];
    let mut renderer = Renderer::new(&event_loop, window_size, PresentMode::Immediate, "Wasabi");

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

    let mut gui_state = GuiWasabiWindow::new(&mut gui_render_data, &mut perm_settings);

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
            Event::RedrawRequested(_) => {
                renderer.render(|frame, future| {
                    // Generate egui layouts
                    gui.immediate_ui(|gui| {
                        let mut state = GuiState { gui, frame };
                        gui_state.layout(&mut state, &mut perm_settings, &mut temp_settings);
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

        if temp_settings.fullscreen {
            let fullscreen = Some(Fullscreen::Exclusive(mode.clone()));
            renderer.window().set_fullscreen(fullscreen);
        } else {
            renderer.window().set_fullscreen(None);
        }
    });
}
