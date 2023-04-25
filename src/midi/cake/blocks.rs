use crate::midi::{IntVector4, MIDIColor};

pub struct CakeBlock {
    pub start_time: u32,
    pub end_time: u32,
    pub tree: Vec<IntVector4>,
}

pub struct CakeNoteData {
    pub start_time: u32,
    pub end_time: u32,
    pub color: MIDIColor,
}

impl CakeBlock {
    pub fn get_note_at(&self, time: u32) -> Option<CakeNoteData> {
        let mut next_index = self.tree[0].val1;

        loop {
            let node = self.tree[next_index as usize];

            let offset = if time < node.val1 as u32 {
                node.val2
            } else {
                node.val3
            };

            if offset > 0 {
                next_index -= offset;
                break;
            }
            next_index += offset;
        }

        let note = self.tree[next_index as usize];

        if note.val3 == -1 {
            None
        } else {
            Some(CakeNoteData {
                start_time: note.val1 as u32,
                end_time: note.val2 as u32,
                color: MIDIColor::from_u32(note.val3 as u32),
            })
        }
    }

    pub fn get_notes_passed_at(&self, time: u32) -> u64 {
        return 0;
    }
}
