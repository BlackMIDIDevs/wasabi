use std::collections::VecDeque;

use super::block::LiveRefNoteBlock;

pub struct InRamNoteColumnViewData {
    pub rendered_notes: usize,

    /// Exclusive end block for when notes go outside of view.
    /// We iterate over notes backwards, so we start at the block before this one and iterate to 0.
    pub end_block: usize,

    /// The number of blocks that have passed the keyboard in the current blocks vec
    pub blocks_passed_keyboard_index: usize,
    /// The number of notes that have passed the keyboard overall
    pub notes_passed_keyboard: u64,
}

impl InRamNoteColumnViewData {
    pub fn new() -> Self {
        InRamNoteColumnViewData {
            rendered_notes: 0,
            end_block: 0,
            blocks_passed_keyboard_index: 0,
            notes_passed_keyboard: 0,
        }
    }
}

pub struct LiveNoteColumn {
    pub blocks: VecDeque<LiveRefNoteBlock>,
    pub data: InRamNoteColumnViewData,
}

impl LiveNoteColumn {
    pub fn new() -> Self {
        LiveNoteColumn {
            blocks: VecDeque::new(),
            data: InRamNoteColumnViewData::new(),
        }
    }
}
