//! Auditioning a sound from the library: its own player on the shared audio output, separate from the timeline player.

use crate::media::stream::SAMPLE_RATE;
use rodio::Player;
use rodio::buffer::SamplesBuffer;
use rodio::mixer::Mixer;
use std::num::NonZero;

#[derive(Default)]
pub struct SoundPreview {
    player: Option<Player>,
}

impl SoundPreview {
    /// Plays decoded audio through the app's audio output. Silent if there is no audio device.
    pub fn play(&mut self, mixer: Option<&Mixer>, samples: Vec<f32>) {
        self.stop();
        let Some(mixer) = mixer else { return };
        let player = Player::connect_new(mixer);
        player.append(SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(SAMPLE_RATE).unwrap(),
            samples,
        ));
        self.player = Some(player);
    }

    pub fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
    }

    pub fn is_playing(&self) -> bool {
        self.player.as_ref().is_some_and(|p| !p.empty())
    }
}
