use std::sync::{Arc, Mutex};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

use super::{intvec4::IntVector4, tree_serializer::TreeSerializer};

pub struct MidiData {
    pub vec: Vec<IntVector4>,
    pub time: i32,
}

pub enum NoteEvent {
    On {
        time: i32,
        channel_track: i32,
        color: i32,
    },
    Off {
        time: i32,
        channel_track: i32,
        color: i32,
    },
}

pub struct ThreadedTreeSerializers {
    trees: Arc<Mutex<Vec<TreeSerializer>>>,
    rcv: crossbeam_channel::Receiver<Vec<Vec<NoteEvent>>>,
    snd: crossbeam_channel::Sender<Vec<Vec<NoteEvent>>>,
    join: std::thread::JoinHandle<()>,

    current_vec: Vec<Vec<NoteEvent>>,
    cached_event_count: usize,
}

impl ThreadedTreeSerializers {
    fn make_vecs() -> Vec<Vec<NoteEvent>> {
        (0..256).map(|_| Vec::new()).collect()
    }

    pub fn new() -> ThreadedTreeSerializers {
        let trees = (0..256).map(|_| TreeSerializer::new()).collect::<Vec<_>>();
        let trees = Arc::new(Mutex::new(trees));

        let (snd_in, rcv_in) = crossbeam_channel::unbounded::<Vec<Vec<NoteEvent>>>();
        let (snd_back, rcv_back) = crossbeam_channel::unbounded::<Vec<Vec<NoteEvent>>>();

        let trees_thread = trees.clone();
        let handle = std::thread::spawn(move || {
            let mut trees = trees_thread.lock().unwrap();

            for mut vecs in rcv_in.into_iter() {
                vecs.par_iter_mut()
                    .zip(trees.par_iter_mut())
                    .for_each(move |(events, tree)| {
                        for event in events.drain(..) {
                            match event {
                                NoteEvent::On {
                                    time,
                                    channel_track,
                                    color,
                                } => {
                                    tree.start_note(time, channel_track, color);
                                }
                                NoteEvent::Off {
                                    time,
                                    channel_track,
                                    color: _color,
                                } => {
                                    tree.end_note(time, channel_track);
                                }
                            }
                        }
                    });
                snd_back.send(vecs).unwrap();
            }
        });

        snd_in.send(ThreadedTreeSerializers::make_vecs()).unwrap();

        ThreadedTreeSerializers {
            trees,
            rcv: rcv_back,
            snd: snd_in,
            join: handle,

            current_vec: ThreadedTreeSerializers::make_vecs(),
            cached_event_count: 0,
        }
    }

    fn swap_buffers(&mut self) {
        self.cached_event_count = 0;
        let recieved = self.rcv.recv().unwrap();

        let send = std::mem::replace(&mut self.current_vec, recieved);
        self.snd.send(send).unwrap();
    }

    pub fn push_event(&mut self, key: usize, event: NoteEvent) {
        self.current_vec[key].push(event);
        self.cached_event_count += 1;

        if self.cached_event_count > 1024 * 1024 {
            self.swap_buffers();
        }
    }

    pub fn seal(self, time: i32) -> Vec<Vec<IntVector4>> {
        self.snd.send(self.current_vec).unwrap();
        drop(self.snd);

        self.rcv.recv().unwrap();
        self.rcv.recv().unwrap();

        self.join.join().unwrap();

        let trees = Arc::try_unwrap(self.trees).unwrap().into_inner().unwrap();

        let mut serialized = Vec::new();
        for tree in trees.into_iter() {
            let sealed = tree.complete_and_seal(time);
            serialized.push(sealed);
        }

        serialized
    }
}
