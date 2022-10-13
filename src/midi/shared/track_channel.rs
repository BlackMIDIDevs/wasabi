#![allow(dead_code)]

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct TrackAndChannel(u32);

impl TrackAndChannel {
    pub fn new(track: u32, channel: u8) -> Self {
        TrackAndChannel(track * 16 + channel as u32)
    }

    pub fn track(&self) -> u32 {
        self.0 / 16
    }

    pub fn channel(&self) -> u8 {
        (self.0 % 16) as u8
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
