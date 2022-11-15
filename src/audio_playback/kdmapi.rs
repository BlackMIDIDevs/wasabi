use kdmapi::{KDMAPIStream, KDMAPI};

pub struct KDMAPIPlayer {
    kdmapi: KDMAPIStream,
}

impl KDMAPIPlayer {
    pub fn new() -> Self {
        let kdmapi = KDMAPI.open_stream();
        KDMAPIPlayer { kdmapi }
    }

    pub fn push_event(&mut self, data: u32) {
        self.kdmapi.send_direct_data(data);
    }

    pub fn reset(&mut self) {
        self.kdmapi.reset();
    }
}
