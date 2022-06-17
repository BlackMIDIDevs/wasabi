use std::sync::Arc;

use self::{column::InRamNoteColumn, view::InRamNoteViews};

use super::{MIDIFile, MIDIFileBase};

pub mod block;
pub mod column;
mod parse;
pub mod view;

pub struct InRamMIDIFile {
    columns: Arc<Vec<InRamNoteColumn>>,
    track_count: usize,
}

impl InRamMIDIFile {}

impl MIDIFileBase for InRamMIDIFile {
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
        InRamNoteViews::new(self.columns.clone(), self.track_count)
    }
}
