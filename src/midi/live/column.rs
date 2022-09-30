use std::collections::VecDeque;

use super::block::{LiveNoteBlock, LiveRefNoteBlock};

pub struct InRamNoteColumn {
    pub blocks: VecDeque<LiveRefNoteBlock>,
}

impl InRamNoteColumn {
    pub fn new() -> Self {
        InRamNoteColumn {
            blocks: VecDeque::new(),
        }
    }
}
