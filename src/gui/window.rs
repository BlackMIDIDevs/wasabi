mod keyboard;
mod keyboard_layout;
mod scene;
mod fps;
mod stats;

mod settings_window;
mod top_panel;

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use egui::{style::Margin, Frame, Visuals};

use crate::{
    GuiRenderer, GuiState,
    audio_playback::{AudioPlayerType, SimpleTemporaryPlayer},
    midi::{MIDIFileBase, MIDIFileUnion},
    settings::{WasabiPermanentSettings, WasabiTemporarySettings},
    gui::window::{keyboard::GuiKeyboard, scene::GuiRenderScene},
};

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: Option<MIDIFileUnion>,
    synth: Arc<RwLock<SimpleTemporaryPlayer>>,
    fps: fps::Fps,
}

impl GuiWasabiWindow {
    pub fn new(
        renderer: &mut GuiRenderer,
        perm_settings: &mut WasabiPermanentSettings,
    ) -> GuiWasabiWindow {
        let synth = match perm_settings.synth {
            1 => Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                AudioPlayerType::Kdmapi,
            ))),
            _ => {
                let synth = Arc::new(RwLock::new(SimpleTemporaryPlayer::new(
                    AudioPlayerType::XSynth(perm_settings.buffer_ms),
                )));
                synth
                    .write()
                    .unwrap()
                    .set_soundfont(&perm_settings.sfz_path);
                synth
                    .write()
                    .unwrap()
                    .set_layer_count(match perm_settings.layer_count {
                        0 => None,
                        _ => Some(perm_settings.layer_count),
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
        }
    }

    /// Defines the layout of our UI
    pub fn layout(
        &mut self,
        state: &mut GuiState,
        perm_settings: &mut WasabiPermanentSettings,
        temp_settings: &mut WasabiTemporarySettings,
    ) {
        let ctx = state.gui.context();
        //let window_size = vec![ctx.available_rect().width(), ctx.available_rect().height()];
        self.fps.update();
        ctx.set_visuals(Visuals::dark());
        let note_speed = perm_settings.note_speed;

        if temp_settings.settings_visible {
            settings_window::draw_settings(self, perm_settings, temp_settings, &ctx);
        }

        let panel_height = if temp_settings.panel_visible {
            let height = 40.0;
            top_panel::draw_panel(self, perm_settings, temp_settings, &ctx, height);
            height + 20.0
        } else {
            0.0
        };

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let visible_keys = perm_settings.key_range.len();
        let keyboard_height = 11.6 / visible_keys as f32 * available.width();
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(
            *perm_settings.key_range.start() as usize,
            *perm_settings.key_range.end() as usize,
        );

        let no_frame = Frame::default()
            .inner_margin(Margin::same(0.0))
            .fill(perm_settings.bg_color);

        let mut stats = stats::GuiMidiStats::empty();

        let mut render_result_data = None;

        // Render the notes
        egui::TopBottomPanel::top("Note panel")
            .height_range(notes_height..=notes_height)
            .frame(no_frame)
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
                                    egui::Key::Space => midi_file.timer_mut().toggle_pause(),
                                    _ => {}
                                }
                            }
                        }
                    }

                    let result = self
                        .render_scene
                        .draw(state, ui, &key_view, midi_file, note_speed);
                    let notes_on_screen = result.notes_rendered;
                    stats.set_rendered_note_count(notes_on_screen);
                    render_result_data = Some(result);
                }
            });

        // Render the keyboard
        egui::TopBottomPanel::top("Keyboard panel")
            .height_range(keyboard_height..=keyboard_height)
            .frame(no_frame)
            .show(&ctx, |ui| {
                let events = ui.input().events.clone();
                for event in &events {
                    if let egui::Event::Key { key, pressed, .. } = event {
                        if pressed == &true {
                            match key {
                                egui::Key::F => match temp_settings.panel_visible {
                                    true => temp_settings.panel_visible = false,
                                    false => temp_settings.panel_visible = true,
                                },
                                egui::Key::G => match temp_settings.stats_visible {
                                    true => temp_settings.stats_visible = false,
                                    false => temp_settings.stats_visible = true,
                                },
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
                    .draw(ui, &key_view, &colors, &perm_settings.bar_color);
            });

        // Render the stats
        if temp_settings.stats_visible {
            let voice_count = {
                let x = if let Ok(player) = self.synth.read() {
                    player.get_voice_count()
                } else {
                    0
                };
                x
            };
            stats.set_voice_count(voice_count);

            let pos = egui::Pos2::new(10.0, panel_height + 10.0);
            stats::draw_stats(self, &ctx, pos, stats);
        }
    }

    fn reset_synth(&mut self) {
        if let Ok(mut player) = self.synth.write() {
            player.reset()
        };
    }
}
