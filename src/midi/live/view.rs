#![allow(dead_code)]

use gen_iter::GenIter;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::midi::{DisplacedMIDINote, MIDIColor, MIDINoteColumnView, MIDINoteViews, MIDIViewRange};

use super::{column::LiveNoteColumn, parse::LiveMidiParser};

pub struct LiveNoteViewData {
    parser: LiveMidiParser,
    columns: Vec<LiveNoteColumn>,
    default_track_colors: Vec<MIDIColor>,
    view_range: MIDIViewRange,
}

pub struct LiveCurrentNoteViews<'a> {
    data: &'a LiveNoteViewData,
}

impl<'a> LiveCurrentNoteViews<'a> {
    pub fn new(data: &'a LiveNoteViewData) -> Self {
        LiveCurrentNoteViews { data }
    }
}

impl LiveNoteViewData {
    pub fn new(parser: LiveMidiParser, track_count: usize, random_colors: bool) -> Self {
        let mut columns = Vec::with_capacity(256);
        columns.resize_with(256, LiveNoteColumn::new);
        LiveNoteViewData {
            parser,
            columns,
            view_range: MIDIViewRange {
                start: 0.0,
                end: 0.0,
            },
            default_track_colors: if random_colors {
                MIDIColor::new_random_vec_for_tracks(track_count)
            } else {
                MIDIColor::new_vec_for_tracks(track_count)
            },
        }
    }

    pub fn shift_view_range(&mut self, new_view_range: MIDIViewRange) {
        if self.view_range.start > new_view_range.start {
            panic!("Can't shift live loaded view range backwards");
        }

        self.view_range = new_view_range;

        // Update columns from the parser queue
        for block in self.parser.recieve_next_note_blocks() {
            let column = &mut self.columns[block.key as usize];
            column.blocks.push_back(block.block);
        }

        self.columns.par_iter_mut().for_each(|column| {
            if column.blocks.is_empty() {
                return;
            }

            let blocks = &mut column.blocks;
            let data = &mut column.data;

            // Move the last block value up until the first block outside view range
            while data.end_block < blocks.len() {
                let block = &blocks[data.end_block];
                if block.start > new_view_range.end {
                    break;
                }
                data.rendered_notes += block.notes.len();
                data.end_block += 1;
            }

            while data.blocks_passed_keyboard_index < blocks.len() {
                if blocks[data.blocks_passed_keyboard_index].start > new_view_range.start {
                    break;
                }
                data.notes_passed_keyboard +=
                    blocks[data.blocks_passed_keyboard_index].notes.len() as u64;
                data.blocks_passed_keyboard_index += 1;
            }

            while let Some(block) = blocks.front() {
                if block.max_end() < new_view_range.start {
                    data.rendered_notes -= block.notes.len();
                    data.blocks_passed_keyboard_index -= 1;
                    blocks.pop_front();

                    // Unconditionally reduce this value because blocks that have an
                    // end time below the view range start time are always behind a block that has
                    // a start time above the view range start time.
                    data.end_block -= 1;
                } else {
                    break;
                }
            }
        });
    }

    pub fn parse_time(&self) -> f64 {
        self.parser.parse_time()
    }

    pub fn passed_notes(&self) -> u64 {
        self.columns
            .iter()
            .map(|column| column.data.notes_passed_keyboard)
            .sum()
    }
}

pub struct LiveNoteColumnView<'a> {
    view: &'a LiveNoteViewData,
    column: &'a LiveNoteColumn,
    view_range: MIDIViewRange,
}

impl<'a> MIDINoteViews for LiveCurrentNoteViews<'a> {
    type View<'b> = LiveNoteColumnView<'b> where Self: 'a + 'b;

    fn get_column(&self, key: usize) -> Self::View<'_> {
        LiveNoteColumnView {
            view: self.data,
            column: &self.data.columns[key],
            view_range: self.data.view_range,
        }
    }

    fn range(&self) -> MIDIViewRange {
        self.data.view_range
    }
}

struct LiveNoteBlockIter<'a, Iter: Iterator<Item = DisplacedMIDINote>> {
    view: &'a LiveNoteColumnView<'a>,
    iter: Iter,
}

impl<'a> MIDINoteColumnView for LiveNoteColumnView<'a> {
    type Iter<'b> = impl 'b + ExactSizeIterator<Item = DisplacedMIDINote> where Self: 'b;

    fn iterate_displaced_notes(&self) -> Self::Iter<'_> {
        let colors = &self.view.default_track_colors;

        let iter = GenIter(move || {
            for block_index in (0..self.column.data.end_block).rev() {
                let block = &self.column.blocks[block_index];
                let start = (block.start - self.view_range.start) as f32;

                for note in block.notes.iter().rev() {
                    yield DisplacedMIDINote {
                        start,
                        len: note.len,
                        color: colors[note.track_chan.as_usize()],
                    };
                }
            }
        });

        LiveNoteBlockIter { view: self, iter }
    }
}

impl<Iter: Iterator<Item = DisplacedMIDINote>> Iterator for LiveNoteBlockIter<'_, Iter> {
    type Item = DisplacedMIDINote;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<Iter: Iterator<Item = DisplacedMIDINote>> ExactSizeIterator for LiveNoteBlockIter<'_, Iter> {
    fn len(&self) -> usize {
        self.view.column.data.rendered_notes
    }
}

impl Drop for LiveNoteViewData {
    fn drop(&mut self) {
        let data = std::mem::take(&mut self.columns);

        // Drop the columns in a separate thread because often it takes a long time
        std::thread::spawn(move || drop(data));
    }
}
