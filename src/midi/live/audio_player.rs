use std::{
    thread::{self, JoinHandle},
    time::Duration,
    sync::{Arc, RwLock}
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

            for event in self.events.into_iter() {
                if self.timer.is_paused() {
                    match self.timer.wait_until_unpause() {
                        UnpauseWaitResult::Unpaused => {}
                        UnpauseWaitResult::UnpausedAndSeeked(time) => {
                            if time.as_secs_f64() - event.time > max_fall_time {
                                seek_catching_up = true;
                                if let Ok(mut player) = self.player.clone().write() {
                                    player.reset();
                                };
                            }
                            continue;
                        }
                        UnpauseWaitResult::Killed => break,
                    }
                }

                if seek_catching_up {
                    let time = self.timer.get_time().as_secs_f64();
                    if time - event.time > max_fall_time {
                        if let Ok(mut player) = self.player.clone().write() {
                            player.push_events(event.iter_control_events());
                        }
                        continue;
                    } else {
                        seek_catching_up = false;
                    }
                }

                let time = Duration::from_secs_f64(event.time);
                match self.timer.wait_until(time) {
                    WaitResult::Ok => {}
                    WaitResult::Paused => continue,
                    WaitResult::Seeked(time) => {
                        if time.as_secs_f64() - event.time > max_fall_time {
                            seek_catching_up = true;
                            if let Ok(mut player) = self.player.clone().write() {
                                player.reset();
                            };
                        }
                        continue;
                    }
                    WaitResult::Killed => break,
                }

                if let Ok(mut player) = self.player.clone().write() {
                    player.push_events(event.iter_events());
                }
            }
        })
    }
}
