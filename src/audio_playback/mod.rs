use std::{path::Path, sync::Arc};

//use kdmapi::{KDMAPIStream, KDMAPI};
use xsynth_core::{
    channel::ChannelConfigEvent,
    soundfont::{SampleSoundfont, SoundfontBase},
    AudioStreamParams,
};
use xsynth_realtime::{
    config::XSynthRealtimeConfig, RealtimeEventSender, RealtimeSynth, RealtimeSynthStatsReader,
};

pub struct SimpleTemporaryPlayer {
    //kdmapi: KDMAPIStream,
    sender: RealtimeEventSender,
    pub stats: RealtimeSynthStatsReader,
    stream_params: AudioStreamParams,
}

impl SimpleTemporaryPlayer {
    pub fn new(sfz_path: &str, buffer: f64) -> Self {
        let config = XSynthRealtimeConfig {
            render_window_ms: buffer,
            use_threadpool: false,
            ..Default::default()
        };

        let synth = RealtimeSynth::open_with_default_output(config);
        let mut sender = synth.get_senders();

        let stream_params = synth.stream_params().clone();

        if !sfz_path.is_empty() && Path::new(sfz_path).exists() {
            let samplesf = SampleSoundfont::new(sfz_path, stream_params.clone());
            if let Ok(sf) = samplesf {
                let soundfont: Arc<dyn SoundfontBase> = Arc::new(sf);
                sender.send_config(ChannelConfigEvent::SetSoundfonts(vec![soundfont]));
            }
        }

        sender.send_config(ChannelConfigEvent::SetLayerCount(Some(4)));

        let stats = synth.get_stats();

        // FIXME: Basically I'm leaking a pointer because the synth can't be sent between
        // threads and I really cbb making a synth state manager rn
        Box::leak(Box::new(synth));

        //let kdmapi = KDMAPI.open_stream();
        SimpleTemporaryPlayer { sender, stats, stream_params: stream_params.clone() }
    }

    pub fn get_voice_count(&self) -> u64 {
        self.stats.voice_count()
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for e in data {
            self.push_event(e);
        }
    }

    pub fn push_event(&mut self, data: u32) {
        self.sender.send_event_u32(data);
        // self.kdmapi.send_direct_data(data);
    }

    pub fn reset(&mut self) {
        self.sender.reset_synth();
        // self.kdmapi.reset();
    }

    pub fn set_soundfont(&mut self, path: &str) {
        if !path.is_empty() && Path::new(path).exists() {
            let samplesf = SampleSoundfont::new(path, self.stream_params.clone());
            if let Ok(sf) = samplesf {
                let soundfont: Arc<dyn SoundfontBase> = Arc::new(sf);
                self.sender.send_config(ChannelConfigEvent::SetSoundfonts(vec![soundfont]));
            }
        }
    }
}
