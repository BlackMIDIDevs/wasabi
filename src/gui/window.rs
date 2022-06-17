mod keyboard;
mod keyboard_layout;
mod scene;

use std::time::Instant;

use egui::{style::Margin, Frame, Slider, Visuals};

use crate::midi::{
    InRamMIDIFile, MIDIFileUnion, MIDIFileViewsUnion, MIDINoteViewsBase, MIDIViewRange,
};

use self::{keyboard::GuiKeyboard, scene::GuiRenderScene};

use super::{GuiRenderer, GuiState};

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: MIDIFileUnion,
    midi_file_views: MIDIFileViewsUnion,
    view_start: f64,
    view_length: f64,
    start: Instant,
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer) -> GuiWasabiWindow {
        let midi_file = MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
            "D:/Midis/The Nuker 4 F1/The Nuker 4 - F1 Part 13.mid",
        ));
        let midi_file_views = midi_file.get_views();

        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file_views,
            midi_file,
            view_start: 0.0,
            view_length: 10.0,
            start: Instant::now(),
        }
    }

    /// Defines the layout of our UI
    pub fn layout(&mut self, state: &mut GuiState) {
        let ctx = state.gui.context();

        ctx.set_visuals(Visuals::dark());

        // Render the top panel
        egui::TopBottomPanel::top("Top panel")
            .height_range(100.0..=100.0)
            .show(&ctx, |ui| {
                ui.add(Slider::new(&mut self.view_start, 0.0..=500.0).text("Start"));
                ui.add(Slider::new(&mut self.view_length, 0.0..=500.0).text("Length"));

                let time = self.start.elapsed().as_secs_f64();
                self.midi_file_views
                    .shift_view_range(MIDIViewRange::new(time, time + 0.25));
            });

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let keyboard_height = 100.0;
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(0, 127);

        let no_frame = Frame::default().margin(Margin::same(0.0));

        // Render the notes
        egui::TopBottomPanel::top("Note panel")
            .height_range(notes_height..=notes_height)
            .frame(no_frame)
            .show(&ctx, |mut ui| {
                self.render_scene
                    .draw(state, &mut ui, &key_view, &self.midi_file_views)
            });

        // Render the keyboard
        egui::TopBottomPanel::top("Keyboard panel")
            .height_range(keyboard_height..=keyboard_height)
            .frame(no_frame)
            .show(&ctx, |ui| {
                self.keyboard.draw(ui, &key_view);
            });
    }
}
