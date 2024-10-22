use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct Fps {
    ticks: VecDeque<Instant>,
}

impl Fps {
    pub fn new() -> Self {
        Self {
            ticks: VecDeque::new(),
        }
    }

    pub fn update(&mut self) {
        self.ticks.push_back(Instant::now());
        while let Some(front) = self.ticks.front() {
            if front.elapsed() > Duration::from_secs(1) {
                self.ticks.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_fps(&self) -> usize {
        self.ticks.len()
    }
}
