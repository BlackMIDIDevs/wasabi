use super::block::InRamNoteBlock;

pub struct InRamNoteColumn {
    pub blocks: Vec<InRamNoteBlock>,
}

impl InRamNoteColumn {
    pub fn new(blocks: Vec<InRamNoteBlock>) -> Self {
        InRamNoteColumn { blocks }
    }
}
