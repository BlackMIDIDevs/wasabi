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
        let mut next_index = self.tree[0].length_marker_len();

        loop {
            let node = self.tree[next_index];

            let offset = if time < node.leaf_cutoff() as u32 {
                node.leaf_left()
            } else {
                node.leaf_right()
            };

            if offset > 0 {
                next_index -= offset as usize;
                break;
            }
            let offset = -offset;
            next_index -= offset as usize;
        }

        let note = self.tree[next_index];

        if note.is_note_empty() {
            None
        } else {
            Some(CakeNoteData {
                start_time: note.note_start(),
                end_time: note.note_end(),
                color: MIDIColor::from_u32(note.note_color()),
            })
        }
    }
    pub fn get_notes_passed_at(&self, time: i32) -> u32 {
        let mut last_notes_passed;
        let mut next_index = self.tree[0].length_marker_len();

        loop {
            let node = self.tree[next_index];

            let offset = if time < node.leaf_cutoff() {
                node.leaf_left()
            } else {
                node.leaf_right()
            };

            last_notes_passed = node.leaf_notes_to_the_left();

            if offset > 0 {
                break;
            }
            let offset = -offset;
            next_index -= offset as usize;
        }

        last_notes_passed
    }
}
