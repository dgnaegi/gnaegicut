//! The audio output device: opened once at startup and kept for the whole session, because opening it on every
//! play press blocks for a noticeable moment and closing it on every pause does too.

use crate::app::App;
use rodio::DeviceSinkBuilder;

impl App {
    pub fn open_audio(&mut self) {
        self.audio = DeviceSinkBuilder::open_default_sink().ok().map(|mut device| {
            device.log_on_drop(false);
            device
        });
    }

    /// Where players send their sound, if there is an audio device.
    pub fn mixer(&self) -> Option<&rodio::mixer::Mixer> {
        self.audio.as_ref().map(|d| d.mixer())
    }
}
