use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct Fps(VecDeque<Instant>);

impl Fps {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn update(&mut self) {
        self.0.push_back(Instant::now());
        while let Some(front) = self.0.front() {
            if front.elapsed() > Duration::from_secs(1) {
                self.0.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_fps(&self) -> usize {
        self.0.len()
    }
}
