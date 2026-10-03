//! Small shared types of the app: background-job messages, what is selected, and inspector tabs.

use crate::fonts::Fonts;
use crate::project::Caption;
use crate::sounds::Sound;

/// Messages background jobs send back to the UI thread.
pub enum Event {
    Status(String),
    /// The background job (export, transcription) has finished, whatever its result.
    Done,
    /// New captions and the timing fingerprint of the edit they were made from.
    Captions(Vec<Caption>, u64),
    Fonts(Box<Fonts>),
    /// A library thumbnail: source path and RGBA pixels.
    Thumb(String, Vec<u8>, u32, u32),
    /// Loudness peaks of a file's sound, for the timeline.
    Waveform(String, Vec<f32>),
    SoundResults(crate::sounds::Source, Result<Vec<Sound>, String>),
    /// A sound is downloaded and decoded; play it.
    SoundPcm(String, Vec<f32>),
    /// A sound is downloaded and probed; put it on the timeline.
    SoundItem(String, Box<crate::project::Item>),
    SoundFailed(String, String),
    /// The list of animated stickers arrived.
    StickerIndex(Result<Vec<crate::stickers::Sticker>, String>),
    /// A sticker's preview: code and RGBA pixels with the size.
    StickerPreview(String, Result<(Vec<u8>, u32, u32), String>),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Selection {
    None,
    Item(u64),
    Captions,
    /// One caption picked on the timeline: drag its ends to change its length, Backspace removes it.
    Caption(usize),
    /// A lane picked by clicking its number; Backspace or Delete removes it if it is empty.
    Track(usize),
}

/// Tabs of the inspector. Not every tab exists for every selection.
#[derive(Clone, Copy, PartialEq)]
pub enum Tab {
    Text,
    Place,
    Motion,
    Audio,
    Time,
    Transition,
}
