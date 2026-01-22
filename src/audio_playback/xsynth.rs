use std::{sync::Arc, thread};

use crate::{
    gui::window::{LoadingType, WasabiError},
    settings::{WasabiSoundfont, XSynthSettings},
};

use xsynth_core::{
    channel::{ChannelAudioEvent, ChannelConfigEvent, ChannelEvent, ControlEvent},
    soundfont::{SampleSoundfont, SoundfontBase},
    AudioStreamParams,
};
use xsynth_realtime::{
    RealtimeEventSender, RealtimeSynth, RealtimeSynthStatsReader, SynthEvent, XSynthRealtimeConfig,
};

use super::*;

pub struct XSynthPlayer {
    sender: RealtimeEventSender,
    stats: RealtimeSynthStatsReader,
    stream_params: AudioStreamParams,
    synth: RealtimeSynth,
    port_data: bool,
    num_ports: u32,
}

impl XSynthPlayer {
    pub fn new(settings: &XSynthSettings) -> Self {
        let ports = if settings.use_ports {
            settings.num_ports as u32
        } else {
            1
        };

        let config = XSynthRealtimeConfig {
            format: xsynth_realtime::SynthFormat::Custom {
                channels: ports * 16,
            },
            ..settings.config.clone()
        };
        let mut synth = RealtimeSynth::open_with_default_output(config);
        let sender = synth.get_sender_ref().clone();
        let stream_params = synth.stream_params();
        let stats = synth.get_stats();

        for i in 0..ports {
            let chan = i * 16 + 9;
            synth.send_event(SynthEvent::Channel(
                chan,
                ChannelEvent::Config(ChannelConfigEvent::SetPercussionMode(true)),
            ));
        }

        XSynthPlayer {
            sender,
            stats,
            stream_params,
            synth,
            port_data: settings.use_ports,
            num_ports: ports,
        }
    }

    pub fn voice_count(&self) -> u64 {
        self.stats.voice_count()
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for ev in data {
            let port = if self.port_data { (ev >> 24) & 0xFF } else { 0 };
            if port >= self.num_ports {
                continue;
            }

            let channel = 16 * port + (ev & 0xF);

            match ev & 0xF0 {
                0x80 => self.sender.send_event(SynthEvent::Channel(
                    channel,
                    ChannelEvent::Audio(ChannelAudioEvent::NoteOff {
                        key: (ev >> 8) as u8,
                    }),
                )),
                0x90 => self.sender.send_event(SynthEvent::Channel(
                    channel,
                    ChannelEvent::Audio(ChannelAudioEvent::NoteOn {
                        key: (ev >> 8) as u8,
                        vel: (ev >> 16) as u8,
                    }),
                )),
                0xB0 => self.sender.send_event(SynthEvent::Channel(
                    channel,
                    ChannelEvent::Audio(ChannelAudioEvent::Control(ControlEvent::Raw(
                        (ev >> 8) as u8,
                        (ev >> 16) as u8,
                    ))),
                )),
                0xC0 => {
                    self.sender.send_event(SynthEvent::Channel(
                        channel,
                        ChannelEvent::Audio(ChannelAudioEvent::ProgramChange((ev >> 8) as u8)),
                    ));
                }
                0xE0 => {
                    let value =
                        (((((ev >> 16) as u8) as i16) << 7) | ((ev >> 8) as u8) as i16) - 8192;
                    let value = value as f32 / 8192.0;
                    self.sender.send_event(SynthEvent::Channel(
                        channel,
                        ChannelEvent::Audio(ChannelAudioEvent::Control(
                            ControlEvent::PitchBendValue(value),
                        )),
                    ));
                }
                0xF0 => {
                    if ev == 0xFF {
                        self.sender.reset_synth();
                    }
                }
                _ => {}
            }
        }
    }

    pub fn reset(&mut self) {
        self.sender.reset_synth();
    }

    pub fn configure(&mut self, settings: &XSynthSettings) {
        let layers = if settings.limit_layers {
            Some(settings.layers)
        } else {
            None
        };
        self.sender
            .send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                ChannelConfigEvent::SetLayerCount(layers),
            )));

        self.synth.set_buffer(settings.config.render_window_ms);
        self.sender
            .set_ignore_range(settings.config.ignore_range.clone());
    }

    pub fn set_soundfonts(
        &mut self,
        soundfonts: &[WasabiSoundfont],
        loading_status: Arc<LoadingStatus>,
        errors: Arc<GuiMessageSystem>,
    ) {
        let mut sender = self.sender.clone();
        let soundfonts: Vec<WasabiSoundfont> = soundfonts.to_vec();
        let stream_params = self.stream_params;

        loading_status.create(LoadingType::SoundFont, Default::default());

        thread::spawn(move || {
            sender.send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                ChannelConfigEvent::SetSoundfonts(Vec::new()),
            )));

            let mut out: Vec<Arc<dyn SoundfontBase>> = Vec::new();

            for sf in soundfonts.iter().rev() {
                if sf.enabled {
                    loading_status.update_message(format!(
                        "Loading {:?}",
                        sf.path.file_name().unwrap_or_default()
                    ));

                    match SampleSoundfont::new(&sf.path, stream_params, sf.options) {
                        Ok(sf) => out.push(Arc::new(sf)),
                        Err(err) => errors.error(&WasabiError::SoundFontLoadError(err)),
                    }
                }
            }

            sender.send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                ChannelConfigEvent::SetSoundfonts(out),
            )));
            loading_status.clear();
        });
    }
}
