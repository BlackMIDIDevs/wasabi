use std::{collections::VecDeque, sync::Arc, thread};

use midi_toolkit::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    io::MIDIFile as TKMIDIFile,
    pipe,
    sequence::{
        event::{
            cancel_tempo_events, convert_events_into_batches, scale_event_time, Delta, EventBatch,
            Track,
        },
        unwrap_items, TimeCaster,
    },
};
use rustc_hash::FxHashMap;

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::{
        ram::{audio_player::InRamAudioPlayer, column::InRamNoteColumn, view::InRamNoteViewData},
        shared::{audio::CompressedAudio, timer::TimeKeeper, track_channel::TrackAndChannel},
    },
};

use super::{block::InRamNoteBlock, InRamMIDIFile};

struct UnendedNote {
    column_index: usize,
    block_index: usize,
}

struct Key {
    column: Vec<InRamNoteBlock>,
    block_builder: Vec<TrackAndChannel>,
    unended_notes: FxHashMap<TrackAndChannel, VecDeque<UnendedNote>>,
}

impl Key {
    fn new() -> Self {
        Key {
            column: Vec::new(),
            block_builder: Vec::new(),
            unended_notes: FxHashMap::default(),
        }
    }

    fn add_note(&mut self, track_chan: TrackAndChannel) {
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

    pub fn end_note(&mut self, track_chan: TrackAndChannel, time: f64) {
        let note = self
            .unended_notes
            .get_mut(&track_chan)
            .and_then(|unended_queue| unended_queue.pop_front());

        if let Some(note) = note {
            if note.column_index == self.column.len() {
                // Note is zero length
                // We don't need to remove it, because when it gets added,
                // the length defaults to zero.
            } else {
                let block = &mut self.column[note.column_index];
                if note.block_index >= block.notes.len() {
                    dbg!(note.block_index, block.notes.len());
                }
                block.set_note_end_time(note.block_index, time);
            }
        }
    }

    pub fn flush(&mut self, time: f64) {
        if !self.block_builder.is_empty() {
            let block = InRamNoteBlock::new_from_trackchans(time, self.block_builder.drain(..));
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

impl InRamMIDIFile {
    pub fn load_from_file(path: &str, player: SimpleTemporaryPlayer) -> Self {
        let midi = TKMIDIFile::open(path, None).unwrap();

        let ppq = midi.ppq();
        let merged = pipe!(
            midi.iter_all_track_events_merged_batches()
            |>TimeCaster::<f64>::cast_event_delta()
            |>cancel_tempo_events(250000)
            |>scale_event_time(1.0 / ppq as f64)
            |>unwrap_items()
        );

        type Ev = Delta<f64, Track<EventBatch<Event>>>;
        let (key_snd, key_rcv) = crossbeam_channel::bounded::<Arc<Ev>>(1000);
        let (audio_snd, audio_rcv) = crossbeam_channel::bounded::<Arc<Ev>>(1000);

        let key_join_handle = thread::spawn(|| {
            let mut keys: Vec<Key> = (0..256).map(|_| Key::new()).collect();

            let mut time = 0.0;

            let mut notes: usize = 0;

            fn flush_keys(time: f64, keys: &mut [Key]) {
                for key in keys.iter_mut() {
                    key.flush(time);
                }
            }

            for batch in key_rcv.into_iter() {
                if batch.delta > 0.0 {
                    flush_keys(time, &mut keys);
                    time += batch.delta;
                }

                for event in batch.iter_events() {
                    let track = event.track;
                    match event.as_event() {
                        Event::NoteOn(e) => {
                            let track_chan = TrackAndChannel::new(track, e.channel);
                            keys[e.key as usize].add_note(track_chan);
                            notes += 1;
                        }
                        Event::NoteOff(e) => {
                            let track_chan = TrackAndChannel::new(track, e.channel);
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

            (keys, notes)
        });

        let audio_join_handle = thread::spawn(|| {
            let vec: Vec<_> = CompressedAudio::build_blocks(audio_rcv.into_iter()).collect();
            vec
        });

        let mut length = 0.0;

        // Write events to the threads
        for batch in merged {
            length += batch.delta;
            let batch = Arc::new(batch);
            key_snd.send(batch.clone()).unwrap();
            audio_snd.send(batch).unwrap();
        }
        // Drop the writers so the threads finish
        drop(key_snd);
        drop(audio_snd);

        let (keys, note_count) = key_join_handle.join().unwrap();
        let audio = audio_join_handle.join().unwrap();

        let mut timer = TimeKeeper::new();

        InRamAudioPlayer::new(audio, timer.get_listener(), player).spawn_playback();

        let columns = keys
            .into_iter()
            .map(|key| InRamNoteColumn::new(key.column))
            .collect();

        InRamMIDIFile {
            view_data: InRamNoteViewData::new(columns, midi.track_count()),
            timer,
            length,
            note_count,
        }
    }
}
