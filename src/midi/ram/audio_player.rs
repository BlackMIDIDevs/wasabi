use std::{
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::shared::{
        audio::CompressedAudio,
        timer::{SeekWaitResult, TimeListener, UnpauseWaitResult, WaitResult},
    },
};

pub struct InRamAudioPlayer {
    events: Vec<CompressedAudio>,
    timer: TimeListener,
    player: SimpleTemporaryPlayer,
    index: usize,
}

impl InRamAudioPlayer {
    pub fn new(
        events: Vec<CompressedAudio>,
        timer: TimeListener,
        player: SimpleTemporaryPlayer,
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
            if self.timer.is_paused() {
                match self.timer.wait_until_unpause() {
                    UnpauseWaitResult::Unpaused => {}
                    UnpauseWaitResult::UnpausedAndSeeked(time) => {
                        self.seek_to_time(time.as_secs_f64());
                        continue;
                    }
                    UnpauseWaitResult::Killed => break,
                }
            }

            if self.index >= self.events.len() {
                match self.timer.wait_until_seeked() {
                    SeekWaitResult::UnpausedAndSeeked(time) => {
                        self.seek_to_time(time.as_secs_f64());
                        continue;
                    }
                    SeekWaitResult::Killed => break,
                }
            }

            let event = &self.events[self.index];

            let time = Duration::from_secs_f64(event.time);
            match self.timer.wait_until(time) {
                WaitResult::Ok => {}
                WaitResult::Paused => continue,
                WaitResult::Seeked(time) => {
                    self.seek_to_time(time.as_secs_f64());
                    continue;
                }
                WaitResult::Killed => break,
            }

            self.player.push_events(event.iter_events());
            self.index += 1;
        })
    }

    fn seek_to_time(&mut self, time: f64) {
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
                left = mid + 1;
            } else if time > range_end {
                right = mid;
            } else {
                self.index = mid;
                return;
            }

            size = right - left;
        }

        self.index = events.len();
    }
}
