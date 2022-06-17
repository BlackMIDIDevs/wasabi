use std::rc::Rc;

use self::{column::InRamNoteColumn, view::InRamNoteViews};

use super::{MIDIFile, MIDIFileBase};

pub mod block;
pub mod column;
pub mod view;

pub struct InRamMIDIFile {
    columns: Rc<Vec<InRamNoteColumn>>,
}

impl InRamMIDIFile {
    pub fn new_dummy_data(notes_per_block: usize) -> Self {
        let mut columns = Vec::new();
        for _ in 0..256 {
            columns.push(InRamNoteColumn::new_dummy_data(notes_per_block));
        }
        InRamMIDIFile {
            columns: Rc::new(columns),
        }
    }
}

impl MIDIFileBase for InRamMIDIFile {
    fn allows_seeking_backward(&self) -> bool {
        false
    }

    fn midi_length(&self) -> Option<f64> {
        None
    }

    fn parsed_up_to(&self) -> Option<f64> {
        None
    }
}

impl MIDIFile for InRamMIDIFile {
    type ColumnsViews = InRamNoteViews;

    fn get_column_views<'a>(&'a self) -> Self::ColumnsViews {
        InRamNoteViews::new(self.columns.clone())
    }
}
