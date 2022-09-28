mod keyboard;
mod keyboard_layout;
mod scene;

use std::{
    collections::VecDeque,
    time::{Duration, Instant},
    env,
};

use egui::{style::Margin, Frame, Label, Visuals};

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::{InRamMIDIFile, MIDIFileBase, MIDIFileUnion},
};

use self::{keyboard::GuiKeyboard, scene::GuiRenderScene};

use super::{GuiRenderer, GuiState};

struct FPS(VecDeque<Instant>);

const FPS_WINDOW: f64 = 0.5;

impl FPS {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn update(&mut self) {
        self.0.push_back(Instant::now());
        loop {
            if let Some(front) = self.0.front() {
                if front.elapsed().as_secs_f64() > FPS_WINDOW {
                    self.0.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn get_fps(&self) -> f64 {
        if self.0.len() == 0 {
            return 0.0;
        } else {
            self.0.len() as f64 / self.0.front().unwrap().elapsed().as_secs_f64()
        }
    }
}

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: MIDIFileUnion,
    fps: FPS,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer) -> GuiWasabiWindow {
        let args: Vec<String> = env::args().collect();
        let mut midi_file = MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
            &args[1],
            SimpleTemporaryPlayer::new(),
        ));

        midi_file.timer_mut().play();

        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file,
            fps: FPS::new(),
        }
    }

    /// Defines the layout of our UI
    pub fn layout(&mut self, state: &mut GuiState) {
        let ctx = state.gui.context();

        let window_size = vec![ctx.available_rect().width(), ctx.available_rect().height()];

        self.fps.update();

        ctx.set_visuals(Visuals::dark());

        let note_speed = 0.25;

        // Render the top panel
        let panel_height = 40.0;
        let panel_frame = Frame::default()
            .margin(egui::style::Margin::same(10.0))
            .fill(egui::Color32::from_rgb(42, 42, 42));

        egui::TopBottomPanel::top("Top panel")
            .height_range(panel_height..=panel_height)
            .frame(panel_frame)
            .show(&ctx, |ui| {
                let one_sec = Duration::from_secs(1);
                let _five_sec = Duration::from_secs(5);
                let time = self.midi_file.timer().get_time();
                let events = ui.input().events.clone();
                for event in &events {
                    match event {
                        egui::Event::Key{key, pressed, ..} => if pressed == &true {
                            match key {
                                egui::Key::ArrowRight => self.midi_file.timer_mut().seek(time + one_sec),
                                egui::Key::ArrowLeft => self.midi_file.timer_mut().seek(time - one_sec),
                                egui::Key::Space => self.midi_file.timer_mut().toggle_pause(),
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("Open MIDI (N/A)").clicked() {
                        // TODO
                    }
                    if ui.button("Play").clicked() {
                        self.midi_file.timer_mut().play();
                    }
                    if ui.button("Pause").clicked() {
                        self.midi_file.timer_mut().pause();
                    }
                    if ui.button("Settings (N/A)").clicked() {
                        // TODO
                    }
                });

                if let Some(length) = self.midi_file.midi_length() {
                    let time = self.midi_file.timer().get_time().as_secs_f64();
                    let mut progress = time / length;
                    let progress_prev = progress.clone();
                    let slider = egui::Slider::new(&mut progress, 0.0..=1.0)
                        .show_value(false);
                    ui.spacing_mut().slider_width = window_size[0] - 15.0;
                    ui.add(slider);
                    if progress_prev != progress {
                        let position = Duration::from_secs_f64(progress * length);
                        self.midi_file.timer_mut().seek(position);
                    }
                }
            });

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let keyboard_height = 70.0 / 760.0 * available.width() as f32;
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(0, 127);

        let no_frame = Frame::default()
            .margin(Margin::same(0.0))
            .fill(egui::Color32::from_rgb(90, 90, 90));

        let mut render_result_data = None;

        let stats = self.midi_file.stats();

        // Render the notes
        egui::TopBottomPanel::top("Note panel")
            .height_range(notes_height..=notes_height)
            .frame(no_frame)
            .show(&ctx, |mut ui| {
                let result =
                    self.render_scene
                        .draw(state, &mut ui, &key_view, &mut self.midi_file, note_speed.clone());
                render_result_data = Some(result);
            });

        let render_result_data = render_result_data.unwrap();

        let mut polyphony: usize = 0;

        // Render the keyboard
        egui::TopBottomPanel::top("Keyboard panel")
            .height_range(keyboard_height..=keyboard_height)
            .frame(no_frame)
            .show(&ctx, |ui| {
                let pressed = self.keyboard
                    .draw(ui, &key_view, &render_result_data.key_colors);
                polyphony += pressed;
            });

        // Render the stats
        let stats_frame = Frame::default()
            .margin(egui::style::Margin::same(10.0))
            .fill(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 170))
            .rounding(egui::Rounding::same(5.0));

        egui::Window::new("Stats")
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .scroll2([false, false])
            .enabled(true)
            .frame(stats_frame)
            .fixed_pos(egui::Pos2::new(10.0, panel_height + 30.0))
            .show(&ctx, |ui| {
                if let Some(length) = self.midi_file.midi_length() {
                    let time = self.midi_file.timer().get_time().as_secs();
                    let time_sec = time % 60;
                    let time_min = (time / 60) % 60;
                    let length_u64 = length as u64;
                    let length_sec = length_u64 % 60;
                    let length_min = (length_u64 / 60) % 60;
                    if time > length_u64 {
                        ui.add(Label::new(format!("Time: {:0width$}:{:0width$}/{:0width$}:{:0width$}", length_min, length_sec, length_min, length_sec, width = 2)));
                    } else {
                        ui.add(Label::new(format!("Time: {:0width$}:{:0width$}/{:0width$}:{:0width$}", time_min, time_sec, length_min, length_sec, width = 2)));
                    }
                }
                ui.add(Label::new(format!("FPS: {}", self.fps.get_fps().round())));
                ui.add(Label::new(format!("Total Notes: {}", stats.total_notes)));
                ui.add(Label::new(format!("Passed: {}", -1)));  // TODO
                ui.add(Label::new(format!("Polyphony: {}", polyphony)));
                ui.add(Label::new(format!("Rendered: {}", render_result_data.notes_rendered)));
            });
    }
}
