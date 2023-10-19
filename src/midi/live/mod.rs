use std::{
    sync::{Arc, RwLock},
    thread,
};

use midi_toolkit::{io::MIDIFile as TKMIDIFile, sequence::event::get_channels_array_statistics};

use crate::audio_playback::SimpleTemporaryPlayer;

use self::{
    parse::LiveMidiParser,
    view::{LiveCurrentNoteViews, LiveNoteViewData},
};

use super::{
    open_file_and_signature, shared::timer::TimeKeeper, MIDIFile, MIDIFileBase, MIDIFileStats,
    MIDIFileUniqueSignature, MIDIViewRange,
};

pub mod block;
pub mod column;
mod parse;
pub mod view;

struct ParseStats {
    length: f64,
    note_count: u64,
}

pub struct LiveLoadMIDIFile {
    view_data: LiveNoteViewData,
    timer: TimeKeeper,
    stats: Arc<RwLock<Option<ParseStats>>>,
    signature: MIDIFileUniqueSignature,
}

impl LiveLoadMIDIFile {
    pub fn load_from_file(
        path: &str,
        player: Arc<RwLock<SimpleTemporaryPlayer>>,
        random_colors: bool,
    ) -> Self {
        let (file, signature) = open_file_and_signature(path);

        let midi = TKMIDIFile::open_from_stream(file, None).unwrap();

        let stats_outer = Arc::new(RwLock::new(None));
        let stats = stats_outer.clone();

        let ppq = midi.ppq();
        let tracks = midi.iter_all_tracks().collect();
        thread::spawn(move || {
            let stats = get_channels_array_statistics(tracks);
            if let Ok(stats) = stats {
                let mut parser_stats = stats_outer.write().unwrap();
                *parser_stats = Some(ParseStats {
                    length: stats.calculate_total_duration(ppq).as_secs_f64(),
                    note_count: stats.note_count(),
                });
            }
        });

        let mut timer = TimeKeeper::new();

        let parer = LiveMidiParser::init(&midi, player, &mut timer);
        let file = LiveNoteViewData::new(parer, midi.track_count(), random_colors);

        LiveLoadMIDIFile {
            view_data: file,
            timer,
            stats,
            signature,
        }
    }
}

impl MIDIFileBase for LiveLoadMIDIFile {
    fn midi_length(&self) -> Option<f64> {
        let data = self.stats.read().unwrap();
        data.as_ref().map(|data| data.length)
    }

    fn parsed_up_to(&self) -> Option<f64> {
        Some(self.view_data.parse_time())
    }

    fn timer(&self) -> &TimeKeeper {
        &self.timer
    }

    fn timer_mut(&mut self) -> &mut TimeKeeper {
        &mut self.timer
    }

    fn allows_seeking_backward(&self) -> bool {
        false
    }

    fn stats(&self) -> MIDIFileStats {
        let stats = self.stats.read().unwrap();

        MIDIFileStats {
            passed_notes: Some(self.view_data.passed_notes()),
            total_notes: stats.as_ref().map(|stats| stats.note_count),
        }
    }

    fn signature(&self) -> &MIDIFileUniqueSignature {
        &self.signature
    }
}

impl MIDIFile for LiveLoadMIDIFile {
    type ColumnsViews<'a> = LiveCurrentNoteViews<'a> where Self: 'a;

    fn get_current_column_views(&mut self, range: f64) -> Self::ColumnsViews<'_> {
        let time = self.timer.get_time().as_secs_f64();
        let new_range = MIDIViewRange::new(time, time + range);
        self.view_data.shift_view_range(new_range);

        LiveCurrentNoteViews::new(&self.view_data)
    }
}
