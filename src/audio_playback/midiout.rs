use std::thread;

use super::*;
use crossbeam_channel::Sender;
use midir::MidiOutput;

pub struct MidiDevicePlayer {
    sender: Sender<u32>,
}

impl MidiDevicePlayer {
    pub fn new(device: String) -> Result<Self, String> {
        let out = MidiOutput::new("wasabi").map_err(|e| format!("{:?}", e))?;
        let ports = out.ports();
        if ports.is_empty() {
            return Err("No MIDI devices available.".into());
        }

        let find = ports.iter().find(|d| {
            if let Ok(name) = out.port_name(d) {
                name == device
            } else {
                false
            }
        });
        let found = find.unwrap_or(&ports[0]);
        let mut connection = out
            .connect(found, "wasabi")
            .map_err(|e| format!("{:?}", e))?;

        let (sender, receiver) = crossbeam_channel::bounded::<u32>(1000);

        thread::spawn(move || {
            for data in receiver {
                let message = data.to_le_bytes();
                connection.send(&message).unwrap_or_default();
            }
        });

        Ok(Self { sender })
    }
}

impl MidiAudioPlayer for MidiDevicePlayer {
    fn reset(&mut self) {
        // Send "All sounds off" and "Reset controllers" to all channels
        for ch in 0..16 {
            let code: u32 = 0xB << 4 | ch;
            for cc in [120, 121] {
                let z = 0 << 8;
                let cc = cc << 8 | z;
                let cc = cc | code;
                self.push_event(cc);
            }
        }
    }

    fn push_event(&mut self, data: u32) {
        self.sender.send(data).unwrap();
    }

    fn voice_count(&self) -> u64 {
        0
    }

    fn configure(&mut self, _settings: &SynthSettings) {}

    fn set_soundfonts(
        &mut self,
        _soundfonts: &Vec<WasabiSoundfont>,
        _loading_status: Arc<LoadingStatus>,
    ) {
    }
}
