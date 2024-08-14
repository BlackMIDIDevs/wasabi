use std::{cell::UnsafeCell, ops::Deref, sync::Arc};

use crate::midi::shared::track_channel::TrackAndChannel;

pub struct LiveNoteBlock {
    pub start: f64,
    pub max_length: f32,
    pub notes: Box<[LiveMIDINote]>,

    unended_notes: u32,
}

#[derive(Debug, Clone)]
pub struct LiveMIDINote {
    pub len: f32,
    pub track_chan: TrackAndChannel,
}

impl LiveNoteBlock {
    /// Creates a new block from an iterator of Track/Channel values.
    /// This assumes that the lengths will be added in the future.
    pub fn new_from_trackchans(
        time: f64,
        track_chans_iter: impl ExactSizeIterator<Item = TrackAndChannel>,
    ) -> Self {
        let mut notes: Vec<LiveMIDINote> = Vec::with_capacity(track_chans_iter.len());

        for track_chan in track_chans_iter {
            notes.push(LiveMIDINote {
                len: f32::INFINITY,
                track_chan,
            });
        }

        LiveNoteBlock {
            unended_notes: notes.len() as u32,
            start: time,
            notes: notes.into_boxed_slice(),
            max_length: 0.0,
        }
    }

    pub fn set_note_end_time(&mut self, note_index: usize, end_time: f64) {
        let note = &mut self.notes[note_index];
        note.len = (end_time - self.start) as f32;
        self.max_length = self.max_length.max(note.len);
    }

    pub fn max_end(&self) -> f64 {
        if self.unended_notes == 0 {
            self.start + self.max_length as f64
        } else {
            f64::INFINITY
        }
    }
}

pub struct LiveRefNoteBlock(Arc<UnsafeCell<LiveNoteBlock>>);

unsafe impl Send for LiveRefNoteBlock {}
unsafe impl Sync for LiveRefNoteBlock {}

impl LiveRefNoteBlock {
    pub fn new_from_trackchans(
        time: f64,
        track_chans_iter: impl ExactSizeIterator<Item = TrackAndChannel>,
    ) -> (
        Self,
        impl ExactSizeIterator<Item = LiveNoteEnderHandleWithTrackChan>,
    ) {
        #[allow(clippy::arc_with_non_send_sync)]
        let block = LiveRefNoteBlock(Arc::new(UnsafeCell::new(
            LiveNoteBlock::new_from_trackchans(time, track_chans_iter),
        )));

        let inner_cloned = block.0.clone();

        let iter = unsafe {
            (*block.0.get())
                .notes
                .iter()
                .enumerate()
                .map(move |(i, note)| LiveNoteEnderHandleWithTrackChan {
                    handle: LiveNoteEnderHandle {
                        block: inner_cloned.clone(),
                        index: i as u32,
                    },
                    track_chan: note.track_chan,
                })
        };

        (block, iter)
    }

    pub fn max_end(&self) -> f64 {
        unsafe { (*self.0.get()).max_end() }
    }
}

impl Deref for LiveRefNoteBlock {
    type Target = LiveNoteBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

pub struct LiveNoteEnderHandleWithTrackChan {
    pub handle: LiveNoteEnderHandle,
    pub track_chan: TrackAndChannel,
}

pub struct LiveNoteEnderHandle {
    block: Arc<UnsafeCell<LiveNoteBlock>>,
    index: u32,
}

unsafe impl Send for LiveNoteEnderHandle {}

impl LiveNoteEnderHandle {
    pub fn end(&mut self, end_time: f64) {
        unsafe {
            let block = &mut (*self.block.get());
            block.set_note_end_time(self.index as usize, end_time);
            block.unended_notes -= 1;
        }
    }
}
