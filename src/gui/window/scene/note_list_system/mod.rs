mod notes_render_pass;

use std::{cell::UnsafeCell, sync::Arc};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use vulkano::image::view::ImageView;

use crate::{
    gui::{window::keyboard_layout::KeyboardView, GuiRenderer},
    midi::{DisplacedMIDINote, MIDIColor, MIDIFile, MIDINoteColumnView, MIDINoteViews},
    utils,
};

use self::notes_render_pass::{NotePassStatus, NoteRenderPass, NoteVertex};

use super::RenderResultData;

pub struct NoteRenderer {
    render_pass: NoteRenderPass,
    thrad_pool: rayon::ThreadPool,
}

struct UnsafeSyncCell<T>(UnsafeCell<T>);

impl<T> UnsafeSyncCell<T> {
    pub fn new(value: T) -> Self {
        UnsafeSyncCell(UnsafeCell::new(value))
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self) -> &mut T {
        &mut *self.0.get()
    }
}

unsafe impl<T> Sync for UnsafeSyncCell<T> {}
unsafe impl<T> Send for UnsafeSyncCell<T> {}

impl NoteRenderer {
    pub fn new(renderer: &GuiRenderer) -> NoteRenderer {
        NoteRenderer {
            render_pass: NoteRenderPass::new(renderer),
            thrad_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub fn draw(
        &mut self,
        key_view: &KeyboardView,
        final_image: Arc<ImageView>,
        midi_file: &mut impl MIDIFile,
        view_range: f64,
    ) -> RenderResultData {
        let note_views = midi_file.get_current_column_views(view_range);

        struct ColumnViewInfo<Iter: ExactSizeIterator<Item = DisplacedMIDINote> + Send> {
            offset: usize,
            iter: Iter,
            key: u8,
            remaining: usize,
            color: Option<MIDIColor>,
            border_width: f32,
        }

        let mut total_notes = 0;

        let columns: Vec<_> = (0..256).map(|i| note_views.get_column(i)).collect();

        let mut columns_view_info = Vec::new();

        let border_width = utils::calculate_border_width(
            final_image.image().extent()[0] as f32,
            key_view.visible_range.len() as f32,
        );

        // Black keys first
        for (i, column) in columns.iter().enumerate() {
            if key_view.key(i).black {
                let iter = column.iterate_displaced_notes();
                let length = iter.len();
                columns_view_info.push(ColumnViewInfo {
                    offset: total_notes,
                    iter,
                    key: i as u8,
                    remaining: length,
                    color: None,
                    border_width,
                });
                total_notes += length;
            }
        }

        // Then white keys after
        for (i, column) in columns.iter().enumerate() {
            if !key_view.key(i).black {
                let iter = column.iterate_displaced_notes();
                let length = iter.len();
                columns_view_info.push(ColumnViewInfo {
                    offset: total_notes,
                    iter,
                    key: i as u8,
                    remaining: length,
                    color: None,
                    border_width,
                });
                total_notes += length;
            }
        }

        let mut notes_pushed = 0;

        let mut cycle = 0;

        let view_range = note_views.range().length() as f32;

        self.render_pass
            .draw(final_image, key_view, view_range, |buffer| {
                let buffer_length = buffer.len() as usize;

                let buffer_writer = UnsafeSyncCell::new(buffer.write().unwrap());

                // A system to write multiple note columns into 1 large allocated array in parallel
                let written_notes = self.thrad_pool.install(|| {
                    // For each note column, write it into the buffer
                    let written_notes_per_key =
                        columns_view_info.par_iter_mut().rev().map(|column| {
                            if column.remaining == 0 {
                                return 0;
                            }

                            let offset =
                                (column.offset as i64 - notes_pushed as i64).max(0) as usize;

                            if offset >= buffer_length {
                                return 0;
                            }

                            let remaining_buffer_space = buffer_length - offset;
                            let iter_length = column.remaining;

                            let allowed_to_write = if iter_length > remaining_buffer_space {
                                remaining_buffer_space
                            } else {
                                iter_length
                            };

                            unsafe {
                                let buffer = buffer_writer.get_mut();

                                for i in 0..allowed_to_write {
                                    let next_note = column.iter.next();
                                    if let Some(note) = next_note {
                                        buffer[i + offset] = NoteVertex::new(
                                            note.start,
                                            note.len,
                                            column.key,
                                            note.color.as_u32(),
                                            column.border_width as u32,
                                        );

                                        if note.start <= 0.0
                                            && column.color.is_none()
                                            && note.start + note.len > 0.0
                                        {
                                            column.color = Some(note.color);
                                        }
                                    } else {
                                        panic!("Invalid iterator length");
                                    }
                                }
                            }

                            column.remaining -= allowed_to_write;

                            allowed_to_write
                        });

                    written_notes_per_key.sum::<usize>()
                });

                notes_pushed += written_notes;

                cycle += 1;

                if notes_pushed >= total_notes {
                    NotePassStatus::Finished {
                        remaining: written_notes as u32,
                    }
                } else {
                    NotePassStatus::HasMoreNotes
                }
            });

        // Sort for output metrics
        columns_view_info.sort_unstable_by_key(|k| k.key);

        RenderResultData {
            notes_rendered: notes_pushed as u64,
            key_colors: columns_view_info
                .iter()
                .map(|column| column.color)
                .collect(),
        }
    }
}
