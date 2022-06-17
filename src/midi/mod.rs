mod ram;
use enum_dispatch::enum_dispatch;
use palette::convert::FromColorUnclamped;

pub use ram::InRamMIDIFile;

use self::ram::view::InRamNoteViews;

/// A struct that represents the view range of a midi screen render
#[derive(Debug, Clone, Copy, Default)]
pub struct MIDIViewRange {
    pub start: f64,
    pub end: f64,
}

impl MIDIViewRange {
    pub fn new(start: f64, end: f64) -> Self {
        MIDIViewRange { start, end }
    }

    pub fn length(&self) -> f64 {
        self.end - self.start
    }
}

pub struct MIDIColor(u32);

impl MIDIColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        let num = (b as u32) | ((g as u32) << 8) | ((r as u32) << 16);
        MIDIColor(num)
    }

    pub fn new_from_hue(hue: f64) -> Self {
        let hsv = palette::Hsv::new(hue, 1.0, 0.5);
        let rgb = palette::rgb::Rgb::from_color_unclamped(hsv);
        Self::new(
            (rgb.red * 255.0) as u8,
            (rgb.green * 255.0) as u8,
            (rgb.blue * 255.0) as u8,
        )
    }

    pub fn new_vec_for_tracks(tracks: usize) -> Vec<Self> {
        let count = tracks * 16;

        let mut vec = Vec::with_capacity(count);
        for i in 0..count {
            vec.push(MIDIColor::new_from_hue(i as f64 * 360.0 / 16.0 * 15.0));
        }

        vec
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// The basic shared functions in a midi file. The columns related functions are
/// inside the [`MIDIFile`] trait.
#[enum_dispatch]
pub trait MIDIFileBase {
    fn midi_length(&self) -> Option<f64>;
    fn parsed_up_to(&self) -> Option<f64>;
}

/// This trait contains a function to retrieve the column view of the midi
pub trait MIDIFile: MIDIFileBase {
    type ColumnsViews: MIDINoteViews;

    fn get_column_views<'a>(&'a self) -> Self::ColumnsViews;
}

#[enum_dispatch]
pub trait MIDINoteViewsBase {
    fn shift_view_range(&mut self, new_range: MIDIViewRange);
    fn allows_seeking_backward(&self) -> bool;
}

pub trait MIDINoteViews {
    type View<'a>: 'a + MIDINoteColumnView
    where
        Self: 'a;

    fn get_column<'a>(&'a self, key: usize) -> Self::View<'a>;
    fn range<'a>(&'a self) -> MIDIViewRange;
}

pub trait MIDINoteColumnView: Send {
    type Iter<'a>: 'a + ExactSizeIterator<Item = DisplacedMIDINote> + Send
    where
        Self: 'a;

    fn iterate_displaced_notes<'a>(&'a self) -> Self::Iter<'a>;

    fn adjust_view_range(&mut self, range: MIDIViewRange);
}

pub struct DisplacedMIDINote {
    pub start: f32,
    pub len: f32,
    pub color: u32,
}

#[enum_dispatch(MIDIFileBase)]
pub enum MIDIFileUnion {
    InRam(ram::InRamMIDIFile),
}

impl MIDIFileUnion {
    pub fn get_views(&self) -> MIDIFileViewsUnion {
        match self {
            Self::InRam(file) => MIDIFileViewsUnion::InRam(file.get_column_views()),
        }
    }
}

#[enum_dispatch(MIDINoteViewsBase)]
pub enum MIDIFileViewsUnion {
    InRam(InRamNoteViews),
}

impl MIDIFileViewsUnion {}
