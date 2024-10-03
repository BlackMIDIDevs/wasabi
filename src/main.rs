#![feature(type_alias_impl_trait)]
#![feature(coroutines)]
#![feature(impl_trait_in_assoc_type)]

mod audio_playback;
mod gui;
mod midi;
mod renderer;
mod scenes;
mod settings;
mod state;
mod utils;

use egui_winit_vulkano::{Gui, GuiConfig};
use gui::{window::GuiWasabiWindow, GuiRenderer, GuiState};
use renderer::Renderer;
use vulkano::swapchain::PresentMode;

use egui_winit::winit::{
    dpi::{LogicalSize, Size},
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};
use settings::WasabiSettings;
use state::WasabiState;
use winit::event_loop::ControlFlow;

pub const WINDOW_SIZE: Size = Size::Logical(LogicalSize {
    width: 1280.0,
    height: 720.0,
});

pub const PRESENT_MODE: PresentMode = PresentMode::Immediate;
pub const WAYLAND_PRESENT_MODE: PresentMode = PresentMode::Mailbox;

pub fn main() {
    // Winit event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Load the settings values
    let mut settings = WasabiSettings::new_or_load();
    settings.save_to_file();
    let mut wasabi_state = WasabiState::default();

    // Create renderer for our scene & ui
    let mut renderer = Renderer::new(&event_loop, "Wasabi");

    // Vulkano & Winit & egui integration
    let mut gui = Gui::new(
        &event_loop,
        renderer.surface(),
        renderer.queue(),
        renderer.format(),
        GuiConfig {
            is_overlay: true,
            ..Default::default()
        },
    );

    let mut gui_render_data = GuiRenderer {
        gui: &mut gui,
        device: renderer.device(),
        queue: renderer.queue(),
        format: renderer.format(),
    };

    let mut gui_state = GuiWasabiWindow::new(&mut gui_render_data, &mut settings, &wasabi_state);

    event_loop
        .run(move |event, target| {
            let device = renderer.device();
            let queue = renderer.queue();
            let format = renderer.format();

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
                            target.exit();
                        }
                        WindowEvent::DroppedFile(path) => {
                            gui_state.load_midi(path, &mut settings, &wasabi_state);
                        }
                        WindowEvent::RedrawRequested => {
                            renderer.render(|frame, future| {
                                // Generate egui layouts
                                gui.immediate_ui(|gui| {
                                    let mut gui_render_data = GuiRenderer {
                                        gui,
                                        device,
                                        queue,
                                        format,
                                    };

                                    let mut state = GuiState {
                                        renderer: &mut gui_render_data,
                                        frame,
                                    };
                                    egui_extras::install_image_loaders(
                                        &state.renderer.gui.context(),
                                    );
                                    gui_state.layout(&mut state, &mut settings, &mut wasabi_state);
                                });

                                // Render the layouts
                                gui.draw_on_image(future, frame.image.clone())
                            });
                        }
                        _ => (),
                    }
                }
                Event::NewEvents(..) => {
                    renderer.window().request_redraw();
                }
                _ => (),
            }

            let mode = target
                .available_monitors()
                .next()
                .unwrap()
                .video_modes()
                .next()
                .unwrap();

            if wasabi_state.fullscreen {
                renderer.set_fullscreen(mode);
                wasabi_state.fullscreen = false;
            }
        })
        .unwrap();
}
