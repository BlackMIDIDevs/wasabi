use std::sync::Arc;

use gen_iter::GenIter;
use midi_toolkit::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    sequence::event::EventBatch,
};

pub struct CompressedAudio {
    pub time: f64,
    data: Vec<u8>,
}

const EV_OFF: u8 = 0x80;
const EV_ON: u8 = 0x90;
const EV_POLYPHONIC: u8 = 0xA0;
const EV_CONTROL: u8 = 0xB0;
const EV_PROGRAM: u8 = 0xC0;
const EV_CHAN_PRESSURE: u8 = 0xD0;
const EV_PITCH_BEND: u8 = 0xE0;

impl CompressedAudio {
    pub fn build_blocks<Iter: Iterator<Item = Arc<EventBatch<f64, E>>>, E: MIDIEventEnum<f64>>(
        iter: Iter,
    ) -> impl Iterator<Item = CompressedAudio> {
        let mut builder_vec: Vec<u8> = Vec::new();
        GenIter(move || {
            let mut time = 0.0;

            for block in iter {
                time += block.delta();

                let min_len: usize = block.count() * 3;

                builder_vec.reserve(min_len);
                builder_vec.clear();

                for event in block.iter() {
                    match event.as_event() {
                        Event::NoteOn(e) => {
                            let head = EV_ON | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.key);
                            builder_vec.push(e.velocity);
                        }
                        Event::NoteOff(e) => {
                            let head = EV_OFF | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.key);
                        }
                        Event::PolyphonicKeyPressure(e) => {
                            let head = EV_POLYPHONIC | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.key);
                            builder_vec.push(e.velocity);
                        }
                        Event::ControlChange(e) => {
                            let head = EV_CONTROL | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.controller);
                            builder_vec.push(e.value);
                        }
                        Event::ProgramChange(e) => {
                            let head = EV_PROGRAM | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.program);
                        }
                        Event::ChannelPressure(e) => {
                            let head = EV_CHAN_PRESSURE | e.channel;
                            builder_vec.push(head);
                            builder_vec.push(e.pressure);
                        }
                        Event::PitchWheelChange(e) => {
                            let head = EV_PITCH_BEND | e.channel;
                            builder_vec.push(head);
                            builder_vec.push((e.pitch & 0x7F) as u8);
                            builder_vec.push(((e.pitch >> 7) & 0x7F) as u8);
                        }
                        _ => {}
                    }
                }

                let mut new_vec = Vec::with_capacity(builder_vec.len());
                new_vec.append(&mut builder_vec);

                yield CompressedAudio {
                    data: new_vec,
                    time,
                };
            }
        })
    }

    pub fn iter_events<'a>(&'a self) -> impl 'a + Iterator<Item = u32> {
        GenIter(move || {
            let mut iter = self.data.iter();
            while let Some(next) = iter.next() {
                let ev = next & 0xF0;
                let val = match ev {
                    EV_OFF | EV_PROGRAM | EV_CHAN_PRESSURE => {
                        let val2 = *iter.next().unwrap() as u32;
                        (*next as u32) | (val2 << 8)
                    }
                    EV_ON | EV_POLYPHONIC | EV_CONTROL | EV_PITCH_BEND => {
                        let val2 = *iter.next().unwrap() as u32;
                        let val3 = *iter.next().unwrap() as u32;
                        (*next as u32) | (val2 << 8) | (val3 << 16)
                    }
                    _ => panic!("Can't reach {:#x}", next),
                };

                yield val;
            }
        })
    }
}
