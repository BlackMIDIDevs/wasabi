use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::midi::ram::block::InRamNoteBlock;

use self::{column::InRamNoteColumn, view::InRamNoteViews};

use super::{MIDIFile, MIDIFileBase};

use midi_toolkit::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    io::MIDIFile as TKMIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, convert_events_into_batches, scale_event_time},
        unwrap_items, TimeCaster,
    },
};

pub mod block;
pub mod column;
pub mod view;

pub struct InRamMIDIFile {
    columns: Arc<Vec<InRamNoteColumn>>,
    track_count: usize,
}

impl InRamMIDIFile {
    pub fn load_from_file(path: &str) -> Self {
        struct UnendedNote {
            column_index: usize,
            block_index: usize,
        }

        struct Key {
            column: Vec<InRamNoteBlock>,
            block_builder: Vec<u32>,
            unended_notes: HashMap<u32, VecDeque<UnendedNote>>,
        }

        impl Key {
            fn new() -> Self {
                Key {
                    column: Vec::new(),
                    block_builder: Vec::new(),
                    unended_notes: HashMap::new(),
                }
            }

            fn add_note(&mut self, track_chan: u32) {
                let block_index = self.block_builder.len();
                let column_index = self.column.len();
                self.block_builder.push(track_chan);
                let unended_queue = self
                    .unended_notes
                    .entry(track_chan)
                    .or_insert_with(VecDeque::new);
                unended_queue.push_back(UnendedNote {
                    column_index,
                    block_index,
                });
            }

            pub fn end_note(&mut self, track_chan: u32, time: f64) {
                let note = self
                    .unended_notes
                    .get_mut(&track_chan)
                    .and_then(|unended_queue| unended_queue.pop_front());

                if let Some(note) = note {
                    if note.column_index == self.column.len() {
                        // Note is zero length
                        let index = self
                            .block_builder
                            .iter()
                            .position(|x| *x == track_chan)
                            .unwrap();
                        self.block_builder.remove(index);
                    } else {
                        self.column[note.column_index].set_note_end_time(note.block_index, time);
                    }
                }
            }

            pub fn flush(&mut self, time: f64) {
                if self.block_builder.len() > 0 {
                    let block =
                        InRamNoteBlock::new_from_trackchans(time, self.block_builder.drain(..));
                    self.column.push(block);
                }
            }

            pub fn end_all(&mut self, time: f64) {
                for (_, mut queue) in self.unended_notes.drain() {
                    for note in queue.drain(..) {
                        self.column[note.column_index].set_note_end_time(note.block_index, time);
                    }
                }
            }
        }

        let midi = TKMIDIFile::open(path, None).unwrap();

        let ppq = midi.ppq();
        let merged = pipe!(
            midi.iter_all_track_events_merged()
            |>TimeCaster::<f64>::cast_event_delta()
            |>cancel_tempo_events(250000)
            |>convert_events_into_batches()
            |>scale_event_time(1.0 / ppq as f64)
            |>unwrap_items()
        );

        let mut keys: Vec<Key> = (0..256).map(|_| Key::new()).collect();

        let mut time = 0.0;

        fn flush_keys(time: f64, keys: &mut Vec<Key>) {
            for key in keys.iter_mut() {
                key.flush(time);
            }
        }

        for batch in merged {
            if batch.delta() > 0.0 {
                flush_keys(time, &mut keys);
                time += batch.delta();
            }

            for event in batch.into_iter() {
                let track = event.track;
                match event.as_event() {
                    Event::NoteOn(e) => {
                        let track_chan = track * 16 + e.channel as u32;
                        keys[e.key as usize].add_note(track_chan);
                    }
                    Event::NoteOff(e) => {
                        let track_chan = track * 16 + e.channel as u32;
                        keys[e.key as usize].end_note(track_chan, time);
                    }
                    _ => {}
                }
            }
        }

        flush_keys(time, &mut keys);

        for key in keys.iter_mut() {
            key.end_all(time);
        }

        InRamMIDIFile {
            columns: Arc::new(
                keys.into_iter()
                    .map(|key| InRamNoteColumn::new(key.column))
                    .collect(),
            ),
            track_count: midi.track_count(),
        }
    }
}

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
