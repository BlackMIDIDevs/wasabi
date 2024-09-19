use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Debug, Copy, Clone, Zeroable)]
pub struct IntVector4 {
    val1: i32,
    val2: i32,
    val3: i32,
    val4: i32,
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

    pub fn new_note(start: i32, end: i32, color: i32) -> IntVector4 {
        IntVector4 {
            val1: start,
            val2: end,
            val3: color,
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

    pub fn new_leaf(cutoff: i32, lower: i32, upper: i32, notes_to_the_left: u32) -> IntVector4 {
        IntVector4 {
            val1: cutoff,
            val2: lower,
            val3: upper,
            val4: notes_to_the_left as i32,
        }
    }

    pub fn new_length_marker(length: usize) -> IntVector4 {
        IntVector4 {
            val1: length as i32,
            val2: 0,
            val3: 0,
            val4: 0,
        }
    }

    pub fn leaf_cutoff(&self) -> i32 {
        self.val1
    }

    pub fn leaf_left(&self) -> i32 {
        self.val2
    }

    pub fn leaf_right(&self) -> i32 {
        self.val3
    }

    pub fn leaf_notes_to_the_left(&self) -> u32 {
        self.val4 as u32
    }

    pub fn note_start(&self) -> u32 {
        self.val1 as u32
    }

    pub fn note_end(&self) -> u32 {
        self.val2 as u32
    }

    pub fn note_color(&self) -> u32 {
        self.val3 as u32
    }

    pub fn is_note_empty(&self) -> bool {
        self.val3 == -1
    }

    pub fn length_marker_len(&self) -> usize {
        self.val1 as usize
    }
}
