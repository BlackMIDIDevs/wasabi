// #![allow(dead_code)]

use std::time::{Duration, Instant};

struct NotifySignal {
    new_state: TimerState,
    has_seeked: bool,
}

#[derive(Debug, Clone)]
enum TimerState {
    Running {
        continue_time: Instant,
        time_offset: Duration,
    },
    Paused {
        time_offset: Duration,
    },
}

impl TimerState {
    fn get_time(&self) -> Duration {
        match self {
            TimerState::Running {
                continue_time,
                time_offset,
            } => continue_time.elapsed() + *time_offset,
            TimerState::Paused { time_offset } => *time_offset,
        }
    }

    fn is_paused(&self) -> bool {
        matches!(self, TimerState::Paused { .. })
    }
}

#[derive(Debug, Clone)]
pub struct TimeKeeper {
    current_state: TimerState,
    listeners: Vec<crossbeam_channel::Sender<NotifySignal>>,
}

impl TimeKeeper {
    pub fn new() -> Self {
        Self {
            current_state: TimerState::Paused {
                time_offset: Duration::new(0, 0),
            },
            listeners: Vec::new(),
        }
    }

    pub fn get_time(&self) -> Duration {
        self.current_state.get_time()
    }

    pub fn is_paused(&self) -> bool {
        self.current_state.is_paused()
    }

    pub fn get_listener(&mut self) -> TimeListener {
        let (snd, rcv) = crossbeam_channel::unbounded();
        self.listeners.push(snd);
        TimeListener {
            reciever: rcv,
            current: self.current_state.clone(),
        }
    }

    fn notify_listeners(&mut self, seeked: bool) {
        let mut i = 0;
        while i < self.listeners.len() {
            let listener = &mut self.listeners[i];

            let signal = NotifySignal {
                new_state: self.current_state.clone(),
                has_seeked: seeked,
            };

            match listener.send(signal) {
                Ok(_) => i += 1,
                Err(_e) => {
                    // The listener has been dropped, so we remove the sender
                    self.listeners.remove(i);
                }
            }
        }
    }

    pub fn toggle_pause(&mut self) {
        let now = self.get_time();
        match self.current_state {
            TimerState::Paused { .. } => {
                self.current_state = TimerState::Running {
                    continue_time: Instant::now(),
                    time_offset: now,
                };
            }
            TimerState::Running { .. } => {
                self.current_state = TimerState::Paused { time_offset: now };
            }
        }

        self.notify_listeners(false);
    }

    pub fn pause(&mut self) {
        let now = self.get_time();
        self.current_state = TimerState::Paused { time_offset: now };
        self.notify_listeners(false);
    }

    pub fn play(&mut self) {
        let now = self.get_time();
        self.current_state = TimerState::Running {
            continue_time: Instant::now(),
            time_offset: now,
        };
        self.notify_listeners(false);
    }

    pub fn seek(&mut self, time: Duration) {
        self.current_state = TimerState::Running {
            continue_time: Instant::now(),
            time_offset: time,
        };
        self.notify_listeners(true);
    }
}

pub struct TimeListener {
    reciever: crossbeam_channel::Receiver<NotifySignal>,
    current: TimerState,
}

#[must_use]
pub enum WaitResult {
    Ok,
    Paused,
    Seeked(Duration),
    Killed,
}

#[must_use]
pub enum UnpauseWaitResult {
    Unpaused,
    UnpausedAndSeeked(Duration),
    Killed,
}

#[must_use]
pub enum SeekWaitResult {
    UnpausedAndSeeked(Duration),
    Killed,
}

impl TimeListener {
    pub fn is_paused(&self) -> bool {
        self.current.is_paused()
    }

    pub fn wait_until(&mut self, time: Duration) -> WaitResult {
        let curr_time = self.current.get_time();
        if curr_time >= time {
            return WaitResult::Ok;
        }

        // TODO: Maybe find a more reliable way to wait while still reading?
        let result = self.reciever.recv_timeout(time - curr_time);

        match result {
            Ok(signal) => {
                self.current = signal.new_state;
                if signal.has_seeked {
                    WaitResult::Seeked(self.current.get_time())
                } else if self.current.is_paused() {
                    WaitResult::Paused
                } else {
                    WaitResult::Ok
                }
            }
            Err(error) => match error {
                crossbeam_channel::RecvTimeoutError::Timeout => WaitResult::Ok,
                crossbeam_channel::RecvTimeoutError::Disconnected => WaitResult::Killed,
            },
        }
    }

    pub fn wait_until_unpause(&mut self) -> UnpauseWaitResult {
        if !self.current.is_paused() {
            return UnpauseWaitResult::Unpaused;
        }

        let mut seeked = None;

        loop {
            let result = self.reciever.recv();

            match result {
                Ok(signal) => {
                    self.current = signal.new_state;
                    if signal.has_seeked {
                        seeked = Some(self.current.get_time());
                    }

                    if !self.current.is_paused() {
                        if let Some(seeked) = seeked {
                            return UnpauseWaitResult::UnpausedAndSeeked(seeked);
                        } else {
                            return UnpauseWaitResult::Unpaused;
                        }
                    }
                }
                Err(_) => return UnpauseWaitResult::Killed,
            }
        }
    }

    pub fn wait_until_seeked(&mut self) -> SeekWaitResult {
        let mut seeked = false;
        loop {
            let result = self.reciever.recv();

            match result {
                Ok(signal) => {
                    self.current = signal.new_state;
                    if signal.has_seeked {
                        seeked = true;
                    }

                    if seeked && !self.current.is_paused() {
                        return SeekWaitResult::UnpausedAndSeeked(self.current.get_time());
                    }
                }
                Err(_) => return SeekWaitResult::Killed,
            }
        }
    }
}
