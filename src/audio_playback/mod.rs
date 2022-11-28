use kdmapi::{KDMAPIStream, KDMAPI};
mod xsynth;

pub enum AudioPlayerType {
    XSynth{ buffer: f64 },
    Kdmapi,
}

pub struct SimpleTemporaryPlayer {
    player_type: AudioPlayerType,
    xsynth: Option<xsynth::XSynthPlayer>,
    kdmapi: Option<KDMAPIStream>,
}

impl SimpleTemporaryPlayer {
    pub fn new(player_type: AudioPlayerType) -> Self {
        let (xsynth, kdmapi) = match player_type {
            AudioPlayerType::XSynth{ buffer: buf } => {
                let xsynth = xsynth::XSynthPlayer::new(buf);
                (Some(xsynth), None)
            }
            AudioPlayerType::Kdmapi => {
                let kdmapi = KDMAPI.open_stream();
                (None, Some(kdmapi))
            }
        };
        Self {
            player_type,
            xsynth,
            kdmapi,
        }
    }

    pub fn switch_player(&mut self, player_type: AudioPlayerType) {
        self.reset();
        self.xsynth = None;
        self.kdmapi = None;
        let new_player = Self::new(player_type);

        self.player_type = new_player.player_type;
        self.xsynth = new_player.xsynth;
        self.kdmapi = new_player.kdmapi;
    }

    pub fn get_voice_count(&self) -> u64 {
        match self.player_type {
            AudioPlayerType::XSynth{..} => {
                if let Some(xsynth) = &self.xsynth {
                    xsynth.get_voice_count()
                } else {
                    0
                }
            }
            AudioPlayerType::Kdmapi => 0,
        }
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for e in data {
            self.push_event(e);
        }
    }

    pub fn push_event(&mut self, data: u32) {
        match self.player_type {
            AudioPlayerType::XSynth{..} => {
                if let Some(xsynth) = self.xsynth.as_mut() {
                    xsynth.push_event(data)
                }
            }
            AudioPlayerType::Kdmapi => {
                if let Some(kdmapi) = self.kdmapi.as_mut() {
                    kdmapi.send_direct_data(data);
                }
            }
        }
    }

    pub fn reset(&mut self) {
        match self.player_type {
            AudioPlayerType::XSynth{..} => {
                if let Some(xsynth) = self.xsynth.as_mut() {
                    xsynth.reset()
                }
            }
            AudioPlayerType::Kdmapi => {
                if let Some(kdmapi) = self.kdmapi.as_mut() {
                    kdmapi.reset()
                }
            }
        }
    }

    pub fn set_layer_count(&mut self, layers: Option<usize>) {
        if let AudioPlayerType::XSynth{..} = self.player_type {
            if let Some(xsynth) = self.xsynth.as_mut() {
                xsynth.set_layer_count(layers)
            }
        }
    }

    pub fn set_soundfont(&mut self, path: &str) {
        if let AudioPlayerType::XSynth{..} = self.player_type {
            if let Some(xsynth) = self.xsynth.as_mut() {
                xsynth.set_soundfont(path)
            }
        }
    }
}
