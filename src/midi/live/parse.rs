use std::{
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
};

use atomic_float::AtomicF64;
use crossbeam_channel::Receiver;
use midi_toolkit::{
    events::{Event, MIDIEvent},
    io::MIDIFile as TKMIDIFile,
    pipe,
    sequence::{
        event::{
            cancel_tempo_events, convert_events_into_batches, scale_event_time, EventBatch,
            TrackEvent,
        },
        unwrap_items, TimeCaster,
    },
};

use crate::{audio_playback::SimpleTemporaryPlayer, midi::shared::timer::TimeKeeper};

use self::notes::LiveNoteBlockWithKey;

use super::audio_player::LiveAudioPlayer;

mod audio;
mod notes;

pub type TrackEventBatch = EventBatch<f64, TrackEvent<f64, Event<f64>>>;

pub struct ThreadManager {
    parse_time: Arc<AtomicF64>,
    handle: JoinHandle<()>,
}

pub struct LiveMidiParser {
    file_manager: ThreadManager,
    note_manager: ThreadManager,
    audio_manager: ThreadManager,
    note_reciever: Receiver<LiveNoteBlockWithKey>,
}

impl LiveMidiParser {
    pub fn init(path: &str, player: SimpleTemporaryPlayer) -> Self {
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

        let (note_snd, note_rcv) = crossbeam_channel::bounded::<Arc<TrackEventBatch>>(1000);
        let (audio_snd, audio_rcv) = crossbeam_channel::bounded::<Arc<TrackEventBatch>>(1000);

        let notes = notes::init_note_manager(note_rcv);
        let audio = audio::init_audio_manager(audio_rcv);

        let mut timer = TimeKeeper::new();
        LiveAudioPlayer::new(audio.reciever, timer.get_listener(), player).spawn_playback();

        let parse_time_outer = Arc::new(AtomicF64::default());
        let parse_time = parse_time_outer.clone();
        let file_handle = thread::spawn(move || {
            let mut time = 0.0;
            for block in merged {
                if block.delta() > 0.0 {
                    time += block.delta();
                    parse_time.store(time, Ordering::Relaxed);
                }

                let block = Arc::new(block);

                let res = note_snd.send(block.clone());
                if res.is_err() {
                    break;
                }

                let res = audio_snd.send(block);
                if res.is_err() {
                    break;
                }
            }
        });

        Self {
            file_manager: ThreadManager {
                handle: file_handle,
                parse_time: parse_time_outer,
            },
            note_manager: notes.manager,
            audio_manager: audio.manager,
            note_reciever: notes.reciever,
        }
    }

    pub fn parse_time(&self) -> f64 {
        let file_time = self.file_manager.parse_time.load(Ordering::Relaxed);
        let note_time = self.note_manager.parse_time.load(Ordering::Relaxed);
        let audio_time = self.audio_manager.parse_time.load(Ordering::Relaxed);

        file_time.min(note_time).min(audio_time)
    }

    pub fn recieve_next_note_blocks(&self) -> impl '_ + Iterator<Item = LiveNoteBlockWithKey> {
        self.note_reciever.try_iter()
    }
}
