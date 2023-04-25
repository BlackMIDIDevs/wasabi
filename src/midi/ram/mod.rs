use self::view::{InRamCurrentNoteViews, InRamNoteViewData};

use super::{
    shared::timer::TimeKeeper, MIDIFile, MIDIFileBase, MIDIFileUniqueSignature, MIDIViewRange,
};

pub mod block;
pub mod column;
mod parse;
pub mod view;

pub struct MIDIFileStats {
    pub total_notes: u64,
    pub passed_notes: u64,
}

impl MIDIFileStats {
    pub fn new(notes: u64) -> Self {
        Self {
            total_notes: notes,
            passed_notes: 0,
        }
    }
}

pub struct InRamMIDIFile {
    view_data: InRamNoteViewData,
    timer: TimeKeeper,
    length: f64,
    note_count: u64,
    signature: MIDIFileUniqueSignature,
}

impl InRamMIDIFile {}

impl MIDIFileBase for InRamMIDIFile {
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

    fn signature(&self) -> &MIDIFileUniqueSignature {
        &self.signature
    }
}

impl MIDIFile for InRamMIDIFile {
    type ColumnsViews<'a> = InRamCurrentNoteViews<'a> where Self: 'a;

    fn get_current_column_views(&mut self, range: f64) -> Self::ColumnsViews<'_> {
        let time = self.timer.get_time().as_secs_f64();
        let new_range = MIDIViewRange::new(time, time + range);
        self.view_data.shift_view_range(new_range);

        InRamCurrentNoteViews::new(&self.view_data)
    }
}
