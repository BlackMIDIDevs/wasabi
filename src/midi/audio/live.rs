use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use time::Duration;

use crossbeam_channel::Receiver;

use crate::{
    audio_playback::WasabiAudioPlayer,
    midi::shared::{
        audio::CompressedAudio,
        timer::{TimeListener, UnpauseWaitResult, WaitResult},
    },
};

pub struct LiveAudioPlayer {
    events: Receiver<CompressedAudio>,
    timer: TimeListener,
    player: Arc<WasabiAudioPlayer>,
}

impl LiveAudioPlayer {
    pub fn new(
        events: Receiver<CompressedAudio>,
        timer: TimeListener,
        player: Arc<WasabiAudioPlayer>,
    ) -> Self {
        LiveAudioPlayer {
            events,
            timer,
            player,
        }
    }

    pub fn spawn_playback(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut seek_catching_up = false;

            let max_fall_time = 0.1;

            let push_cc = |e: &CompressedAudio| {
                self.player.push_events(e.iter_control_events());
            };

            for event in self.events.into_iter() {
                if self.timer.is_paused() {
                    self.player.reset();
                    match self.timer.wait_until_unpause() {
                        UnpauseWaitResult::Unpaused => push_cc(&event),
                        UnpauseWaitResult::UnpausedAndSeeked(time) => {
                            if time.as_seconds_f64() - event.time > max_fall_time {
                                seek_catching_up = true;
                            }
                            continue;
                        }
                        UnpauseWaitResult::Killed => break,
                    }
                }

                if seek_catching_up {
                    let time = self.timer.get_time().as_seconds_f64();
                    if time - event.time > max_fall_time {
                        push_cc(&event);
                        continue;
                    } else {
                        seek_catching_up = false;
                    }
                }

                let time = Duration::seconds_f64(event.time);
                match self.timer.wait_until(time) {
                    WaitResult::Ok => {}
                    WaitResult::Paused => {
                        continue;
                    }
                    WaitResult::Seeked(time) => {
                        self.player.reset();
                        if time.as_seconds_f64() - event.time > max_fall_time {
                            seek_catching_up = true;
                        }
                        continue;
                    }
                    WaitResult::Killed => {
                        self.player.reset();
                        break;
                    }
                }

                self.player.push_events(event.iter_events());
            }
        })
    }
}
