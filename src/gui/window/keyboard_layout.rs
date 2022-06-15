use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub enum KeyboardParams {
    SameWidth,
    Classic {
        black_key_2_set_offset: f32,
        black_key_3_set_offset: f32,
        black_key_scale: f32,
    },
}

impl Default for KeyboardParams {
    fn default() -> Self {
        KeyboardParams::Classic {
            black_key_2_set_offset: 0.3,
            black_key_3_set_offset: 0.5,
            black_key_scale: 0.6,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct KeyPosition {
    pub black: bool,
    pub left: f32,
    pub right: f32,
}

impl KeyPosition {
    pub fn new(key: usize, left: f32, right: f32) -> KeyPosition {
        KeyPosition {
            black: is_black(key),
            left,
            right,
        }
    }
}

pub struct KeyboardLayout {
    keys: [KeyPosition; 257],
    notes: [KeyPosition; 257],
}

const fn is_black(key: usize) -> bool {
    let key = key % 12;
    key == 1 || key == 3 || key == 6 || key == 8 || key == 10
}

fn load_key_numbers() -> [usize; 257] {
    let mut black = 0;
    let mut white = 0;
    let mut numbers = [0; 257];

    for i in 0..257 {
        if is_black(i) {
            numbers[i] = black;
            black += 1;
        } else {
            numbers[i] = white;
            white += 1;
        }
    }

    numbers
}

impl KeyboardLayout {
    pub fn new(params: &KeyboardParams) -> KeyboardLayout {
        let mut keys = [Default::default(); 257];
        let mut notes = [Default::default(); 257];

        // let mut key_numbers = Vec::new();

        let last_key = 256.0;

        match params {
            KeyboardParams::SameWidth => {
                for i in 0..257 {
                    let left = i as f32 / last_key;
                    let right = (i + 1) as f32 / last_key;

                    notes[i] = KeyPosition::new(i, left, right);

                    let mut left = left;
                    let mut right = right;

                    let n = i % 12;

                    let half = 1.0 / 2.0;
                    let third = 1.0 / 3.0;
                    let quarter = 1.0 / 4.0;

                    if n == 0 {
                        right += third * 2.0;
                    } else if n == 2 {
                        left -= third;
                        right += third;
                    } else if n == 4 {
                        left -= third * 2.0
                    } else if n == 5 {
                        right += half + quarter;
                    } else if n == 7 {
                        left -= quarter;
                        right += half;
                    } else if n == 9 {
                        left -= half;
                        right += quarter;
                    } else if n == 11 {
                        left -= half + quarter;
                    }

                    keys[i] = KeyPosition::new(i, left, right);
                }
            }
            KeyboardParams::Classic {
                black_key_2_set_offset,
                black_key_3_set_offset,
                black_key_scale,
            } => {
                let key_numbers = load_key_numbers();

                for i in 0..257 {
                    if !is_black(i) {
                        let left = key_numbers[i] as f32;
                        let right = left + 1.0;

                        notes[i] = KeyPosition::new(i, left, right);
                        keys[i] = KeyPosition::new(i, left, right);
                    } else {
                        let _i = i + 1;
                        let half_width = black_key_scale / 2.0;
                        let black_num = key_numbers[i] % 5;
                        let mut offset = half_width;

                        if black_num == 0 {
                            offset += half_width * black_key_2_set_offset;
                        } else if black_num == 2 {
                            offset += half_width * black_key_3_set_offset;
                        } else if black_num == 1 {
                            offset -= half_width * black_key_2_set_offset;
                        } else if black_num == 4 {
                            offset -= half_width * black_key_3_set_offset;
                        }

                        let left = key_numbers[_i] as f32 - offset;
                        let right = left + black_key_scale;

                        notes[i] = KeyPosition::new(i, left, right);
                        keys[i] = KeyPosition::new(i, left, right);
                    }
                }
            }
        }

        KeyboardLayout { keys, notes }
    }

    pub fn get_range_for_keys(&self, first_key: usize, last_key: usize) -> KeyboardRange {
        KeyboardRange {
            left: self.keys[first_key].left,
            right: self.keys[last_key].right,
        }
    }

    pub fn get_view_for_keys(&self, first_key: usize, last_key: usize) -> KeyboardView {
        let range = self.get_range_for_keys(first_key, last_key);

        let mut left_key = first_key;
        let mut right_key = last_key;

        if self.keys[left_key].black {
            left_key -= 1;
        }
        if self.keys[right_key - 1].black {
            right_key += 1;
        }

        KeyboardView {
            layout: self,
            range,
            visible_range: left_key..right_key,
        }
    }

    pub fn get_view_for_range(&self, range: KeyboardRange) -> KeyboardView {
        let mut left_key = self
            .keys
            .iter()
            .position(|x| x.right >= range.left)
            .unwrap_or(257);
        let mut right_key = self
            .keys
            .iter()
            .position(|x| x.left >= range.right)
            .unwrap_or(257);

        if self.keys[left_key].black {
            left_key -= 1;
        }
        if self.keys[right_key - 1].black {
            right_key += 1;
        }

        if left_key > right_key {
            std::mem::swap(&mut left_key, &mut right_key);
        }

        KeyboardView {
            layout: self,
            range,
            visible_range: left_key..right_key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyboardRange {
    pub left: f32,
    pub right: f32,
}

impl KeyboardRange {
    pub fn new(left: f32, right: f32) -> KeyboardRange {
        KeyboardRange { left, right }
    }

    fn transform(&self, x: f32) -> f32 {
        (x - self.left) / (self.right - self.left)
    }
}

pub struct KeyboardView<'a> {
    layout: &'a KeyboardLayout,
    pub range: KeyboardRange,
    pub visible_range: Range<usize>,
}

impl<'a> KeyboardView<'a> {
    pub fn key(&self, key: usize) -> KeyPosition {
        let key = self.layout.keys[key];
        KeyPosition {
            black: key.black,
            left: self.range.transform(key.left),
            right: self.range.transform(key.right),
        }
    }

    pub fn iter_keys<'b>(&'b self) -> impl 'b + Iterator<Item = (usize, KeyPosition)> {
        self.visible_range.clone().map(|i| (i, self.key(i)))
    }
}
