use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use bytemuck::{Pod, Zeroable};
use midi_toolkit::{
    events::{Event, MIDIEventEnum},
    io::MIDIFile as TKMIDIFile,
    pipe,
    sequence::{
        event::{cancel_tempo_events, scale_event_time, Delta, EventBatch, Track},
        unwrap_items, TimeCaster,
    },
};
use rustc_hash::FxHashMap;

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::{
        audio::ram::InRamAudioPlayer,
        cake::tree_threader::{NoteEvent, ThreadedTreeSerializers},
        ram::{column::InRamNoteColumn, view::InRamNoteViewData},
        shared::{audio::CompressedAudio, timer::TimeKeeper, track_channel::TrackAndChannel},
    },
};

use self::{blocks::CakeBlock, intvec4::IntVector4};

use super::{MIDIFileBase, MIDIFileStats};

pub mod blocks;
pub mod intvec4;
mod tree_serializer;
mod tree_threader;
mod unended_note_batch;

pub struct CakeMIDIFile {
    blocks: Vec<CakeBlock>,
    timer: TimeKeeper,
    length: f64,
    note_count: usize,
    ticks_per_second: u32,
}

impl CakeMIDIFile {
    pub fn load_from_file(
        path: &str,
        player: Arc<RwLock<SimpleTemporaryPlayer>>,
        random_colors: bool,
    ) -> Self {
        let ticks_per_second = 1000;

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

        let key_join_handle = thread::spawn(move || {
            let mut trees = ThreadedTreeSerializers::new();
            // let mut keys: Vec<Key> = (0..256).map(|_| Key::new()).collect();

            let mut time = 0.0;

            // let mut notes: usize = 0;

            // fn flush_keys(time: f64, keys: &mut [Key]) {
            //     for key in keys.iter_mut() {
            //         key.flush(time);
            //     }
            // }

            for batch in key_rcv.into_iter() {
                time += batch.delta;

                let int_time = (time * ticks_per_second as f64) as i32;

                fn channel_track(channel: u8, track: u32) -> i32 {
                    (channel as i32) + (track as i32) * 16
                }

                for event in batch.iter_events() {
                    let track = event.track;
                    match event.as_event() {
                        Event::NoteOn(e) => {
                            trees.push_event(
                                e.key as usize,
                                NoteEvent::On {
                                    time: int_time,
                                    channel_track: channel_track(e.channel, track),
                                },
                            );
                        }
                        Event::NoteOff(e) => {
                            trees.push_event(
                                e.key as usize,
                                NoteEvent::Off {
                                    time: int_time,
                                    channel_track: channel_track(e.channel, track),
                                },
                            );
                        }
                        _ => {}
                    }
                }
            }
            let final_time = (time * ticks_per_second as f64) as i32;
            let serialized = trees.seal(final_time);

            // flush_keys(time, &mut keys);

            // for key in keys.iter_mut() {
            //     key.end_all(time);
            // }

            // (keys, notes)

            let keys: Vec<_> = serialized
                .into_iter()
                .map(|s| CakeBlock {
                    start_time: 0,
                    end_time: final_time as u32,
                    tree: s,
                })
                .collect();

            (keys, 0)
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

        CakeMIDIFile {
            blocks: keys,
            timer,
            length,
            note_count,
            ticks_per_second,
        }
    }

    pub fn key_blocks(&self) -> &[CakeBlock] {
        &self.blocks
    }

    pub fn ticks_per_second(&self) -> u32 {
        self.ticks_per_second
    }

    pub fn current_time(&self) -> Duration {
        self.timer.get_time()
    }
}

impl MIDIFileBase for CakeMIDIFile {
    fn midi_length(&self) -> Option<f64> {
        Some(self.length)
    }

    fn parsed_up_to(&self) -> Option<f64> {
        None
    }

    fn timer(&self) -> &TimeKeeper {
        &self.timer
    }

    fn timer_mut(&mut self) -> &mut TimeKeeper {
        &mut self.timer
    }

    fn allows_seeking_backward(&self) -> bool {
        true
    }

    fn stats(&self) -> MIDIFileStats {
        MIDIFileStats::new(self.note_count)
    }
}
