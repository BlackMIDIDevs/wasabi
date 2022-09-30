use std::ops::Range;

use super::block::InRamNoteBlock;

pub struct InRamNoteColumnViewData {
    pub notes_to_end: usize,
    pub notes_to_start: usize,
    pub block_range: Range<usize>,
}

impl InRamNoteColumnViewData {
    pub fn new() -> Self {
        InRamNoteColumnViewData {
            notes_to_end: 0,
            notes_to_start: 0,
            block_range: 0..0,
        }
    }
}

pub struct InRamNoteColumn {
    pub data: InRamNoteColumnViewData,
    pub blocks: Vec<InRamNoteBlock>,
}

impl InRamNoteColumn {
    pub fn new(blocks: Vec<InRamNoteBlock>) -> Self {
        InRamNoteColumn {
            blocks,
            data: InRamNoteColumnViewData::new(),
        }
    }
}
