//! Auditioning a sound from the library: plays decoded audio through its own output, separate from the timeline player.

use crate::media::stream::SAMPLE_RATE;
use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player};
use std::num::NonZero;

#[derive(Default)]
pub struct SoundPreview {
    // The device must outlive the player or the sound stops.
    _device: Option<MixerDeviceSink>,
    player: Option<Player>,
}

impl SoundPreview {
    pub fn play(&mut self, samples: Vec<f32>) {
        self.stop();
        let Ok(device) = DeviceSinkBuilder::open_default_sink() else {
            return;
        };
        let player = Player::connect_new(device.mixer());
        player.append(SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(SAMPLE_RATE).unwrap(),
            samples,
        ));
        (self._device, self.player) = (Some(device), Some(player));
    }

    pub fn stop(&mut self) {
        self.player = None;
        self._device = None;
    }

    pub fn is_playing(&self) -> bool {
        self.player.as_ref().is_some_and(|p| !p.empty())
    }
}
