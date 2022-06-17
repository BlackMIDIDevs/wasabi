mod notes_render_pass;

use std::{cell::UnsafeCell, sync::Arc};

use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use vulkano::{buffer::TypedBufferAccess, image::ImageViewAbstract};

use crate::{
    gui::{window::keyboard_layout::KeyboardView, GuiRenderer},
    midi::{DisplacedMIDINote, MIDINoteColumnView, MIDINoteViews},
};

use self::notes_render_pass::{NotePassStatus, NoteRenderPass, NoteVertex};

pub struct ChikaraShaderTest {
    render_pass: NoteRenderPass,
    thrad_pool: rayon::ThreadPool,
}

struct UnsafeSyncCell<T>(UnsafeCell<T>);

impl<T> UnsafeSyncCell<T> {
    pub fn new(value: T) -> Self {
        UnsafeSyncCell(UnsafeCell::new(value))
    }

    pub unsafe fn get_mut(&self) -> &mut T {
        &mut *self.0.get()
    }
}

unsafe impl<T> Sync for UnsafeSyncCell<T> {}
unsafe impl<T> Send for UnsafeSyncCell<T> {}

impl ChikaraShaderTest {
    pub fn new(renderer: &GuiRenderer) -> ChikaraShaderTest {
        ChikaraShaderTest {
            render_pass: NoteRenderPass::new(renderer),
            thrad_pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub fn draw(
        &mut self,
        key_view: &KeyboardView,
        final_image: Arc<dyn ImageViewAbstract + 'static>,
        note_views: impl MIDINoteViews,
    ) {
        struct ColumnViewInfo<Iter: ExactSizeIterator<Item = DisplacedMIDINote> + Send> {
            offset: usize,
            iter: Iter,
            key: u8,
            remaining: usize,
        }

        let mut total_notes = 0;

        let columns: Vec<_> = (0..256).map(|i| note_views.get_column(i)).collect();

        let mut columns_view_info = Vec::new();

        // Add black keys first
        for (i, column) in columns.iter().enumerate() {
            if key_view.key(i).black {
                let iter = column.iterate_displaced_notes();
                let length = iter.len();
                columns_view_info.push(ColumnViewInfo {
                    offset: total_notes,
                    iter,
                    key: i as u8,
                    remaining: length,
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
                });
                total_notes += length;
            }
        }

        let mut notes_pushed = 0;

        let mut cycle = 0;

        self.render_pass.draw(final_image, key_view, |buffer| {
            let buffer_length = buffer.len() as usize;

            let buffer_writer = UnsafeSyncCell::new(buffer.write().unwrap());

            let written_notes = self.thrad_pool.install(|| {
                let written_notes_per_key = columns_view_info.par_iter_mut().map(|column| {
                    if column.remaining == 0 {
                        return 0;
                    }

                    let offset = (column.offset as i64 - notes_pushed as i64).max(0) as usize;

                    if offset >= buffer_length {
                        return 0;
                    }

                    let remaining_buffer_space = buffer_length - offset as usize;
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
                                buffer[i + offset] =
                                    NoteVertex::new(note.start, note.len, column.key, note.color);
                            } else {
                                panic!("Invalid iterator length");
                            }
                        }
                    }

                    column.remaining -= allowed_to_write;

                    return allowed_to_write;
                });

                written_notes_per_key.sum::<usize>()
            });

            notes_pushed += written_notes;

            cycle += 1;

            if notes_pushed >= total_notes {
                return NotePassStatus::Finished {
                    remaining: written_notes as u32,
                };
            } else {
                return NotePassStatus::HasMoreNotes;
            }
        });
    }
}
