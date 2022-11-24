use std::{
    sync::{atomic::Ordering, Arc, RwLock},
    thread,
};

use atomic_float::AtomicF64;
use midi_toolkit::{io::MIDIFile as TKMIDIFile, sequence::event::get_channels_array_statistics};

use crate::audio_playback::SimpleTemporaryPlayer;

use self::{
    parse::LiveMidiParser,
    view::{LiveCurrentNoteViews, LiveNoteViewData},
};

use super::{shared::timer::TimeKeeper, MIDIFile, MIDIFileBase, MIDIFileStats, MIDIViewRange};

mod audio_player;
pub mod block;
pub mod column;
mod parse;
pub mod view;

pub struct LiveLoadMIDIFile {
    view_data: LiveNoteViewData,
    timer: TimeKeeper,
    length: Arc<AtomicF64>,
}

impl LiveLoadMIDIFile {
    pub fn load_from_file(
        path: &str,
        player: Arc<RwLock<SimpleTemporaryPlayer>>,
        random_colors: bool,
    ) -> Self {
        let midi = TKMIDIFile::open(path, None).unwrap();

        let parse_length_outer = Arc::new(AtomicF64::new(f64::NAN));
        let parse_length = parse_length_outer.clone();

        let ppq = midi.ppq();
        let tracks = midi.iter_all_tracks().collect();
        thread::spawn(move || {
            let stats = get_channels_array_statistics(tracks);
            if let Ok(stats) = stats {
                parse_length.store(
                    stats.calculate_total_duration(ppq).as_secs_f64(),
                    Ordering::Relaxed,
                );
            }
        });

        let mut timer = TimeKeeper::new();

        let parer = LiveMidiParser::init(&midi, player, &mut timer);
        let file = LiveNoteViewData::new(parer, midi.track_count(), random_colors);

        LiveLoadMIDIFile {
            view_data: file,
            timer,
            length: parse_length_outer,
        }
    }
}

impl MIDIFileBase for LiveLoadMIDIFile {
    fn midi_length(&self) -> Option<f64> {
        let value = self.length.load(Ordering::Relaxed);
        if value.is_nan() {
            None
        } else {
            Some(value)
        }
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
        MIDIFileStats::new(0)
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
