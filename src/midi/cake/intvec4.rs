use bytemuck::{Pod, Zeroable};

use crate::midi::MIDIColor;

fn get_col(c: i32) -> i32 {
    let rgb = MIDIColor::new_from_hue((c * 123) as f64);

    rgb.0 as i32
}

#[repr(C)]
#[derive(Pod, Debug, Copy, Clone, Zeroable)]
pub struct IntVector4 {
    pub val1: i32,
    pub val2: i32,
    pub val3: i32,
    pub val4: i32,
}

impl IntVector4 {
    pub fn default() -> Self {
        IntVector4 {
            val1: 0,
            val2: 0,
            val3: 0,
            val4: 0,
        }
    }

    pub fn new_note(start: i32, end: i32, track_channel: i32) -> IntVector4 {
        IntVector4 {
            val1: start,
            val2: end,
            val3: get_col(track_channel),
            val4: 0,
        }
    }

    pub fn set_note_end(&mut self, end: i32) {
        self.val2 = end;
    }

    pub fn new_empty() -> IntVector4 {
        IntVector4 {
            val1: 0,
            val2: 0,
            val3: -1,
            val4: 0,
        }
    }

    pub fn new_leaf(cutoff: i32, lower: i32, upper: i32) -> IntVector4 {
        IntVector4 {
            val1: cutoff,
            val2: lower,
            val3: upper,
            val4: 12345,
        }
    }
}
