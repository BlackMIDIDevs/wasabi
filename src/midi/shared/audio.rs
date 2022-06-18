use std::sync::Arc;

use gen_iter::GenIter;
use midi_toolkit::{
    events::{Event, MIDIEvent, MIDIEventEnum},
    sequence::event::EventBatch,
};

pub struct CompressedAudio {
    pub time: f64,
    data: Vec<u8>,
    control_only_data: Option<Vec<u8>>,
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
        let mut control_builder_vec: Vec<u8> = Vec::new();
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
                            let events = &[head, e.key, e.velocity];
                            builder_vec.extend_from_slice(events);
                        }
                        Event::NoteOff(e) => {
                            let head = EV_OFF | e.channel;
                            let events = &[head, e.key];
                            builder_vec.extend_from_slice(events);
                        }
                        Event::PolyphonicKeyPressure(e) => {
                            let head = EV_POLYPHONIC | e.channel;
                            let events = &[head, e.key, e.velocity];
                            builder_vec.extend_from_slice(events);
                        }
                        Event::ControlChange(e) => {
                            let head = EV_CONTROL | e.channel;
                            let events = &[head, e.controller, e.value];
                            builder_vec.extend_from_slice(events);
                            control_builder_vec.extend_from_slice(events);
                        }
                        Event::ProgramChange(e) => {
                            let head = EV_PROGRAM | e.channel;
                            let events = &[head, e.program];
                            builder_vec.extend_from_slice(events);
                            control_builder_vec.extend_from_slice(events);
                        }
                        Event::ChannelPressure(e) => {
                            let head = EV_CHAN_PRESSURE | e.channel;
                            let events = &[head, e.pressure];
                            builder_vec.extend_from_slice(events);
                            control_builder_vec.extend_from_slice(events);
                        }
                        Event::PitchWheelChange(e) => {
                            let head = EV_PITCH_BEND | e.channel;
                            let value = e.pitch + 8192;
                            let events = &[head, (value & 0x7F) as u8, ((value >> 7) & 0x7F) as u8];
                            builder_vec.extend_from_slice(events);
                            control_builder_vec.extend_from_slice(events);
                        }
                        _ => {}
                    }
                }

                let mut new_vec = Vec::with_capacity(builder_vec.len());
                new_vec.append(&mut builder_vec);

                let new_control_vec = if control_builder_vec.len() > 0 {
                    let mut new_control_vec = Vec::with_capacity(control_builder_vec.len());
                    new_control_vec.append(&mut control_builder_vec);
                    Some(new_control_vec)
                } else {
                    None
                };

                yield CompressedAudio {
                    data: new_vec,
                    control_only_data: new_control_vec,
                    time,
                };
            }
        })
    }

    pub fn iter_events<'a>(&'a self) -> impl 'a + Iterator<Item = u32> {
        CompressedAudio::iter_events_from_vec(self.data.iter().cloned())
    }

    pub fn iter_control_events<'a>(&'a self) -> impl 'a + Iterator<Item = u32> {
        CompressedAudio::iter_events_from_vec(self.control_only_data.iter().flatten().cloned())
    }

    pub fn iter_events_from_vec<'a>(
        mut iter: impl 'a + Iterator<Item = u8>,
    ) -> impl 'a + Iterator<Item = u32> {
        GenIter(move || {
            while let Some(next) = iter.next() {
                let ev = next & 0xF0;
                let val = match ev {
                    EV_OFF | EV_PROGRAM | EV_CHAN_PRESSURE => {
                        let val2 = iter.next().unwrap() as u32;
                        (next as u32) | (val2 << 8)
                    }
                    EV_ON | EV_POLYPHONIC | EV_CONTROL | EV_PITCH_BEND => {
                        let val2 = iter.next().unwrap() as u32;
                        let val3 = iter.next().unwrap() as u32;
                        (next as u32) | (val2 << 8) | (val3 << 16)
                    }
                    _ => panic!("Can't reach {:#x}", next),
                };

                yield val;
            }
        })
    }
}
