use std::{
    ops::{Deref, DerefMut, RangeInclusive},
    path::Path,
    sync::Arc,
};

use crate::WasabiSettings;

use xsynth_core::{
    channel::{ChannelConfigEvent, ChannelEvent, ChannelInitOptions},
    soundfont::{EnvelopeOptions, SampleSoundfont, SoundfontBase, SoundfontInitOptions},
    AudioStreamParams,
};
use xsynth_realtime::{
    RealtimeEventSender, RealtimeSynth, RealtimeSynthStatsReader, SynthEvent, ThreadCount,
    XSynthRealtimeConfig,
};

#[repr(transparent)]
struct FuckYouImSend<T>(T);

unsafe impl<T> Sync for FuckYouImSend<T> {}
unsafe impl<T> Send for FuckYouImSend<T> {}

impl<T> Deref for FuckYouImSend<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FuckYouImSend<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct XSynthPlayer {
    sender: RealtimeEventSender,
    pub stats: RealtimeSynthStatsReader,
    stream_params: AudioStreamParams,
    _synth: FuckYouImSend<RealtimeSynth>,
}

impl XSynthPlayer {
    pub fn new(
        buffer: f64,
        use_threadpool: bool,
        ignore_range: RangeInclusive<u8>,
        options: ChannelInitOptions,
    ) -> Self {
        let config = XSynthRealtimeConfig {
            render_window_ms: buffer,
            channel_init_options: options,
            ignore_range,
            multithreading: if use_threadpool {
                ThreadCount::Auto
            } else {
                ThreadCount::None
            },
            ..Default::default()
        };

        let synth = FuckYouImSend(RealtimeSynth::open_with_default_output(config));
        let sender = synth.get_senders();
        let stream_params = synth.stream_params();
        let stats = synth.get_stats();

        XSynthPlayer {
            sender,
            stats,
            stream_params,
            _synth: synth,
        }
    }

    pub fn get_voice_count(&self) -> u64 {
        self.stats.voice_count()
    }

    pub fn push_event(&mut self, data: u32) {
        self.sender.send_event_u32(data);
    }

    pub fn reset(&mut self) {
        self.sender.reset_synth();
    }

    pub fn set_layer_count(&mut self, layers: Option<usize>) {
        self.sender
            .send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                ChannelConfigEvent::SetLayerCount(layers),
            )));
    }

    pub fn set_soundfont(&mut self, path: &str, options: SoundfontInitOptions) {
        if !path.is_empty() && Path::new(path).exists() {
            let samplesf = SampleSoundfont::new(path, self.stream_params, options);
            if let Ok(sf) = samplesf {
                let soundfont: Arc<dyn SoundfontBase> = Arc::new(sf);
                self.sender
                    .send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                        ChannelConfigEvent::SetSoundfonts(vec![soundfont]),
                    )));
            }
        }
    }
}

pub fn convert_to_sf_init(settings: &WasabiSettings) -> SoundfontInitOptions {
    SoundfontInitOptions {
        vol_envelope_options: EnvelopeOptions::default(),
        use_effects: settings.synth.use_effects,
        ..Default::default()
    }
}

pub fn convert_to_channel_init(settings: &WasabiSettings) -> ChannelInitOptions {
    ChannelInitOptions {
        fade_out_killing: settings.synth.fade_out_kill,
    }
}
