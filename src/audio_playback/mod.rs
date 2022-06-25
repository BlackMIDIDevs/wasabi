use std::sync::Arc;

use kdmapi::{KDMAPIStream, KDMAPI};
use xsynth_core::{
    channel::ChannelEvent,
    soundfont::{SampleSoundfont, SoundfontBase},
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

        let soundfont: Arc<dyn SoundfontBase> = Arc::new(
            SampleSoundfont::new(
                "D:/Midis/Loud and Proud Remastered/Axley Presets/Loud and Proud Remastered.sfz",
                params.clone(),
            )
            .unwrap(),
        );

        sender.send_event(SynthEvent::AllChannels(ChannelEvent::SetSoundfonts(vec![
            soundfont,
        ])));

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
        self.sender.send_event_u32(data);
        // self.kdmapi.send_direct_data(data);
    }

    pub fn reset(&mut self) {
        self.sender.reset_synth();
        // self.kdmapi.reset();
    }
}
