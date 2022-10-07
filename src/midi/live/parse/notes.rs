use std::{
    collections::VecDeque,
    sync::{atomic::Ordering, Arc},
};

use atomic_float::AtomicF64;
use crossbeam_channel::{Receiver, Sender};
use midi_toolkit::events::{Event, MIDIEventEnum};

use crate::midi::{
    live::block::{LiveNoteEnderHandle, LiveRefNoteBlock},
    shared::track_channel::TrackAndChannel,
};

use super::{ThreadManager, TrackEventBatch};

pub struct LiveNoteBlockWithKey {
    pub block: LiveRefNoteBlock,
    pub key: u8,
}

struct TrackUnendedNotes {
    queues: Box<[VecDeque<LiveNoteEnderHandle>]>,
}

impl TrackUnendedNotes {
    fn new() -> Self {
        let mut queues = Vec::with_capacity(16 * 256);
        for _ in 0..16 * 256 {
            queues.push(VecDeque::new());
        }

        TrackUnendedNotes {
            queues: queues.into_boxed_slice(),
        }
    }

    fn get_index(&mut self, key: u8, channel: u8) -> usize {
        (key as usize) + (channel as usize) * 256
    }

    fn end_all_notes(&mut self, time: f64) {
        for queue in self.queues.iter_mut() {
            while let Some(mut note) = queue.pop_front() {
                note.end(time);
            }
        }
    }

    fn end_note(&mut self, key: u8, channel: u8, time: f64) {
        let index = self.get_index(key, channel);
        if let Some(mut note) = self.queues[index].pop_front() {
            note.end(time);
        }
    }

    fn add_note(&mut self, key: u8, channel: u8, handle: LiveNoteEnderHandle) {
        let index = self.get_index(key, channel);
        self.queues[index].push_back(handle);
    }
}

struct UnendedNotesHandler {
    unended_notes: Vec<Option<TrackUnendedNotes>>,
}

impl UnendedNotesHandler {
    pub fn new() -> Self {
        UnendedNotesHandler {
            unended_notes: Vec::new(),
        }
    }

    fn get_track(&mut self, track: u32) -> &mut TrackUnendedNotes {
        while self.unended_notes.len() <= track as usize {
            self.unended_notes.push(None);
        }

        self.unended_notes[track as usize].get_or_insert_with(TrackUnendedNotes::new)
    }

    fn end_all_notes(&mut self, time: f64) {
        for track in self.unended_notes.iter_mut().flatten() {
            track.end_all_notes(time);
        }
    }
}

struct ParserState {
    unended_notes: UnendedNotesHandler,
    keys: Box<[Vec<TrackAndChannel>]>,
    sender: Sender<LiveNoteBlockWithKey>,
}

impl ParserState {
    fn new(sender: Sender<LiveNoteBlockWithKey>) -> Self {
        let mut keys = Vec::with_capacity(256);
        for _ in 0..256 {
            keys.push(Vec::new());
        }
        ParserState {
            unended_notes: UnendedNotesHandler::new(),
            keys: keys.into_boxed_slice(),
            sender,
        }
    }

    fn add_note(&mut self, key: u8, track_chan: TrackAndChannel) {
        self.keys[key as usize].push(track_chan);
    }

    fn flush(&mut self, time: f64) -> Result<(), ()> {
        for i in 0..self.keys.len() {
            if !self.keys[i].is_empty() {
                let (block, iter) =
                    LiveRefNoteBlock::new_from_trackchans(time, self.keys[i].drain(..));
                for unended in iter {
                    let track = self.unended_notes.get_track(unended.track_chan.track());
                    track.add_note(i as u8, unended.track_chan.channel(), unended.handle);
                }
                self.sender
                    .send(LiveNoteBlockWithKey {
                        block,
                        key: i as u8,
                    })
                    .map_err(|_| ())?;
            }
        }

        Ok(())
    }

    fn end_note(&mut self, key: u8, track_chan: TrackAndChannel, time: f64) {
        let track = self.unended_notes.get_track(track_chan.track());
        track.end_note(key, track_chan.channel(), time);
    }

    fn end_all_notes(&mut self, time: f64) {
        self.unended_notes.end_all_notes(time);
    }
}

pub struct NoteParserResult {
    pub reciever: Receiver<LiveNoteBlockWithKey>,
    pub manager: ThreadManager,
}

pub fn init_note_manager(blocks: Receiver<Arc<TrackEventBatch>>) -> NoteParserResult {
    let (sender, reciever) = crossbeam_channel::unbounded();
    let parse_time_outer = Arc::new(AtomicF64::default());

    let parse_time = parse_time_outer.clone();

    let mut state = ParserState::new(sender);
    let join_handle = std::thread::spawn(move || {
        let mut time: f64 = 0.0;
        for block in blocks.into_iter() {
            if block.delta > 0.0 {
                let result = state.flush(time);
                if result.is_err() {
                    break;
                }
                time += block.delta;
            }

            for event in block.iter_events() {
                match event.as_event() {
                    Event::NoteOn(e) => {
                        state.add_note(e.key, TrackAndChannel::new(event.track, e.channel));
                    }
                    Event::NoteOff(e) => {
                        state.end_note(e.key, TrackAndChannel::new(event.track, e.channel), time);
                    }
                    _ => {}
                }
            }

            parse_time.store(time, Ordering::Relaxed);
        }
    });

    NoteParserResult {
        reciever,
        manager: ThreadManager {
            handle: join_handle,
            parse_time: parse_time_outer,
        },
    }
}
