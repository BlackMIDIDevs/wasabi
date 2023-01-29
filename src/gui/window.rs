mod fps;
mod keyboard;
mod keyboard_layout;
mod scene;
mod stats;

mod settings_window;
mod top_panel;
mod xsynth_settings;

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use egui::{style::Margin, Frame, Visuals};

use crate::{
    audio_playback::{
        xsynth::{convert_to_channel_init, convert_to_sf_init},
        AudioPlayerType, SimpleTemporaryPlayer,
    },
    gui::window::{keyboard::GuiKeyboard, scene::GuiRenderScene},
    midi::{InRamMIDIFile, LiveLoadMIDIFile, MIDIFileBase, MIDIFileUnion},
    settings::WasabiSettings,
    state::WasabiState,
    GuiRenderer, GuiState,
};

use egui_file::FileDialog;

pub struct WasabiFileDialogs {
    midi_file_dialog: Option<FileDialog>,
    sf_file_dialog: Option<FileDialog>,
}

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: Option<MIDIFileUnion>,
    synth: Arc<RwLock<SimpleTemporaryPlayer>>,
    fps: fps::Fps,
    file_dialogs: WasabiFileDialogs,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer, settings: &mut WasabiSettings) -> GuiWasabiWindow {
        let synth = match settings.synth {
            1 => Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                AudioPlayerType::Kdmapi,
            ))),
            _ => {
                let synth = Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                    AudioPlayerType::XSynth {
                        buffer: settings.buffer_ms,
                        ignore_range: settings.vel_ignore.clone(),
                        options: convert_to_channel_init(settings),
                    },
                )));
                synth
                    .write()
                    .unwrap()
                    .set_soundfont(&settings.sfz_path, convert_to_sf_init(settings));
                synth
                    .write()
                    .unwrap()
                    .set_layer_count(match settings.layer_count {
                        0 => None,
                        _ => Some(settings.layer_count),
                    });
                synth
            }
        };
        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file: None,
            synth,
            fps: fps::Fps::new(),
            file_dialogs: WasabiFileDialogs {
                midi_file_dialog: None,
                sf_file_dialog: None,
            },
        }
    }

    /// Defines the layout of our UI
    pub fn layout(
        &mut self,
        state: &mut GuiState,
        settings: &mut WasabiSettings,
        wasabi_state: &mut WasabiState,
    ) {
        let ctx = state.gui.context();
        self.fps.update();
        ctx.set_visuals(Visuals::dark());

        if wasabi_state.settings_visible {
            settings_window::draw_settings(self, settings, wasabi_state, &ctx);
        }
        if wasabi_state.xsynth_settings_visible {
            xsynth_settings::draw_xsynth_settings(self, settings, wasabi_state, &ctx);
        }

        if let Some(dialog) = &mut self.file_dialogs.midi_file_dialog {
            if dialog.show(&ctx).selected() {
                if let Some(midi_path) = dialog.path() {
                    wasabi_state.last_midi_file = Some(midi_path.clone());
                    self.load_midi(settings, midi_path);
                }
                self.file_dialogs.midi_file_dialog = None;
            }
        }

        let height_prev = ctx.available_rect().height();
        if wasabi_state.panel_visible {
            top_panel::draw_panel(self, settings, wasabi_state, &ctx);
        }

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let panel_height = height_prev - height;
        let keyboard_height =
            (11.6 / settings.key_range.len() as f32 * available.width()).min(height / 2.0);
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(
            *settings.key_range.start() as usize,
            *settings.key_range.end() as usize,
        );

        let no_frame = Frame::default()
            .inner_margin(Margin::same(0.0))
            .fill(settings.bg_color);

        let mut stats = stats::GuiMidiStats::empty();

        let mut render_result_data = None;

        // Render the notes
        egui::TopBottomPanel::top("Note panel")
            .height_range(notes_height..=notes_height)
            .frame(no_frame)
            .show_separator_line(false)
            .show(&ctx, |ui| {
                if let Some(midi_file) = self.midi_file.as_mut() {
                    let one_sec = Duration::from_secs(1);
                    let time = midi_file.timer().get_time();

                    let events = ui.input().events.clone();
                    for event in &events {
                        if let egui::Event::Key { key, pressed, .. } = event {
                            if pressed == &true {
                                match key {
                                    egui::Key::ArrowRight => {
                                        midi_file.timer_mut().seek(time + one_sec)
                                    }
                                    egui::Key::ArrowLeft => {
                                        if midi_file.allows_seeking_backward() {
                                            midi_file.timer_mut().seek(if time <= one_sec {
                                                Duration::from_secs(0)
                                            } else {
                                                time - one_sec
                                            })
                                        }
                                    }
                                    egui::Key::ArrowUp => {
                                        settings.note_speed += 0.05;
                                    }
                                    egui::Key::ArrowDown => {
                                        settings.note_speed -= 0.05;
                                    }
                                    egui::Key::Space => midi_file.timer_mut().toggle_pause(),
                                    _ => {}
                                }
                            }
                        }
                    }

                    let result = self.render_scene.draw(
                        state,
                        ui,
                        &key_view,
                        midi_file,
                        settings.note_speed,
                    );
                    stats.set_rendered_note_count(result.notes_rendered);
                    render_result_data = Some(result);
                }
            });

        // Render the keyboard
        egui::TopBottomPanel::top("Keyboard panel")
            .height_range(keyboard_height..=keyboard_height)
            .frame(no_frame)
            .show_separator_line(false)
            .show(&ctx, |ui| {
                let events = ui.input().events.clone();
                for event in &events {
                    if let egui::Event::Key {
                        key,
                        pressed,
                        modifiers,
                    } = event
                    {
                        if *pressed && modifiers.ctrl {
                            match key {
                                egui::Key::F => {
                                    wasabi_state.panel_visible = !wasabi_state.panel_visible
                                }
                                egui::Key::G => {
                                    wasabi_state.stats_visible = !wasabi_state.stats_visible
                                }
                                //egui::Key::O => self.open_midi_dialog(wasabi_state),
                                _ => {}
                            }
                        }
                        if *pressed && modifiers.alt {
                            match key {
                                egui::Key::Enter => {
                                    wasabi_state.fullscreen = !wasabi_state.fullscreen
                                }
                                _ => {}
                            }
                        }
                    }
                }

                let colors = if let Some(data) = render_result_data {
                    data.key_colors
                } else {
                    vec![None; 256]
                };

                self.keyboard
                    .draw(ui, &key_view, &colors, &settings.bar_color);
            });

        // Render the stats
        if wasabi_state.stats_visible {
            let voice_count = self.synth.read().unwrap().get_voice_count();
            stats.set_voice_count(voice_count);

            let pos = egui::Pos2::new(10.0, panel_height + 10.0);
            stats::draw_stats(self, &ctx, pos, stats);
        }
    }

    pub fn open_midi_dialog(&mut self, state: &mut WasabiState) {
        let filter = |path: &std::path::Path| {
            if let Some(path) = path.to_str() {
                path.ends_with(".mid")
            } else {
                false
            }
        };
        let filter = Box::new(filter);

        let mut dialog = FileDialog::open_file(state.last_midi_file.clone())
            .show_rename(false)
            .show_new_folder(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 10.0))
            .filter(filter);

        dialog.open();
        self.file_dialogs.midi_file_dialog = Some(dialog);
    }

    pub fn load_midi(&mut self, settings: &mut WasabiSettings, midi_path: PathBuf) {
        if let Some(midi_file) = self.midi_file.as_mut() {
            midi_file.timer_mut().pause();
        }
        self.synth.write().unwrap().reset();
        self.midi_file = None;

        if let Some(midi_path) = midi_path.to_str() {
            match settings.midi_loading {
                0 => {
                    let mut midi_file = MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
                        midi_path,
                        self.synth.clone(),
                        settings.random_colors,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
                1 => {
                    let mut midi_file = MIDIFileUnion::Live(LiveLoadMIDIFile::load_from_file(
                        midi_path,
                        self.synth.clone(),
                        settings.random_colors,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
                _ => {}
            }
        }
    }
}
