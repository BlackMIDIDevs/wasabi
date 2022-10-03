use std::{path::Path, sync::Arc};

//use kdmapi::{KDMAPIStream, KDMAPI};
use xsynth_core::{
    channel::ChannelConfigEvent,
    soundfont::{SampleSoundfont, SoundfontBase},
};
use xsynth_realtime::{
    config::XSynthRealtimeConfig, RealtimeEventSender, RealtimeSynth, RealtimeSynthStatsReader,
};

pub struct SimpleTemporaryPlayer {
    //kdmapi: KDMAPIStream,
    sender: RealtimeEventSender,
    pub stats: RealtimeSynthStatsReader,
}

impl SimpleTemporaryPlayer {
    pub fn new(sfz_path: &str) -> Self {
        let config = XSynthRealtimeConfig {
            render_window_ms: 1000.0 / 60.0,
            use_threadpool: true,
            ..Default::default()
        };

        let synth = RealtimeSynth::open_with_default_output(config);
        let mut sender = synth.get_senders();

        let params = synth.stream_params();

        if !sfz_path.is_empty() && Path::new(sfz_path).exists() {
            let samplesf = SampleSoundfont::new(sfz_path, params.clone());
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
        SimpleTemporaryPlayer { sender, stats }
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
}
