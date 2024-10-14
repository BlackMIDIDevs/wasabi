pub mod fps;
mod keyboard;
mod keyboard_layout;
mod scene;
mod stats;

mod about;
mod errors;
mod loading;
mod playback_panel;
mod settings;
mod shortcuts;
pub use errors::*;

use std::path::Path;
use std::path::PathBuf;
use std::thread;

use egui::FontFamily::{Monospace, Proportional};
use egui::FontId;
use egui::Frame;
pub use loading::*;
use settings::SettingsWindow;
use time::Duration;
use tokio::sync::{oneshot, oneshot::Receiver};

use crate::{
    gui::{
        window::{keyboard::GuiKeyboard, scene::GuiRenderScene},
        GuiRenderer, GuiState,
    },
    midi::{CakeMIDIFile, InRamMIDIFile, LiveLoadMIDIFile, MIDIFileBase, MIDIFileUnion},
    settings::{MidiParsing, WasabiSettings},
    state::WasabiState,
    utils::NOTE_SPEED_RANGE,
};

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: Option<MIDIFileUnion>,
    fps: fps::Fps,

    settings_win: SettingsWindow,
    midi_picker: Option<Receiver<PathBuf>>,
    midi_loader: Option<Receiver<MIDIFileUnion>>,
}

impl GuiWasabiWindow {
    pub fn new(
        renderer: &mut GuiRenderer,
        settings: &mut WasabiSettings,
        state: &WasabiState,
    ) -> GuiWasabiWindow {
        let mut settings_win = SettingsWindow::new(settings);
        settings_win
            .load_palettes(settings)
            .unwrap_or_else(|e| state.errors.warning(e.to_string()));
        settings_win
            .load_midi_devices(settings)
            .unwrap_or_else(|e| state.errors.warning(e.to_string()));

        state.synth.switch(
            &settings.synth,
            state.loading_status.clone(),
            state.errors.clone(),
        );

        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file: None,
            fps: fps::Fps::new(),

            settings_win,
            midi_picker: None,
            midi_loader: None,
        }
    }

    #[inline(always)]
    fn set_style(ctx: &egui::Context, _settings: &WasabiSettings) {
        // Set theme
        ctx.style_mut(|style| {
            style.visuals.panel_fill = egui::Color32::from_rgb(18, 18, 18);
            style.visuals.window_fill = style.visuals.panel_fill;
            style.visuals.widgets.inactive.weak_bg_fill = style.visuals.panel_fill;
            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;

            style.visuals.selection.bg_fill = style.visuals.widgets.active.weak_bg_fill;
            style.visuals.selection.stroke.color = egui::Color32::TRANSPARENT;

            style.visuals.override_text_color = Some(egui::Color32::from_rgb(210, 210, 210));

            style.spacing.menu_margin = egui::Margin::same(8.0);
            style.spacing.interact_size.y = 22.0;
        });

        // Set fonts
        let mut fonts = egui::FontDefinitions::default();

        fonts.font_data.insert(
            "poppins".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/Poppins-Medium.ttf")),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "poppins".to_owned());

        fonts.font_data.insert(
            "ubuntu".to_owned(),
            egui::FontData::from_static(include_bytes!("../../assets/UbuntuSansMono-Medium.ttf")),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "ubuntu".to_owned());

        ctx.set_fonts(fonts);

        // Set font size
        let mut style = (*ctx.style()).clone();
        style.text_styles = [
            (egui::TextStyle::Heading, FontId::new(22.0, Proportional)),
            (egui::TextStyle::Body, FontId::new(14.0, Proportional)),
            (egui::TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (egui::TextStyle::Button, FontId::new(14.0, Proportional)),
            (egui::TextStyle::Small, FontId::new(10.0, Proportional)),
            (
                egui::TextStyle::Name("monospace big".into()),
                FontId::new(20.0, Monospace),
            ),
        ]
        .into();
        ctx.set_style(style);
    }

    /// Defines the layout of our UI
    pub fn layout(
        &mut self,
        gui_state: &mut GuiState,
        settings: &mut WasabiSettings,
        state: &mut WasabiState,
    ) {
        let ctx = gui_state.renderer.gui.context();
        Self::set_style(&ctx, settings);

        // Check for MIDIs selected by the file picker
        if let Some(recv) = self.midi_picker.as_mut() {
            if let Ok(midi) = recv.try_recv() {
                state.last_midi_location = midi.clone();
                self.load_midi(midi, settings, state);
                self.midi_picker = None;
            }
        }

        // Check for MIDIs parsed by the MIDI loader and play
        if let Some(recv) = self.midi_loader.as_mut() {
            if let Ok(mut midi) = recv.try_recv() {
                midi.timer_mut().play();
                self.midi_file = Some(midi);
                self.midi_loader = None;
            }
        }

        // If something is loading, pause playback and hide all windows
        if state.loading_status.is_loading() {
            if let Some(midi) = self.midi_file.as_mut() {
                midi.timer_mut().pause();
            }
            state.loading_status.show(&ctx);
            state.show_about = false;
            state.show_settings = false;
            state.show_shortcuts = false;
        }

        // Render windows
        if state.show_settings {
            self.settings_win.show(&ctx, settings, state);
        }

        if state.show_about {
            self.show_about(&ctx, state);
        }

        if state.show_shortcuts {
            self.show_shortcuts(&ctx, state);
        }

        // Set global keyboard shortcuts
        ctx.input(|events| {
            for event in &events.events {
                if let egui::Event::Key {
                    key,
                    pressed,
                    modifiers,
                    ..
                } = event
                {
                    if *pressed && modifiers.ctrl {
                        match key {
                            egui::Key::F => state.panel_pinned = !state.panel_pinned,
                            egui::Key::G => state.stats_visible = !state.stats_visible,
                            egui::Key::O => self.open_midi_dialog(state),
                            _ => {}
                        }
                    }
                    if *pressed && modifiers.alt && key == &egui::Key::Enter {
                        state.fullscreen = !state.fullscreen
                    }
                    if key == &egui::Key::Insert {
                        state.synth.reset();
                    }
                }
            }
        });

        // Render the panel
        let panel_height = self.show_playback_panel(&ctx, settings, state);

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let keyboard_height =
            (11.6 / settings.scene.key_range.len() as f32 * available.width()).min(height / 2.0);
        let notes_height = height - keyboard_height;

        let key_view = self.keyboard_layout.get_view_for_keys(
            *settings.scene.key_range.start() as usize,
            *settings.scene.key_range.end() as usize,
        );

        let no_frame = Frame::default()
            .inner_margin(egui::Margin::same(0.0))
            .fill(settings.scene.bg_color);

        let mut stats = stats::GuiMidiStats::empty();

        let mut render_result_data: Option<scene::RenderResultData> = None;

        // Render the notes
        egui::TopBottomPanel::top("Note panel")
            .height_range(notes_height..=notes_height)
            .frame(no_frame)
            .show_separator_line(false)
            .show(&ctx, |ui| {
                if let Some(midi_file) = self.midi_file.as_mut() {
                    // Set playback keyboard shortcuts
                    ui.input(|events| {
                        for event in &events.events {
                            if let egui::Event::Key { key, pressed, .. } = event {
                                if pressed == &true {
                                    let skip_dur = Duration::seconds_f64(settings.gui.skip_control);
                                    let time = midi_file.timer().get_time();

                                    match key {
                                        egui::Key::ArrowRight => {
                                            midi_file.timer_mut().seek(time + skip_dur)
                                        }
                                        egui::Key::ArrowLeft => {
                                            if midi_file.allows_seeking_backward() {
                                                midi_file.timer_mut().seek((time - skip_dur).max(
                                                    Duration::seconds_f64(
                                                        -settings.midi.start_delay,
                                                    ),
                                                ))
                                            }
                                        }
                                        egui::Key::ArrowUp => {
                                            settings.scene.note_speed = (settings.scene.note_speed
                                                + settings.gui.speed_control)
                                                .min(*NOTE_SPEED_RANGE.start());
                                        }
                                        egui::Key::ArrowDown => {
                                            settings.scene.note_speed = (settings.scene.note_speed
                                                - settings.gui.speed_control)
                                                .max(*NOTE_SPEED_RANGE.end());
                                        }
                                        egui::Key::Space => midi_file.timer_mut().toggle_pause(),
                                        _ => {}
                                    }
                                }
                            }
                        }
                    });

                    // If song is finished, pause
                    {
                        let length = midi_file.midi_length().unwrap_or(0.0);
                        let current = midi_file.timer().get_time().as_seconds_f64();
                        if current > length {
                            midi_file.timer_mut().pause();
                        }
                    }

                    let result = self.render_scene.draw(
                        gui_state,
                        ui,
                        &key_view,
                        midi_file,
                        settings.scene.note_speed,
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
                let colors = if let Some(data) = render_result_data {
                    data.key_colors
                } else {
                    vec![None; 256]
                };

                self.keyboard
                    .draw(ui, &key_view, &colors, &settings.scene.bar_color);
            });

        // Render the stats
        if state.stats_visible {
            let voice_count = state.synth.voice_count();
            stats.set_voice_count(voice_count);

            let pad = if settings.scene.statistics.floating {
                12.0
            } else {
                0.0
            };
            let pos = egui::Pos2::new(pad, panel_height + pad);
            self.draw_stats(&ctx, pos, stats, settings);
        }

        // Render errors
        state.errors.show(&ctx);

        let fps_limit = match settings.gui.fps_limit {
            0 => None,
            f => Some(f),
        };
        self.fps.set_limit(fps_limit);
        self.fps.update();
    }

    pub fn open_midi_dialog(&mut self, state: &mut WasabiState) {
        // Do not open if something is loading already
        if state.loading_status.is_loading() {
            return;
        }

        let (tx, rx) = oneshot::channel();
        self.midi_picker = Some(rx);
        let last_location = state.last_midi_location.clone();

        // Open the file picker in a thread so the main UI thread does not freeze
        // and send the selected path via crossbeam
        thread::spawn(move || {
            let midi_path = rfd::FileDialog::new()
                .add_filter("mid", &["mid", "MID"])
                .set_title("Pick a MIDI file...")
                .set_directory(last_location.parent().unwrap_or(Path::new("./")))
                .pick_file();

            if let Some(midi_path) = midi_path {
                tx.send(midi_path).unwrap_or_default();
            }
        });
    }

    pub fn load_midi(
        &mut self,
        midi_path: PathBuf,
        settings: &mut WasabiSettings,
        state: &WasabiState,
    ) {
        // Unload current MIDI to free resources while loading the new one
        if let Some(mut midi_file) = self.midi_file.take() {
            midi_file.timer_mut().pause();
        }

        let filename = midi_path.file_name().unwrap_or_default().to_os_string();

        state.loading_status.create(
            loading::LoadingType::Midi,
            format!("Parsing {:?}", filename),
        );

        let synth = state.synth.clone();
        let settings = settings.midi.clone();
        let loading_status = state.loading_status.clone();
        let errors = state.errors.clone();

        let (tx, rx) = oneshot::channel();
        self.midi_loader = Some(rx);

        // Load the MIDI in a thread so the UI doesn't freeze and send it
        // via crossbeam
        thread::spawn(move || {
            if let Some(midi_path) = midi_path.to_str() {
                match settings.parsing {
                    MidiParsing::Ram => {
                        match InRamMIDIFile::load_from_file(midi_path, synth, &settings) {
                            Ok(midi) => {
                                let midi_file = MIDIFileUnion::InRam(midi);
                                tx.send(midi_file).ok();
                            }
                            Err(e) => errors.error(&e),
                        }
                        loading_status.clear();
                    }
                    MidiParsing::Live => {
                        match LiveLoadMIDIFile::load_from_file(midi_path, synth, &settings) {
                            Ok(midi) => {
                                let midi_file = MIDIFileUnion::Live(midi);
                                tx.send(midi_file).ok();
                            }
                            Err(e) => errors.error(&e),
                        }
                        loading_status.clear();
                    }
                    MidiParsing::Cake => {
                        match CakeMIDIFile::load_from_file(midi_path, synth, &settings) {
                            Ok(midi) => {
                                let midi_file = MIDIFileUnion::Cake(midi);
                                tx.send(midi_file).ok();
                            }
                            Err(e) => errors.error(&e),
                        }
                        loading_status.clear();
                    }
                }
            }
        });
    }
}
