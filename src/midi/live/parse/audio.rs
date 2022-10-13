use std::sync::{atomic::Ordering, Arc};

use atomic_float::AtomicF64;
use crossbeam_channel::Receiver;

use crate::midi::shared::audio::CompressedAudio;

use super::{ThreadManager, TrackEventBatch};

pub struct AudioParserResult {
    pub reciever: Receiver<CompressedAudio>,
    pub manager: ThreadManager,
}

pub fn init_audio_manager(blocks: Receiver<Arc<TrackEventBatch>>) -> AudioParserResult {
    let (sender, reciever) = crossbeam_channel::unbounded();
    let parse_time_outer = Arc::new(AtomicF64::default());

    let parse_time = parse_time_outer.clone();
    let join_handle = std::thread::spawn(move || {
        for block in CompressedAudio::build_blocks(blocks.into_iter()) {
            parse_time.store(block.time, Ordering::Relaxed);
            let res = sender.send(block);
            if res.is_err() {
                break;
            }
        }
    });

    AudioParserResult {
        reciever,
        manager: ThreadManager {
            handle: join_handle,
            parse_time: parse_time_outer,
        },
    }
}
