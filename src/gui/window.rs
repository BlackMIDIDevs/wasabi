mod fps;
mod keyboard;
mod keyboard_layout;
mod scene;
mod stats;

mod about;
mod playback_panel;
mod settings;
mod shortcuts;

use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::thread;

use crossbeam_channel::{Receiver, Sender};
use egui::FontFamily::{Monospace, Proportional};
use egui::FontId;
use egui::Frame;
use settings::SettingsWindow;
use time::Duration;

use crate::audio_playback::{EmptyPlayer, MidiDevicePlayer};
use crate::{
    audio_playback::{KdmapiPlayer, MidiAudioPlayer, WasabiAudioPlayer, XSynthPlayer},
    gui::window::{keyboard::GuiKeyboard, scene::GuiRenderScene},
    midi::{CakeMIDIFile, InRamMIDIFile, LiveLoadMIDIFile, MIDIFileBase, MIDIFileUnion},
    settings::{MidiParsing, Synth, WasabiSettings},
    state::WasabiState,
    GuiRenderer, GuiState,
};

const WIN_MARGIN: egui::Margin = egui::Margin::same(12.0);
const SPACE: f32 = 12.0;

pub struct GuiWasabiWindow {
    render_scene: GuiRenderScene,
    keyboard_layout: keyboard_layout::KeyboardLayout,
    keyboard: GuiKeyboard,
    midi_file: Option<MIDIFileUnion>,
    synth: Arc<RwLock<WasabiAudioPlayer>>,
    fps: fps::Fps,

    settings_win: SettingsWindow,
    midi_picker: (Sender<PathBuf>, Receiver<PathBuf>),
}

impl GuiWasabiWindow {
    pub fn new(renderer: &mut GuiRenderer, settings: &mut WasabiSettings) -> GuiWasabiWindow {
        let synth = Self::create_synth(settings);
        let synth = Arc::new(RwLock::new(WasabiAudioPlayer::new(synth)));

        let mut settings_win = SettingsWindow::new(settings);
        settings_win.load_palettes(settings);
        settings_win.load_midi_devices(settings);

        let midi_picker = crossbeam_channel::unbounded();

        GuiWasabiWindow {
            render_scene: GuiRenderScene::new(renderer),
            keyboard_layout: keyboard_layout::KeyboardLayout::new(&Default::default()),
            keyboard: GuiKeyboard::new(),
            midi_file: None,
            synth,
            fps: fps::Fps::new(),

            settings_win,
            midi_picker,
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
            style.spacing.interact_size.y = 26.0;
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
            (egui::TextStyle::Heading, FontId::new(26.0, Proportional)),
            (egui::TextStyle::Body, FontId::new(16.0, Proportional)),
            (egui::TextStyle::Monospace, FontId::new(12.0, Monospace)),
            (egui::TextStyle::Button, FontId::new(16.0, Proportional)),
            (egui::TextStyle::Small, FontId::new(12.0, Proportional)),
            (
                egui::TextStyle::Name("monospace big".into()),
                FontId::new(22.0, Monospace),
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

        let fps_limit = match settings.gui.fps_limit {
            0 => None,
            f => Some(f),
        };
        self.fps.update(fps_limit);
        Self::set_style(&ctx, settings);

        {
            let recv = self.midi_picker.1.clone();
            if !recv.is_empty() {
                for midi in recv {
                    state.last_location = midi.clone();
                    self.load_midi(settings, midi);
                    break;
                }
            }
        }

        // Other windows
        if state.show_settings {
            self.settings_win
                .show(&ctx, settings, state, self.synth.clone());
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
                }
            }
        });

        // Render the panel
        let height_prev = ctx.available_rect().height();
        self.show_playback_panel(&ctx, settings, state);

        // Calculate available space left for keyboard and notes
        // We must render notes before keyboard because the notes
        // renderer tells us the key colors
        let available = ctx.available_rect();
        let height = available.height();
        let panel_height = height_prev - height;
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
                    let skip_dur = Duration::seconds_f64(settings.gui.skip_control);
                    let time = midi_file.timer().get_time();

                    // Set playback keyboard shortcuts
                    ui.input(|events| {
                        for event in &events.events {
                            if let egui::Event::Key {
                                key,
                                pressed,
                                modifiers,
                                ..
                            } = event
                            {
                                if pressed == &true {
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
                                            if modifiers.ctrl {
                                                settings.scene.note_speed +=
                                                    settings.gui.speed_control;
                                            }
                                        }
                                        egui::Key::ArrowDown => {
                                            if modifiers.ctrl {
                                                settings.scene.note_speed -=
                                                    settings.gui.speed_control;
                                            }
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
                            midi_file.timer_mut().seek(Duration::seconds_f64(length));
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
            let voice_count = self.synth.read().unwrap().voice_count();
            stats.set_voice_count(voice_count);

            let pad = if settings.scene.statistics.floating {
                12.0
            } else {
                0.0
            };
            let pos = egui::Pos2::new(pad, panel_height + pad);
            stats::draw_stats(self, &ctx, pos, stats, settings);
        }
    }

    pub fn open_midi_dialog(&mut self, state: &mut WasabiState) {
        let sender = self.midi_picker.0.clone();
        let last_location = state.last_location.clone();

        thread::spawn(move || {
            let midi_path = rfd::FileDialog::new()
                .add_filter("mid", &["mid", "MID"])
                .set_directory(last_location.parent().unwrap_or(Path::new("./")))
                .pick_file();

            if let Some(midi_path) = midi_path {
                sender.send(midi_path).unwrap_or_default();
            }
        });
    }

    pub fn load_midi(&mut self, settings: &mut WasabiSettings, midi_path: PathBuf) {
        // TODO: Load in thread
        if let Some(midi_file) = self.midi_file.as_mut() {
            midi_file.timer_mut().pause();
        }
        self.synth.write().unwrap().reset();
        self.midi_file = None;

        if let Some(midi_path) = midi_path.to_str() {
            match settings.midi.parsing {
                MidiParsing::Ram => {
                    let mut midi_file = MIDIFileUnion::InRam(InRamMIDIFile::load_from_file(
                        midi_path,
                        self.synth.clone(),
                        settings,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
                MidiParsing::Live => {
                    let mut midi_file = MIDIFileUnion::Live(LiveLoadMIDIFile::load_from_file(
                        midi_path,
                        self.synth.clone(),
                        settings,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
                MidiParsing::Cake => {
                    let mut midi_file = MIDIFileUnion::Cake(CakeMIDIFile::load_from_file(
                        midi_path,
                        self.synth.clone(),
                        settings,
                    ));
                    midi_file.timer_mut().play();
                    self.midi_file = Some(midi_file);
                }
            }
        }
    }

    pub fn create_synth(settings: &WasabiSettings) -> Box<dyn MidiAudioPlayer> {
        let mut synth: Box<dyn MidiAudioPlayer> = match settings.synth.synth {
            Synth::XSynth => Box::new(XSynthPlayer::new(settings.synth.xsynth.config.clone())),
            Synth::Kdmapi => Box::new(KdmapiPlayer::new()),
            Synth::MidiDevice => {
                if let Ok(midiout) = MidiDevicePlayer::new(settings.synth.midi_device.clone()) {
                    Box::new(midiout)
                } else {
                    Box::new(EmptyPlayer::new())
                }
            }
            Synth::None => Box::new(EmptyPlayer::new()),
        };
        synth.set_soundfonts(&settings.synth.soundfonts);
        synth.configure(&settings.synth);

        synth
    }
}
