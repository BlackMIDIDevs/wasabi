use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

pub struct Fps {
    frames: VecDeque<Instant>,
    limit: Option<usize>,
    current: Instant,
}

impl Fps {
    pub fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            limit: None,
            current: Instant::now(),
        }
    }

    pub fn set_limit(&mut self, limit: Option<usize>) {
        self.limit = limit;
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

        if let Some(limit) = self.limit {
            let elapsed = self.current.elapsed().as_secs_f64();
            if self.frames.len() > limit {
                let limit = 1.0 / limit as f64;
                let should_wait = limit - elapsed;
                if should_wait > 0.0 {
                    spin_sleep::sleep(Duration::from_secs_f64(should_wait));
                }
            }
        }

        self.current = Instant::now();
    }

    pub fn get_fps(&self) -> usize {
        self.frames.len()
    }
}
