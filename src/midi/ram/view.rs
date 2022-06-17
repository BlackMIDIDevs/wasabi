use std::rc::Rc;

use gen_iter::GenIter;

use crate::midi::{
    DisplacedMIDINote, MIDINoteColumnView, MIDINoteViews, MIDINoteViewsBase, MIDIViewRange,
};

use super::column::InRamNoteColumn;

pub struct InRamNoteViews {
    columns: Rc<Vec<InRamNoteColumn>>,
    column_view_data: Vec<InRamNoteColumnViewData>,
    view_range: MIDIViewRange,
}

impl InRamNoteViews {
    pub fn new(columns: Rc<Vec<InRamNoteColumn>>) -> Self {
        let column_view_data = columns
            .iter()
            .map(InRamNoteColumnViewData::from_column)
            .collect();
        InRamNoteViews {
            columns,
            column_view_data,
            view_range: MIDIViewRange {
                start: 0.0,
                end: 0.0,
            },
        }
    }
}

pub struct InRamNoteColumnViewData {
    note_sum: usize,
}

impl InRamNoteColumnViewData {
    pub fn from_column(column: &InRamNoteColumn) -> Self {
        let note_count = column.blocks.iter().map(|b| b.notes.len()).sum();
        InRamNoteColumnViewData {
            note_sum: note_count,
        }
    }
}

pub struct InRamNoteColumnView<'a> {
    column: &'a InRamNoteColumn,
    data: &'a InRamNoteColumnViewData,
    view_range: MIDIViewRange,
}

impl MIDINoteViewsBase for InRamNoteViews {
    fn shift_view_range(&mut self, new_range: MIDIViewRange) {
        self.view_range = new_range;
    }
}

impl MIDINoteViews for InRamNoteViews {
    type View<'a> = InRamNoteColumnView<'a>;

    fn get_column<'a>(&'a self, key: usize) -> Self::View<'a> {
        InRamNoteColumnView {
            column: &self.columns[key],
            data: &self.column_view_data[key],
            view_range: self.view_range,
        }
    }
}

impl MIDINoteViews for &InRamNoteViews {
    type View<'a> = InRamNoteColumnView<'a> where Self: 'a;

    fn get_column<'a>(&'a self, key: usize) -> Self::View<'a> {
        InRamNoteColumnView {
            column: &self.columns[key],
            data: &self.column_view_data[key],
            view_range: self.view_range,
        }
    }
}

impl<'a> InRamNoteColumnView<'a> {}

struct InRamNoteBlockIter<'a, Iter: Iterator<Item = DisplacedMIDINote>> {
    view: &'a InRamNoteColumnView<'a>,
    iter: Iter,
}

impl<'a> MIDINoteColumnView for InRamNoteColumnView<'a> {
    type Iter<'b> = impl 'b + ExactSizeIterator<Item = DisplacedMIDINote> where Self: 'b;

    fn iterate_displaced_notes<'b>(&'b self) -> Self::Iter<'b> {
        let iter = GenIter(move || {
            for block in self.column.blocks.iter().rev() {
                let start = (block.start - self.view_range.start) as f32;

                for note in block.notes.iter().rev() {
                    yield DisplacedMIDINote {
                        start: start,
                        len: note.len,
                        track_chan: note.track_chan,
                    };
                }
            }
        });

        InRamNoteBlockIter {
            view: self,
            iter: iter.into_iter(),
        }
    }

    fn adjust_view_range(&mut self, range: MIDIViewRange) {
        self.view_range = range;
    }
}

impl<Iter: Iterator<Item = DisplacedMIDINote>> Iterator for InRamNoteBlockIter<'_, Iter> {
    type Item = DisplacedMIDINote;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Iter: Iterator<Item = DisplacedMIDINote>> ExactSizeIterator for InRamNoteBlockIter<'_, Iter> {
    fn len(&self) -> usize {
        self.view.data.note_sum
    }
}
