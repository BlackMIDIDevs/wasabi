use std::sync::Arc;

use kdmapi::{KDMAPIStream, KDMAPI};
use xsynth_core::{
    channel::ChannelConfigEvent,
    soundfont::{SampleSoundfont, SoundfontBase},
};
use xsynth_realtime::{config::XSynthRealtimeConfig, RealtimeEventSender, RealtimeSynth};

pub struct SimpleTemporaryPlayer {
    //kdmapi: KDMAPIStream,
    sender: RealtimeEventSender,
}

impl SimpleTemporaryPlayer {
    pub fn new() -> Self {
        let config = XSynthRealtimeConfig {
            render_window_ms: 0.5,
            use_threadpool: true,
            ..Default::default()
        };

        let synth = RealtimeSynth::open_with_default_output(config);
        let mut sender = synth.get_senders();

        let params = synth.stream_params();

        let soundfont: Arc<dyn SoundfontBase> = Arc::new(
            SampleSoundfont::new("D:/Midis/Loud and Proud Remastered/Kaydax Presets/Loud and Proud Remastered (Realistic).sfz", params.clone()).unwrap(),
        );

        sender.send_config(ChannelConfigEvent::SetSoundfonts(vec![soundfont]));
        sender.send_config(ChannelConfigEvent::SetLayerCount(Some(4)));

        // FIXME: Basically I'm leaking a pointer because the synth can't be sent between
        // threads and I really cbb making a synth state manager rn
        Box::leak(Box::new(synth));

        //let kdmapi = KDMAPI.open_stream();
        SimpleTemporaryPlayer { sender }
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
