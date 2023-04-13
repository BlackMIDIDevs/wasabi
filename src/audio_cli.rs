use std::io::{stdout, BufWriter, Write};
use std::panic::PanicInfo;
use std::path::PathBuf;
use std::time::Duration;

use clap::error::ErrorKind;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{self, Attribute};
use crossterm::QueueableCommand;

use crate::audio_playback::xsynth::{convert_to_channel_init, convert_to_sf_init};
use crate::audio_playback::{AudioPlayerType, ManagedSynth};
use crate::midi::MIDIFileBase;
use crate::settings::{Synth, WasabiSettings};

pub fn run_audio_cli(settings: &mut WasabiSettings) {
    if settings.synth.sfz_path.is_empty() {
        let err = clap::Error::raw(
            ErrorKind::MissingRequiredArgument,
            "The following was not provided through the config file, \
            or the command line\n\t-s, --sfz-path <sfz-path>",
        )
        .with_cmd(&clap::Command::new("wasabi"));

        err.exit();
    }
    let mut synth = ManagedSynth::new(settings);

    let file: PathBuf = settings.load_midi_file.take().unwrap().into();
    synth.load_midi(settings, file);

    let mut stdout = BufWriter::new(stdout().lock());

    stdout
        .queue(style::SetAttribute(Attribute::Bold))
        .unwrap()
        .queue(style::SetAttribute(Attribute::Underlined))
        .unwrap()
        .write_all(b"\nKeyboard Controls:")
        .unwrap();

    stdout
        .queue(style::ResetColor)
        .unwrap()
        .write_all(
            br#"

Space: Play/Pause
Left Arrow: -10s
Right Arrow: +10s
Down Arrow: -60s
Up Arrow: +60s
R: Reload Synthesizer
Q/Ctrl+c: Quit

            "#,
        )
        .unwrap();

    stdout.flush().unwrap();

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info: &PanicInfo| {
        crossterm::terminal::disable_raw_mode().unwrap();
        default_hook(info);
    }));

    crossterm::terminal::enable_raw_mode().unwrap();

    loop {
        if let Some(midi_file) = synth.midi_file.as_mut() {
            let mut time = midi_file.timer().get_time().as_secs();
            let length = midi_file.midi_length().unwrap() as u64;

            if event::poll(Duration::from_secs(1)).unwrap() {
                match event::read().unwrap() {
                    Event::Key(KeyEvent {
                        code: KeyCode::Left,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        if midi_file.allows_seeking_backward() {
                            if time > 10 {
                                time -= 10;
                            } else {
                                time = 0;
                            }
                            midi_file.timer_mut().seek(Duration::from_secs(time));
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Down,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        if midi_file.allows_seeking_backward() {
                            if time > 60 {
                                time -= 60;
                            } else {
                                time = 0;
                            }
                            midi_file.timer_mut().seek(Duration::from_secs(time));
                        }
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Right,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        if (time + 10) < length {
                            time += 10;
                        } else {
                            time = length;
                        }
                        midi_file.timer_mut().seek(Duration::from_secs(time));
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Up,
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        if (time + 60) < length {
                            time += 60;
                        } else {
                            time = length;
                        }
                        midi_file.timer_mut().seek(Duration::from_secs(time));
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char(' '),
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        midi_file.timer_mut().toggle_pause();
                    }
                    Event::Key(
                        KeyEvent {
                            code: KeyCode::Char('c'),
                            kind: KeyEventKind::Press,
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        }
                        | KeyEvent {
                            code: KeyCode::Char('q'),
                            kind: KeyEventKind::Press,
                            ..
                        },
                    ) => {
                        crossterm::terminal::disable_raw_mode().unwrap();
                        return;
                    }
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('r'),
                        kind: KeyEventKind::Press,
                        ..
                    }) => {
                        match settings.synth.synth {
                            Synth::XSynth => {
                                synth.player.write().unwrap().switch_player(
                                    AudioPlayerType::XSynth {
                                        buffer: settings.synth.buffer_ms,
                                        ignore_range: settings.synth.vel_ignore.clone(),
                                        options: convert_to_channel_init(settings),
                                    },
                                );
                                synth.player.write().unwrap().set_soundfont(
                                    &settings.synth.sfz_path,
                                    convert_to_sf_init(settings),
                                );
                                synth.player.write().unwrap().set_layer_count(
                                    if settings.synth.limit_layers {
                                        Some(settings.synth.layer_count)
                                    } else {
                                        None
                                    },
                                );
                            }
                            Synth::Kdmapi => {
                                synth
                                    .player
                                    .write()
                                    .unwrap()
                                    .switch_player(AudioPlayerType::Kdmapi);
                            }
                        }
                    }
                    _ => {}
                }
            }

            stdout
                .queue(crossterm::terminal::Clear(
                    crossterm::terminal::ClearType::CurrentLine,
                ))
                .unwrap()
                .queue(crossterm::cursor::MoveToColumn(0))
                .unwrap()
                .write_fmt(format_args!(
                    "[{:02}:{:02}/{:02}:{:02}]",
                    time / 60,
                    time % 60,
                    length / 60,
                    length % 60
                ))
                .unwrap();

            stdout.flush().unwrap();
        }
    }
}
