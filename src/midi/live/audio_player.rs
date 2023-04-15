use std::{
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::Receiver;

use crate::{
    audio_playback::SimpleTemporaryPlayer,
    midi::shared::{
        audio::CompressedAudio,
        timer::{TimeListener, UnpauseWaitResult, WaitResult},
    },
};

pub struct LiveAudioPlayer {
    events: Receiver<CompressedAudio>,
    timer: TimeListener,
    player: Arc<RwLock<SimpleTemporaryPlayer>>,
}

impl LiveAudioPlayer {
    pub fn new(
        events: Receiver<CompressedAudio>,
        timer: TimeListener,
        player: Arc<RwLock<SimpleTemporaryPlayer>>,
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

            let reset = || {
                if let Ok(mut player) = self.player.clone().write() {
                    player.reset();
                };
            };

            let push_cc = |e: &CompressedAudio| {
                if let Ok(mut player) = self.player.clone().write() {
                    player.push_events(e.iter_control_events());
                }
            };

            for event in self.events.into_iter() {
                if self.timer.is_paused() {
                    reset();
                    match self.timer.wait_until_unpause() {
                        UnpauseWaitResult::Unpaused => push_cc(&event),
                        UnpauseWaitResult::UnpausedAndSeeked(time) => {
                            if time.as_secs_f64() - event.time > max_fall_time {
                                seek_catching_up = true;
                            }
                            continue;
                        }
                        UnpauseWaitResult::Killed => break,
                    }
                }

                if seek_catching_up {
                    let time = self.timer.get_time().as_secs_f64();
                    if time - event.time > max_fall_time {
                        push_cc(&event);
                        continue;
                    } else {
                        seek_catching_up = false;
                    }
                }

                let time = Duration::from_secs_f64(event.time);
                match self.timer.wait_until(time) {
                    WaitResult::Ok => {}
                    WaitResult::Paused => {
                        continue;
                    }
                    WaitResult::Seeked(time) => {
                        reset();
                        if time.as_secs_f64() - event.time > max_fall_time {
                            seek_catching_up = true;
                        }
                        continue;
                    }
                    WaitResult::Killed => {
                        reset();
                        break;
                    }
                }

                if let Ok(mut player) = self.player.clone().write() {
                    player.push_events(event.iter_events());
                }
            }
        })
    }
}
