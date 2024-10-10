use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
    thread,
};

use crate::{
    gui::window::{LoadingType, WasabiError},
    settings::WasabiSoundfont,
};

use xsynth_core::{
    channel::{ChannelConfigEvent, ChannelEvent},
    soundfont::{SampleSoundfont, SoundfontBase},
    AudioStreamParams,
};
use xsynth_realtime::{
    RealtimeEventSender, RealtimeSynth, RealtimeSynthStatsReader, SynthEvent, XSynthRealtimeConfig,
};

use super::*;

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
    stats: RealtimeSynthStatsReader,
    stream_params: AudioStreamParams,
    _synth: FuckYouImSend<RealtimeSynth>,
}

impl XSynthPlayer {
    pub fn new(config: XSynthRealtimeConfig) -> Self {
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
}

impl MidiAudioPlayer for XSynthPlayer {
    fn voice_count(&self) -> Option<u64> {
        Some(self.stats.voice_count())
    }

    fn push_event(&mut self, data: u32) {
        self.sender.send_event_u32(data);
    }

    fn reset(&mut self) {
        self.sender.reset_synth();
    }

    fn configure(&mut self, settings: &SynthSettings) {
        let layers = if settings.xsynth.limit_layers {
            Some(settings.xsynth.layers)
        } else {
            None
        };

        self.sender
            .send_event(SynthEvent::AllChannels(ChannelEvent::Config(
                ChannelConfigEvent::SetLayerCount(layers),
            )));
    }

    fn set_soundfonts(
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
