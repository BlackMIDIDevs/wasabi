use gen_iter::GenIter;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

use crate::midi::{DisplacedMIDINote, MIDIColor, MIDINoteColumnView, MIDINoteViews, MIDIViewRange};

use super::column::InRamNoteColumn;

pub struct InRamNoteViewData {
    columns: Vec<InRamNoteColumn>,
    default_track_colors: Vec<MIDIColor>,
    view_range: MIDIViewRange,
}

pub struct InRamCurrentNoteViews<'a> {
    data: &'a InRamNoteViewData,
}

impl<'a> InRamCurrentNoteViews<'a> {
    pub fn new(data: &'a InRamNoteViewData) -> Self {
        InRamCurrentNoteViews { data }
    }
}

impl InRamNoteViewData {
    pub fn new(columns: Vec<InRamNoteColumn>, colors: Vec<MIDIColor>) -> Self {
        InRamNoteViewData {
            columns,
            view_range: MIDIViewRange {
                start: 0.0,
                end: 0.0,
            },
            default_track_colors: colors,
        }
    }

    pub fn passed_notes(&self) -> u64 {
        self.columns
            .iter()
            .map(|column| column.data.notes_to_keyboard)
            .sum()
    }
}

impl InRamNoteViewData {
    pub fn shift_view_range(&mut self, new_view_range: MIDIViewRange) {
        let old_view_range = self.view_range;
        self.view_range = new_view_range;

        self.columns.par_iter_mut().for_each(|column| {
            if column.blocks.is_empty() {
                return;
            }

            let blocks = &column.blocks;
            let data = &mut column.data;

            let mut new_block_start = data.block_range.start;
            let mut new_block_end = data.block_range.end;

            if new_view_range.end > old_view_range.end {
                while new_block_end < blocks.len() {
                    let block = &blocks[new_block_end];
                    if block.start >= new_view_range.end {
                        break;
                    }
                    data.notes_to_render_end += block.notes.len() as u64;
                    new_block_end += 1;
                }
            } else if new_view_range.end < old_view_range.end {
                while new_block_end > 0 {
                    let block = &blocks[new_block_end - 1];
                    if block.start < new_view_range.end {
                        break;
                    }
                    data.notes_to_render_end -= block.notes.len() as u64;
                    new_block_end -= 1;
                }
            } else {
                // No change in view end
            }

            if new_view_range.start > old_view_range.start {
                // Increment the note view start
                while new_block_start < blocks.len() {
                    let block = &blocks[new_block_start];
                    if block.max_end() >= new_view_range.start {
                        break;
                    }
                    data.notes_to_render_start += block.notes.len() as u64;
                    new_block_start += 1;
                }

                // Increment the keyboard passed notes/blocks
                while data.blocks_to_keyboard < blocks.len() {
                    let block = &blocks[data.blocks_to_keyboard];
                    if block.start > new_view_range.start {
                        break;
                    }
                    data.notes_to_keyboard += block.notes.len() as u64;
                    data.blocks_to_keyboard += 1;
                }
            } else if new_view_range.start < old_view_range.start {
                // It is smaller, we have to start from the beginning
                data.notes_to_render_start = 0;
                new_block_start = 0;

                data.notes_to_keyboard = 0;
                data.blocks_to_keyboard = 0;

                // Increment both view start notes and keyboard notes until we reach view start cutoff
                while new_block_start < blocks.len() {
                    let block = &blocks[new_block_start];
                    if block.max_end() >= new_view_range.start {
                        break;
                    }
                    data.notes_to_render_start += block.notes.len() as u64;
                    new_block_start += 1;

                    data.notes_to_keyboard += block.notes.len() as u64;
                    data.blocks_to_keyboard += 1;
                }

                // Increment the remaining keyboard blocks
                while data.blocks_to_keyboard < blocks.len() {
                    let block = &blocks[data.blocks_to_keyboard];
                    if block.start > new_view_range.start {
                        break;
                    }
                    data.notes_to_keyboard += block.notes.len() as u64;
                    data.blocks_to_keyboard += 1;
                }
            } else {
                // No change in view start
            }

            data.block_range = new_block_start..new_block_end;
        });
    }
}

impl<'a> MIDINoteViews for InRamCurrentNoteViews<'a> {
    type View<'b> = InRamNoteColumnView<'b> where Self: 'a + 'b;

    fn get_column(&self, key: usize) -> Self::View<'_> {
        InRamNoteColumnView {
            view: self.data,
            column: &self.data.columns[key],
            view_range: self.data.view_range,
        }
    }

    fn range(&self) -> MIDIViewRange {
        self.data.view_range
    }
}

struct InRamNoteBlockIter<'a, Iter: Iterator<Item = DisplacedMIDINote>> {
    view: &'a InRamNoteColumnView<'a>,
    iter: Iter,
}

pub struct InRamNoteColumnView<'a> {
    view: &'a InRamNoteViewData,
    column: &'a InRamNoteColumn,
    view_range: MIDIViewRange,
}

impl<'a> MIDINoteColumnView for InRamNoteColumnView<'a> {
    type Iter<'b> = impl 'b + ExactSizeIterator<Item = DisplacedMIDINote> where Self: 'b;

    fn iterate_displaced_notes(&self) -> Self::Iter<'_> {
        let colors = &self.view.default_track_colors;

        let iter = GenIter(
            #[coroutine]
            move || {
                for block_index in self.column.data.block_range.clone().rev() {
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
            },
        );

        InRamNoteBlockIter { view: self, iter }
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
        let data = &self.view.column.data;
        (data.notes_to_render_end - data.notes_to_render_start) as usize
    }
}

impl Drop for InRamNoteViewData {
    fn drop(&mut self) {
        let data = std::mem::take(&mut self.columns);

        // Drop the columns in a separate thread because often it takes a long time
        std::thread::spawn(move || drop(data));
    }
}
