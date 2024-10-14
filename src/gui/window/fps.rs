use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct Fps {
    frames: VecDeque<Instant>,
    current: Instant,
}

impl Fps {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            current: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        self.frames.push_back(Instant::now());
        while let Some(front) = self.frames.front() {
            if front.elapsed() > Duration::from_secs(1) {
                self.frames.pop_front();
            } else {
                break;
            }
        }

        self.current = Instant::now();
    }

    pub fn get_fps(&self) -> usize {
        self.frames.len()
    }
}
