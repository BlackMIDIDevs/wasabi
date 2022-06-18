use kdmapi::{KDMAPIStream, KDMAPI};

pub struct SimpleTemporaryPlayer {
    kdmapi: KDMAPIStream,
}

impl SimpleTemporaryPlayer {
    pub fn new() -> Self {
        let kdmapi = KDMAPI.open_stream();
        SimpleTemporaryPlayer { kdmapi }
    }

    pub fn push_events(&self, data: impl Iterator<Item = u32>) {
        for e in data {
            self.push_event(e);
        }
    }

    pub fn push_event(&self, data: u32) {
        self.kdmapi.send_direct_data(data);
    }
}
