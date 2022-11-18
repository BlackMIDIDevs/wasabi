use std::{collections::VecDeque, time::Instant};

pub struct Fps(VecDeque<Instant>);

const FPS_WINDOW: f64 = 0.5;

impl Fps {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }

    pub fn update(&mut self) {
        self.0.push_back(Instant::now());
        while let Some(front) = self.0.front() {
            if front.elapsed().as_secs_f64() > FPS_WINDOW {
                self.0.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn get_fps(&self) -> f64 {
        if self.0.is_empty() {
            0.0
        } else {
            self.0.len() as f64 / self.0.front().unwrap().elapsed().as_secs_f64()
        }
    }
}
