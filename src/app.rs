use crate::{renderer::Renderer, settings::WasabiSettings, state::WasabiState, utils};
use egui_winit::winit::event::WindowEvent;
use winit::{
    application::ApplicationHandler,
    event_loop::ActiveEventLoop,
    window::{Icon, WindowAttributes, WindowId},
};

const ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon.bitmap"));

pub struct WasabiApplication {
    settings: WasabiSettings,
    state: WasabiState,

    renderer: Option<Renderer>,
}

impl WasabiApplication {
    pub fn new() -> Self {
        // Load the settings values
        let state = WasabiState::new();
        let settings = WasabiSettings::new_or_load().unwrap_or_else(|e| {
            state.errors.error(&e);
            WasabiSettings::default()
        });
        settings
            .save_to_file()
            .unwrap_or_else(|e| state.errors.error(&e));

        if settings.gui.check_for_updates {
            utils::check_for_updates(&state);
        }

        Self {
            settings,
            state,
            renderer: None,
        }
    }
}

impl ApplicationHandler for WasabiApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_none() {
            let win_attr = WindowAttributes::default()
                .with_window_icon(Some(Icon::from_rgba(ICON.to_vec(), 16, 16).unwrap()))
                .with_inner_size(crate::WINDOW_SIZE)
                .with_title("Wasabi");
            let window = event_loop.create_window(win_attr).unwrap();
            self.renderer = Some(Renderer::new(
                event_loop,
                window,
                &mut self.settings,
                &self.state,
            ))
        }
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.window().request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(renderer) = self.renderer.as_mut() {
            let _pass_events_to_game = !renderer.gui().update(&event);

            match event {
                WindowEvent::Resized(size) => {
                    renderer.resize(Some(size));
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    renderer.resize(None);
                }
                WindowEvent::CloseRequested => {
                    event_loop.exit();
                }
                WindowEvent::DroppedFile(path) => {
                    renderer
                        .gui_window()
                        .load_midi(path, &mut self.settings, &mut self.state);
                }
                WindowEvent::RedrawRequested => {
                    renderer.render(&mut self.settings, &mut self.state);
                }
                _ => (),
            }

            let mode = event_loop
                .available_monitors()
                .next()
                .unwrap()
                .video_modes()
                .next()
                .unwrap();

            if self.state.fullscreen {
                renderer.set_fullscreen(mode);
                self.state.fullscreen = false;
            }
        }
    }
}
