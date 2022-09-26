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
        egui::TopBottomPanel::top("Top panel")
            .height_range(60.0..=60.0)
            .show(&ctx, |ui| {
                let one_sec = Duration::from_secs(1);
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

                /*if ui.small_button("Play/Pause").clicked() {
                    self.midi_file.timer_mut().toggle_pause();
                }

                let time = self.midi_file.timer().get_time();
                let five_sec = Duration::from_secs(5);
                if ui.small_button("Skip 5 sec").clicked() {
                    self.midi_file.timer_mut().seek(time + five_sec);
                }

                if ui.small_button("Back 5 sec").clicked() {
                    if time >= five_sec {
                        self.midi_file.timer_mut().seek(time - five_sec);
                    } else {
                        self.midi_file.timer_mut().seek(Duration::from_secs(0));
                    }
                }*/

                if let Some(length) = self.midi_file.midi_length() {
                    let time = self.midi_file.timer().get_time().as_secs_f64();
                    let mut progress = time / length;
                    let progress_prev = progress.clone();
                    if time > length { let time = length; }
                    let slider = egui::Slider::new(&mut progress, 0.0..=1.0)
                        .show_value(false);
                    ui.spacing_mut().slider_width = window_size[0] - 15.0;
                    ui.add(slider);
                    if progress_prev != progress {
                        let position = Duration::from_secs_f64(progress * length);
                        self.midi_file.timer_mut().seek(position);
                    }

                    ui.add(Label::new(format!("Time: {}/{}", time as u32, length as u32)));
                }
                ui.add(Label::new(format!("FPS: {}", self.fps.get_fps().round())));
            });

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let keyboard_height = 70.0 / 760.0 * available.width() as f32;
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(0, 127);

        let no_frame = Frame::default().margin(Margin::same(0.0));

        let mut render_result_data = None;

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

        // Render the keyboard
        egui::TopBottomPanel::top("Keyboard panel")
            .height_range(keyboard_height..=keyboard_height)
            .frame(no_frame)
            .show(&ctx, |ui| {
                self.keyboard
                    .draw(ui, &key_view, &render_result_data.key_colors);
            });
    }
}
