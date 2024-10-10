use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use time::Duration;

use crate::{
    audio_playback::WasabiAudioPlayer,
    midi::shared::{
        audio::CompressedAudio,
        timer::{SeekWaitResult, TimeListener, UnpauseWaitResult, WaitResult},
    },
};

pub struct InRamAudioPlayer {
    events: Vec<CompressedAudio>,
    timer: TimeListener,
    player: Arc<WasabiAudioPlayer>,
    index: usize,
}

impl InRamAudioPlayer {
    pub fn new(
        events: Vec<CompressedAudio>,
        timer: TimeListener,
        player: Arc<WasabiAudioPlayer>,
    ) -> Self {
        InRamAudioPlayer {
            events,
            timer,
            player,
            index: 0,
        }
    }

    pub fn spawn_playback(mut self) -> JoinHandle<()> {
        thread::spawn(move || loop {
            let reset = || {
                self.player.reset();
            };

            if self.timer.is_paused() {
                reset();
                match self.timer.wait_until_unpause() {
                    UnpauseWaitResult::Unpaused => {
                        self.seek_to_time(self.timer.get_time().as_seconds_f64());
                        continue;
                    }
                    UnpauseWaitResult::UnpausedAndSeeked(time) => {
                        self.seek_to_time(time.as_seconds_f64());
                        continue;
                    }
                    UnpauseWaitResult::Killed => break,
                }
            }

            if self.index >= self.events.len() {
                match self.timer.wait_until_seeked() {
                    SeekWaitResult::UnpausedAndSeeked(time) => {
                        self.seek_to_time(time.as_seconds_f64());
                        continue;
                    }
                    SeekWaitResult::Killed => break,
                }
            }

            let event = &self.events[self.index];

            let time = Duration::seconds_f64(event.time);
            match self.timer.wait_until(time) {
                WaitResult::Ok => {}
                WaitResult::Paused => {
                    continue;
                }
                WaitResult::Seeked(time) => {
                    reset();
                    self.seek_to_time(time.as_seconds_f64());
                    continue;
                }
                WaitResult::Killed => {
                    reset();
                    break;
                }
            }

            self.player.push_events(event.iter_events());
            self.index += 1;
        })
    }

    fn find_time_index(&self, time: f64) -> usize {
        if time < 0.0 {
            return 0;
        }

        let events = &self.events;

        // Binary search to find the right time segment

        let mut size = events.len();
        let mut left = 0;
        let mut right = size;
        while left < right {
            let mid = left + size / 2;

            let range_start = events
                .get(mid - 1)
                .map(|t| t.time)
                .unwrap_or(f64::NEG_INFINITY);
            let range_end = events[mid].time;

            if time < range_start {
                right = mid;
            } else if time > range_end {
                left = mid + 1;
            } else {
                return mid;
            }

            size = right - left;
        }

        events.len()
    }

    fn seek_to_time(&mut self, time: f64) {
        self.index = self.find_time_index(time);

        // Reset and push all control events before
        self.player.reset();
        for i in 0..(self.index) {
            self.player
                .push_events(self.events[i].iter_control_events());
        }
    }
}
