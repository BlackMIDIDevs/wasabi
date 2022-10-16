mod keyboard;
mod keyboard_layout;
mod scene;

use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use core::ops::RangeInclusive;

use egui::{style::Margin, Frame, Label, Visuals};

use rfd::FileDialog;

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::{InRamMIDIFile, MIDIFileBase, MIDIFileUnion},
};

use self::{keyboard::GuiKeyboard, scene::GuiRenderScene};

use super::{GuiRenderer, GuiState};
use crate::settings::{WasabiPermanentSettings, WasabiTemporarySettings};

struct Fps(VecDeque<Instant>);

const FPS_WINDOW: f64 = 0.5;

impl Fps {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn update(&mut self) {
        self.0.push_back(Instant::now());
        while let Some(front) = self.0.front() {
            if front.elapsed().as_secs_f64() > FPS_WINDOW {
                self.0.pop_front();
            } else {
                break;
            }
        }
    }

    fn get_fps(&self) -> f64 {
        if self.0.is_empty() {
            0.0
        } else {
            self.0.len() as f64 / self.0.front().unwrap().elapsed().as_secs_f64()
        }
    }
}

struct GuiMidiStats {
    time_passed: f64,
    time_total: f64,
    //notes_passed: usize,
    notes_total: usize,
    notes_on_screen: u64,
    voice_count: u64,
}

impl GuiMidiStats {
    fn empty() -> GuiMidiStats {
        GuiMidiStats {
            time_passed: 0.0,
            time_total: 0.0,
            //notes_passed: 0,
            notes_total: 0,
            notes_on_screen: 0,
            voice_count: 0,
        }
    }
}

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: Option<MIDIFileUnion>,
    synth: Option<Arc<RwLock<SimpleTemporaryPlayer>>>,
    fps: Fps,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer) -> GuiWasabiWindow {
        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file: None,
            synth: None,
            fps: Fps::new(),
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
        let window_size = vec![ctx.available_rect().width(), ctx.available_rect().height()];
        self.fps.update();
        ctx.set_visuals(Visuals::dark());
        let note_speed = perm_settings.note_speed;

        // Render the settings window if the value from
        // the temporary settings allows it
        if temp_settings.settings_visible {
            egui::Window::new("Settings")
                .resizable(true)
                .collapsible(true)
                .title_bar(true)
                .scroll2([false, true])
                .enabled(true)
                .open(&mut temp_settings.settings_visible)
                .show(&ctx, |ui| {
                    egui::Grid::new("settings_grid")
                        .num_columns(2)
                        .spacing([40.0, 4.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("SFZ Path: ");
                            ui.horizontal(|ui| {
                                ui.add(egui::TextEdit::singleline(&mut perm_settings.sfz_path));
                                if ui.button("Browse...").clicked() {
                                    let sfz_path = FileDialog::new()
                                        .add_filter("sfz", &["sfz"])
                                        .set_directory("/")
                                        .pick_file();

                                    if let Some(sfz_path) = sfz_path {
                                        if let Ok(path) = sfz_path.into_os_string().into_string() {
                                            perm_settings.sfz_path = path;
                                        }
                                    }
                                }
                            });
                            ui.end_row();

                            ui.label("Note speed: ");
                            ui.spacing_mut().slider_width = 150.0;
                            ui.add(egui::Slider::new(
                                &mut perm_settings.note_speed,
                                2.0..=0.001,
                            ));
                            ui.end_row();

                            ui.label("Background Color: ");
                            ui.color_edit_button_srgba(&mut perm_settings.bg_color);
                            ui.end_row();

                            ui.label("Bar Color: ");
                            ui.color_edit_button_srgba(&mut perm_settings.bar_color);
                            ui.end_row();

                            ui.label("Random Track Colors: ");
                            ui.checkbox(&mut perm_settings.random_colors, "");
                            ui.end_row();

                            ui.label("Keyboard Range: ");
                            let mut firstkey = *perm_settings.key_range.start();
                            let mut lastkey = *perm_settings.key_range.end();
                            ui.horizontal(|ui| {
                                ui.add(
                                    egui::DragValue::new(&mut firstkey)
                                        .speed(1)
                                        .clamp_range(RangeInclusive::new(0, 255)),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut lastkey)
                                        .speed(1)
                                        .clamp_range(RangeInclusive::new(0, 255)),
                                );
                            });
                            ui.end_row();
                            if firstkey != *perm_settings.key_range.start()
                                || lastkey != *perm_settings.key_range.end()
                            {
                                perm_settings.key_range = firstkey..=lastkey;
                            }
                        });
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        if ui.button("Save").clicked() {
                            perm_settings.save_to_file();
                        }
                    });
                });
        }

        // Render the top panel if the value from
        // the temporary settings allows it,
        // and return its height in a variable
        let panel_height = if temp_settings.panel_visible {
            let panel_height = 40.0;
            let panel_frame = Frame::default()
                .inner_margin(egui::style::Margin::same(10.0))
                .fill(egui::Color32::from_rgb(42, 42, 42));
            egui::TopBottomPanel::top("Top panel")
                .height_range(panel_height..=panel_height)
                .frame(panel_frame)
                .show(&ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Open MIDI").clicked() {
                            let midi_path = FileDialog::new()
                                .add_filter("midi", &["mid"])
                                .set_directory("/")
                                .pick_file();

                            if let Some(midi_path) = midi_path {
                                if let Some(midi_file) = self.midi_file.as_mut() {
                                    midi_file.timer_mut().pause();
                                }
                                self.reset_synth();

                                self.midi_file = None;
                                self.synth = None;

                                let synth = SimpleTemporaryPlayer::new(&perm_settings.sfz_path);
                                let synth = RwLock::new(synth);
                                let synth = Arc::new(synth);
                                self.synth = Some(synth.clone());

                                if let Ok(path) = midi_path.into_os_string().into_string() {
                                    let mut midi_file =
                                        MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
                                            &path,
                                            synth,
                                            perm_settings.random_colors,
                                        ));
                                    midi_file.timer_mut().play();
                                    self.midi_file = Some(midi_file);
                                }
                            }
                        }
                        if let Some(midi_file) = self.midi_file.as_mut() {
                            if ui.button("Close MIDI").clicked() {
                                midi_file.timer_mut().pause();
                                self.reset_synth();
                                self.midi_file = None;
                            }
                        }
                        if ui.button("Play").clicked() {
                            if let Some(midi_file) = self.midi_file.as_mut() {
                                midi_file.timer_mut().play();
                            }
                        }
                        if ui.button("Pause").clicked() {
                            if let Some(midi_file) = self.midi_file.as_mut() {
                                midi_file.timer_mut().pause();
                            }
                        }
                        if ui.button("Settings").clicked() {
                            match temp_settings.settings_visible {
                                true => temp_settings.settings_visible = false,
                                false => temp_settings.settings_visible = true,
                            }
                        }
                        ui.horizontal(|ui| {
                            ui.label("Note speed: ");
                            ui.add(
                                egui::Slider::new(&mut perm_settings.note_speed, 2.0..=0.001)
                                    .show_value(false),
                            );
                        })
                    });

                    if let Some(midi_file) = self.midi_file.as_mut() {
                        if let Some(length) = midi_file.midi_length() {
                            let time = midi_file.timer().get_time().as_secs_f64();
                            let mut progress = time / length;
                            let progress_prev = progress;
                            let slider =
                                egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false);
                            ui.spacing_mut().slider_width = window_size[0] - 20.0;
                            ui.add(slider);
                            if progress_prev != progress {
                                let position = Duration::from_secs_f64(progress * length);
                                midi_file.timer_mut().seek(position);
                            }
                        }
                    } else {
                        let mut progress = 0.0;
                        let slider = egui::Slider::new(&mut progress, 0.0..=1.0).show_value(false);
                        ui.spacing_mut().slider_width = window_size[0] - 20.0;
                        ui.add(slider);
                    }
                });
            panel_height + 20.0
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

        let stats_frame = Frame::default()
            .inner_margin(egui::style::Margin::same(6.0))
            .fill(egui::Color32::from_rgba_unmultiplied(15, 15, 15, 200))
            .rounding(egui::Rounding::same(4.0));

        let mut stats = GuiMidiStats::empty();

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
                                        midi_file.timer_mut().seek(time - one_sec)
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
                    stats.notes_on_screen = result.notes_rendered;
                    render_result_data = Some(result);
                }
            });

        stats.voice_count = if let Some(synth) = &self.synth {
            let x = if let Ok(player) = synth.read() {
                player.get_voice_count()
            } else {
                0
            };
            x
        } else {
            0
        };

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
            egui::Window::new("Stats")
                .resizable(false)
                .collapsible(false)
                .title_bar(false)
                .scroll2([false, false])
                .enabled(true)
                .frame(stats_frame)
                .fixed_pos(egui::Pos2::new(10.0, panel_height + 10.0))
                .show(&ctx, |ui| {
                    let mut time_sec: u64 = 0;
                    let mut time_min: u64 = 0;
                    let mut length_sec: u64 = 0;
                    let mut length_min: u64 = 0;

                    if let Some(midi_file) = self.midi_file.as_mut() {
                        stats.time_total = if let Some(length) = midi_file.midi_length() {
                            length
                        } else {
                            0.0
                        };
                        let time = midi_file.timer().get_time().as_secs_f64();
                        let length_u64 = stats.time_total as u64;
                        length_sec = length_u64 % 60;
                        length_min = (length_u64 / 60) % 60;
                        if time > stats.time_total {
                            stats.time_passed = stats.time_total;
                        } else {
                            stats.time_passed = time;
                        }
                        let time_u64 = stats.time_passed as u64;
                        time_sec = time_u64 % 60;
                        time_min = (time_u64 / 60) % 60;

                        stats.notes_total = midi_file.stats().total_notes;
                    }
                    ui.add(Label::new(format!(
                        "Time: {:0width$}:{:0width$}/{:0width$}:{:0width$}",
                        time_min,
                        time_sec,
                        length_min,
                        length_sec,
                        width = 2
                    )));
                    ui.add(Label::new(format!("FPS: {}", self.fps.get_fps().round())));
                    ui.add(Label::new(format!("Total Notes: {}", stats.notes_total)));
                    //ui.add(Label::new(format!("Passed: {}", -1)));  // TODO
                    ui.add(Label::new(format!("Voice Count: {}", stats.voice_count)));
                    ui.add(Label::new(format!("Rendered: {}", stats.notes_on_screen)));
                });
        }
    }

    fn reset_synth(&mut self) {
        if let Some(synth) = &self.synth {
            if let Ok(mut player) = synth.write() {
                player.reset()
            };
        };
    }
}
