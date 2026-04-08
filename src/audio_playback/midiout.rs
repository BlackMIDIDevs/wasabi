use std::thread;

use crate::gui::window::WasabiError;

use crossbeam_channel::Sender;
use midir::MidiOutput;

pub struct MidiDevicePlayer {
    sender: Sender<u32>,
}

impl MidiDevicePlayer {
    pub fn new(device: String) -> Result<Self, WasabiError> {
        let out = MidiOutput::new("wasabi")
            .map_err(|e| WasabiError::SynthError(format!("MIDI Out Error: {e}")))?;
        let ports = out.ports();
        if ports.is_empty() {
            return Err(WasabiError::SynthError("No MIDI devices available.".into()));
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
            .map_err(|e| WasabiError::SynthError(format!("MIDI Out Error: {e}")))?;

        let (sender, receiver) = crossbeam_channel::bounded::<u32>(1000);

        thread::spawn(move || {
            for data in receiver {
                let message = data.to_le_bytes();
                connection.send(&message).unwrap_or_default();
            }
        });

        Ok(Self { sender })
    }

    pub fn reset(&mut self) {
        self.sender.send(0xFF).unwrap();
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        for ev in data {
            // Remove port data
            let ev = ev & 0x00FFFFFF;
            self.sender.send(ev).unwrap();
        }
    }
}
