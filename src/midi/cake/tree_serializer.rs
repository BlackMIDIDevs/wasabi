use std::collections::VecDeque;

use super::{intvec4::IntVector4, unended_note_batch::UnendedNotes};

enum TreeFrame {
    WaitingLeft {
        end: i32,
    },
    WaitingRight {
        left_address: i32,
        mid: i32,
        end: i32,
        notes_to_the_left: u32,
    },
}

struct NoteMarker {
    start: i32,
    track_channel: i32,
    color: i32,
    written_pos: Option<i32>,
}

/// The "TreeSerializer" implements a pushdown automata which calculates the note binary tree
/// from the individual events. Each pushdown frame can either be "waiting left", meaning it's
/// waiting for the left side of a binary tree leaf (which can't be at the top of the stack),
/// or "waiting right", which is waiting for the right side of a binary tree leaf, which can be
/// at the top of the stack.
pub struct TreeSerializer {
    note_stack: UnendedNotes<i32, NoteMarker>,
    tree_frames: VecDeque<TreeFrame>,

    written_values: Vec<IntVector4>,

    added_notes: u32,
    last_tree_time: i32,
}

impl std::fmt::Debug for TreeSerializer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TreeSerializer {{}}")
    }
}

impl TreeSerializer {
    pub fn new() -> TreeSerializer {
        let written_values = vec![IntVector4::new_empty()];

        TreeSerializer {
            note_stack: UnendedNotes::new(),
            tree_frames: VecDeque::new(),

            written_values,

            added_notes: 0,
            last_tree_time: 0,
        }
    }

    fn get_top_note_address(&mut self) -> i32 {
        let top_marker = self.note_stack.top_mut();
        match top_marker {
            None => 0,

            // Negative are returned because note addresses are negative
            Some(marker) => match marker.written_pos {
                Some(pos) => -pos,
                None => {
                    let written_pos = self.written_values.len() as i32;
                    self.written_values
                        .push(IntVector4::new_note(marker.start, 0, marker.color));
                    marker.written_pos = Some(written_pos);
                    -written_pos
                }
            },
        }
    }

    /// Writes a new "leaf" to the tree, which is a node with two children. The left and right,
    /// and a cuttoff point "mid" which is the time separator between left and right.
    fn write_leaf(
        &mut self,
        left_addr: i32,
        right_addr: i32,
        mid: i32,
        notes_to_the_left: u32,
    ) -> i32 {
        fn diff(address: i32, reference: i32) -> i32 {
            if address <= 0 {
                if -address > reference {
                    panic!();
                }
                reference + address
            } else {
                if address > reference {
                    panic!();
                }
                address - reference
            }
        }

        let written_pos = self.written_values.len() as i32;

        let left = diff(left_addr, written_pos);
        let right = diff(right_addr, written_pos);

        self.written_values
            .push(IntVector4::new_leaf(mid, left, right, notes_to_the_left));
        written_pos
    }

    /// Processes a note start. If the time is greater than the last tree time, the tree is
    /// updated to the new time. Then, the note is pushed to the note stack.
    pub fn start_note(&mut self, time: i32, track_channel: i32, color: i32) {
        if time > self.last_tree_time {
            self.process_change(time);
        }

        self.added_notes += 1;

        self.note_stack.push_note(
            track_channel,
            NoteMarker {
                start: time,
                track_channel,
                color,
                written_pos: None,
            },
        );
    }

    /// Processes a note end. If the time is greater than the last tree time, the tree is
    /// updated to the new time. Then, the note is popped from the note stack, and the
    /// end for the note is also written.
    pub fn end_note(&mut self, time: i32, track_channel: i32) {
        if time > self.last_tree_time {
            self.process_change(time);
        }

        let marker = self.note_stack.get_note_for(track_channel);

        let marker = if let Some(marker) = marker {
            marker
        } else {
            //ignore
            return;
        };

        if marker.is_last {
            // last note

            if time > self.last_tree_time {
                self.process_change(time);
            }
        }

        if let Some(index) = marker.value.written_pos {
            self.written_values[index as usize].set_note_end(time);
        }
    }

    /// Ends all notes, finishes all stack frames, inserts the address of the last item into the start of the array,
    /// and returns the array.
    pub fn complete_and_seal(mut self, time: i32) -> Vec<IntVector4> {
        self.end_all_notes(time);
        self.end_all_frames();

        if self.written_values.len() == 1 {
            self.write_leaf(0, 0, 0, 0);
        }

        self.written_values
            .insert(0, IntVector4::new_length_marker(self.written_values.len()));

        self.written_values
    }

    fn process_change(&mut self, until: i32) {
        let address = self.get_top_note_address();

        self.last_tree_time = until;

        let top_frame = self.tree_frames.back();

        match top_frame {
            None => {
                // If there are no frames, initialize the top stack frame
                self.tree_frames.push_back(TreeFrame::WaitingRight {
                    left_address: address,
                    mid: until,
                    end: until * 2,
                    notes_to_the_left: 0,
                });
            }
            Some(frame) => match frame {
                TreeFrame::WaitingLeft { .. } => {
                    panic!("Top frame must always be right frame");
                }
                TreeFrame::WaitingRight { mid, end, .. } => {
                    let mut end = *end;
                    let start = *mid;

                    if end > until {
                        loop {
                            let mid = (end + start) / 2;
                            if mid > until {
                                // If the frame is too wide, push a left frame and tunnel down
                                self.tree_frames.push_back(TreeFrame::WaitingLeft { end });
                                end = mid;
                            } else {
                                // If the frame is wide enough, we'll push a right frame instead
                                break;
                            }
                        }

                        // Push a frame, with the midpoint being the end of the change range
                        self.tree_frames.push_back(TreeFrame::WaitingRight {
                            left_address: address,
                            mid: until,
                            end,
                            notes_to_the_left: self.added_notes,
                        });
                    } else {
                        let mut address = address;
                        loop {
                            match self.tree_frames.back() {
                                Some(TreeFrame::WaitingLeft { end, .. }) => {
                                    let end = *end;

                                    self.tree_frames.pop_back();
                                    if until >= end {
                                        // Left frame is smaller than change range
                                        // Skip and step up
                                        continue;
                                    } else {
                                        // Left frame is larger than change range
                                        // Replace with right frame, and exit
                                        self.tree_frames.push_back(TreeFrame::WaitingRight {
                                            left_address: address,
                                            mid: until,
                                            end,
                                            notes_to_the_left: self.added_notes,
                                        });
                                        break;
                                    }
                                }
                                Some(TreeFrame::WaitingRight {
                                    left_address,
                                    mid,
                                    notes_to_the_left,
                                    ..
                                }) => {
                                    let left_address = *left_address;
                                    let mid = *mid;

                                    // Write frame to array, update address, step up
                                    address = self.write_leaf(
                                        left_address,
                                        address,
                                        mid,
                                        *notes_to_the_left,
                                    );

                                    self.tree_frames.pop_back();
                                }
                                None => {
                                    // We have reached the top. Push a new frame
                                    self.tree_frames.push_back(TreeFrame::WaitingRight {
                                        left_address: address,
                                        mid: until,
                                        end: until * 2,
                                        notes_to_the_left: self.added_notes,
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            },
        }
    }

    /// Finishes all unfinished notes, and writes their end times into the array.
    fn end_all_notes(&mut self, time: i32) {
        if self.note_stack.len() == 0 {
            return;
        }

        self.process_change(time);
        for marker in self.note_stack.drain_all() {
            if let Some(index) = marker.written_pos {
                self.written_values[index as usize].set_note_end(time);
            }
        }
    }

    /// Closes all stack frames, and writes them into the array.
    fn end_all_frames(&mut self) {
        // Loop until no frames are left

        let mut address = self.get_top_note_address();

        loop {
            match self.tree_frames.back() {
                Some(TreeFrame::WaitingLeft { .. }) => {
                    self.tree_frames.pop_back();
                    // Left frames hold no data, skip
                }
                Some(TreeFrame::WaitingRight {
                    left_address,
                    mid,
                    notes_to_the_left,
                    ..
                }) => {
                    let left_address = *left_address;
                    let mid = *mid;

                    // Write frame to array, update address, step up
                    address = self.write_leaf(left_address, address, mid, *notes_to_the_left);

                    self.tree_frames.pop_back();
                }
                None => {
                    // We have reached the top. End the loop.
                    break;
                }
            }
        }
    }
}
