use super::{block::InRamNoteBlock, view::InRamNoteColumnView};

pub struct InRamNoteColumn {
    pub blocks: Vec<InRamNoteBlock>,
}

impl InRamNoteColumn {
    pub fn new_dummy_data(notes_per_block: usize) -> Self {
        let mut notes = Vec::new();
        for i in 0..10 {
            notes.push(InRamNoteBlock::new_dummy_data(
                0.07 * i as f64,
                notes_per_block,
            ));
        }
        InRamNoteColumn { blocks: notes }
    }
}
