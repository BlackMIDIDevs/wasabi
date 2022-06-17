pub struct InRamNoteBlock {
    pub start: f64,
    pub notes: Vec<BasicMIDINote>,
}

#[derive(Debug, Clone)]
pub struct BasicMIDINote {
    pub len: f32,
    pub track_chan: u32,
}

impl InRamNoteBlock {
    pub fn new_dummy_data(time: f64, notes_per_block: usize) -> Self {
        let iter = [
            BasicMIDINote {
                len: 0.5,
                track_chan: 1,
            },
            BasicMIDINote {
                len: 0.3,
                track_chan: 3,
            },
            BasicMIDINote {
                len: 0.1,
                track_chan: 5,
            },
            BasicMIDINote {
                len: 0.0005,
                track_chan: 8,
            },
        ]
        .into_iter()
        .cycle()
        .take(notes_per_block);

        InRamNoteBlock {
            start: time,
            notes: iter.collect(),
        }
    }
}
