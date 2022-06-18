

use self::{
    view::{InRamCurrentNoteViews, InRamNoteViewData},
};

use super::{shared::timer::TimeKeeper, MIDIFile, MIDIFileBase, MIDIViewRange};

mod audio_player;
pub mod block;
pub mod column;
mod parse;
pub mod view;

pub struct InRamMIDIFile {
    view_data: InRamNoteViewData,
    timer: TimeKeeper,
}

impl InRamMIDIFile {}

macro_rules! impl_file_base {
    ($for_type:ty) => {
        impl MIDIFileBase for $for_type {
            fn midi_length(&self) -> Option<f64> {
                None
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
        }
    };
}

impl_file_base!(&mut InRamMIDIFile);
impl_file_base!(InRamMIDIFile);

impl MIDIFile for &mut InRamMIDIFile {
    type ColumnsViews<'a> = InRamCurrentNoteViews<'a> where Self: 'a;

    fn get_current_column_views<'a>(&'a mut self, range: f64) -> Self::ColumnsViews<'a> {
        let time = self.timer.get_time().as_secs_f64();
        let new_range = MIDIViewRange::new(time, time + range);
        self.view_data.shift_view_range(new_range);

        InRamCurrentNoteViews::new(&self.view_data)
    }
}
