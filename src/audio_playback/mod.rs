use std::sync::Arc;

use kdmapi::{KDMAPIStream, KDMAPI};
use xsynth_core::{
    channel::ChannelEvent,
    soundfont::{SoundfontBase, SquareSoundfont},
};
use xsynth_realtime::{RealtimeEventSender, RealtimeSynth, SynthEvent};

pub struct SimpleTemporaryPlayer {
    kdmapi: KDMAPIStream,
    sender: RealtimeEventSender,
}

impl SimpleTemporaryPlayer {
    pub fn new() -> Self {
        let synth = RealtimeSynth::open_with_all_defaults();
        let mut sender = synth.get_senders();

        let params = synth.stream_params();

        let soundfonts: Vec<Arc<dyn SoundfontBase>> = vec![Arc::new(SquareSoundfont::new(
            params.sample_rate,
            params.channels,
        ))];

        sender.send_event(SynthEvent::AllChannels(ChannelEvent::SetSoundfonts(
            soundfonts,
        )));

        // FIXME: Basically I'm leaking a pointer because the synth can't be sent between
        // threads and I really cbb making a synth state manager rn
        Box::leak(Box::new(synth));

        let kdmapi = KDMAPI.open_stream();
        SimpleTemporaryPlayer { kdmapi, sender }
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for e in data {
            self.push_event(e);
        }
    }

    pub fn push_event(&mut self, data: u32) {
        // self.sender.send_event_u32(data);
        self.kdmapi.send_direct_data(data);
    }

    pub fn reset(&mut self) {
        // xsynth can't reset at the moment
        self.kdmapi.reset();
    }
}
