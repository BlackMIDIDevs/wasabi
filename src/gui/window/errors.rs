use std::{
    env::consts::{ARCH, OS},
    fmt,
    sync::{Arc, Mutex},
};

use egui::{Context, Id, OpenUrl, WidgetText};
use midi_toolkit::io::MIDILoadError;
use xsynth_core::soundfont::LoadSfError;

use crate::utils;

#[derive(Debug)]
pub enum WasabiError {
    MidiLoadError(MIDILoadError),
    SoundFontLoadError(LoadSfError),
    #[cfg(supported_os)]
    SynthError(String),
    FilesystemError(std::io::Error),
    SettingsError(String),
    UpdaterError(String),
    PaletteError(String),
    Other(String),
}

impl fmt::Display for WasabiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WasabiError::MidiLoadError(e) => match e {
                MIDILoadError::CorruptChunks => write!(f, "MIDI Load Error: Corrupt Chunks"),
                MIDILoadError::FilesystemError(fs) => {
                    write!(f, "MIDI Load Error: Filesystem Error ({fs})")
                }
                MIDILoadError::FileTooBig => write!(f, "MIDI Load Error: File Too Big"),
            },
            WasabiError::SoundFontLoadError(e) => write!(f, "Error Parsing SoundFont: {e}"),
            #[cfg(supported_os)]
            WasabiError::SynthError(e) => write!(f, "Synth Error: {e}"),
            WasabiError::FilesystemError(e) => write!(f, "Filesystem Error: {e}"),
            WasabiError::SettingsError(e) => write!(f, "Settings Error: {e}"),
            WasabiError::UpdaterError(e) => write!(f, "Update Error: {e}"),
            WasabiError::PaletteError(e) => write!(f, "Palette Load Error: {e}"),
            WasabiError::Other(e) => write!(f, "Unknown Error: {e}"),
        }
    }
}

enum MessageType {
    Warning,
    Error,
    NewUpdate(String),
}

struct GuiMessage {
    pub id: Id,
    pub visible: bool,
    pub errtype: MessageType,
    pub title: String,
    pub message: WidgetText,
}

pub struct GuiMessageSystem {
    errors: Mutex<Vec<GuiMessage>>,
}

impl GuiMessageSystem {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            errors: Mutex::new(Vec::new()),
        })
    }

    fn add(&self, error: GuiMessage) {
        self.errors.lock().unwrap().push(error);
    }

    pub fn warning(&self, message: impl Into<WidgetText>) {
        self.add(GuiMessage {
            id: Id::new(rand::random::<u64>()),
            visible: true,
            errtype: MessageType::Warning,
            title: "Warning".into(),
            message: message.into(),
        });
    }

    pub fn error(&self, error: &WasabiError) {
        self.add(GuiMessage {
            id: Id::new(rand::random::<u64>()),
            visible: true,
            errtype: MessageType::Error,
            title: "Error".into(),
            message: error.to_string().into(),
        });
    }

    pub fn new_update(&self, version: impl Into<String>) {
        let version: String = version.into();

        let filename = {
            let ext = if OS == "windows" { ".exe" } else { "" };
            format!("wasabi-{}-{}{}", OS, ARCH, ext)
        };
        let link = format!(
            "https://github.com/BlackMIDIDevs/wasabi/releases/download/{}/{}",
            version.clone(),
            filename
        );

        self.add(GuiMessage {
            id: Id::new(rand::random::<u64>()),
            visible: true,
            errtype: MessageType::NewUpdate(link),
            title: "Update Available".into(),
            message: format!(
                "A new update for Wasabi ({}) is available.\nWould you like to download it?",
                version
            )
            .into(),
        });
    }

    pub fn show(&self, ctx: &Context) {
        self.errors.lock().unwrap().retain(|m| m.visible);

        let frame = utils::create_window_frame(ctx);

        for message in self.errors.lock().unwrap().iter_mut() {
            egui::Window::new(&message.title)
                .id(message.id)
                .resizable(false)
                .collapsible(false)
                .frame(frame)
                .show(ctx, |ui| {
                    let image = match &message.errtype {
                        MessageType::Error => egui::include_image!("../../../assets/error.svg"),
                        MessageType::Warning => egui::include_image!("../../../assets/warning.svg"),
                        MessageType::NewUpdate(..) => {
                            egui::include_image!("../../../assets/info.svg")
                        }
                    };

                    ui.horizontal(|ui| {
                        ui.add(egui::Image::new(image).fit_to_exact_size([64.0, 64.0].into()));
                        ui.label(message.message.clone());
                    });

                    ui.separator();

                    match &message.errtype {
                        MessageType::NewUpdate(link) => ui.horizontal(|ui| {
                            ui.columns(2, |columns| {
                                columns[0].with_layout(
                                    egui::Layout::top_down(egui::Align::RIGHT),
                                    |ui| {
                                        if ui.button("\u{2705} Yes").clicked() {
                                            ctx.open_url(OpenUrl::new_tab(link));
                                            message.visible = false;
                                        }
                                    },
                                );
                                columns[1].with_layout(
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        if ui.button("\u{2716} No").clicked() {
                                            message.visible = false;
                                        }
                                    },
                                );
                            });
                        }),
                        _ => ui.vertical_centered(|ui| {
                            if ui.button("\u{2705} OK").clicked() {
                                message.visible = false;
                            }
                        }),
                    }
                });
        }
    }
}
