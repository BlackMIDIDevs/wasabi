use self::view::{InRamCurrentNoteViews, InRamNoteViewData};

use super::{shared::timer::TimeKeeper, MIDIFile, MIDIFileBase, MIDIViewRange};

mod audio_player;
pub mod block;
pub mod column;
mod parse;
pub mod view;

pub struct LiveLoadMIDIFile {
    view_data: InRamNoteViewData,
    timer: TimeKeeper,
    length: f64,
}

impl LiveLoadMIDIFile {}

macro_rules! impl_file_base {
    ($for_type:ty) => {
        impl MIDIFileBase for $for_type {
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
                false
            }
        }
    };
}

impl_file_base!(&mut LiveLoadMIDIFile);
impl_file_base!(LiveLoadMIDIFile);

impl MIDIFile for &mut LiveLoadMIDIFile {
    type ColumnsViews<'a> = InRamCurrentNoteViews<'a> where Self: 'a;

    fn get_current_column_views(&mut self, range: f64) -> Self::ColumnsViews<'_> {
        let time = self.timer.get_time().as_secs_f64();
        let new_range = MIDIViewRange::new(time, time + range);
        self.view_data.shift_view_range(new_range);

        InRamCurrentNoteViews::new(&self.view_data)
    }
}
