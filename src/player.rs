//! Real-time playback: audio is the master clock, video frames are streamed and dropped to keep up.

use crate::media::stream::{self, FPS, SAMPLE_RATE};
use crate::project::Project;
use rodio::buffer::SamplesBuffer;
use rodio::{DeviceSinkBuilder, MixerDeviceSink, Player as Sink};
use std::num::NonZero;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;

/// What changed since the last poll.
#[derive(Default)]
pub struct Update {
    pub frame: Option<Vec<u8>>,
    pub finished: bool,
}

pub struct Player {
    frames: Receiver<Vec<u8>>,
    audio: Receiver<Result<Vec<f32>, String>>,
    first: Option<Vec<u8>>,
    clock: Option<Instant>,
    shown: u64,
    base: f64,
    /// The edit this playback was started from; if the project no longer matches, it is stale.
    pub fingerprint: u64,
    // Kept alive for the duration of playback; dropping them stops the sound.
    _device: Option<MixerDeviceSink>,
    sink: Option<Sink>,
}

impl Player {
    /// Starts decoding `project.tail_from(base)`. Playback begins once audio and the first frame are ready.
    pub fn start(project: &Project, base: f64, w: u32, h: u32) -> Self {
        let rest = project.tail_from(base);
        let (tx, audio) = std::sync::mpsc::channel();
        let for_audio = rest.clone();
        std::thread::spawn(move || {
            let _ = tx.send(stream::audio(&for_audio));
        });
        let device = DeviceSinkBuilder::open_default_sink().ok();
        let sink = device.as_ref().map(|d| {
            let s = Sink::connect_new(d.mixer());
            s.pause();
            s
        });
        Self {
            frames: stream::video(&rest, w, h),
            audio,
            first: None,
            clock: None,
            shown: 0,
            base,
            fingerprint: project.fingerprint(),
            _device: device,
            sink,
        }
    }

    /// Timeline time currently being played.
    pub fn position(&self) -> f64 {
        self.base + self.clock.map_or(0.0, |c| c.elapsed().as_secs_f64())
    }

    pub fn poll(&mut self) -> Result<Update, String> {
        let Some(clock) = self.clock else {
            return self.try_begin().map(|()| Update::default());
        };
        let target = (clock.elapsed().as_secs_f64() * FPS) as u64;
        let mut update = Update {
            frame: self.first.take(),
            ..Update::default()
        };
        while self.shown <= target {
            match self.frames.try_recv() {
                Ok(f) => {
                    self.shown += 1;
                    update.frame = Some(f);
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    update.finished = true;
                    break;
                }
            }
        }
        Ok(update)
    }

    /// Starts the clock (and sound) once both audio and a first frame have arrived.
    fn try_begin(&mut self) -> Result<(), String> {
        if self.first.is_none() {
            self.first = self.frames.try_recv().ok();
        }
        if self.first.is_none() {
            return Ok(());
        }
        let samples = match self.audio.try_recv() {
            Ok(r) => r.unwrap_or_default(), // no sound beats no playback
            Err(_) => return Ok(()),        // audio still rendering
        };
        if let Some(sink) = &self.sink {
            let (ch, rate) = (NonZero::new(2).unwrap(), NonZero::new(SAMPLE_RATE).unwrap());
            sink.append(SamplesBuffer::new(ch, rate, samples));
            sink.play();
        }
        self.clock = Some(Instant::now());
        self.shown = 1; // the first frame counts as shown at t = 0
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::testutil::video;
    use std::time::Duration;

    /// Plays a real clip in real time (audible!): `cargo test -- --ignored`.
    /// Also checks smoothness: no gap between displayed frames above ~100 ms.
    #[test]
    #[ignore]
    fn plays_in_real_time_smoothly_and_finishes() {
        let mut project = Project::default();
        project.add(0, video("play", "blue", (640, 360), 3, true));

        let mut player = Player::start(&project, 0.0, 90, 160);
        let began = Instant::now();
        let (mut frames, mut last, mut worst) = (0u32, None::<Instant>, Duration::ZERO);
        loop {
            let u = player.poll().unwrap();
            if u.frame.is_some() {
                frames += 1;
                if let Some(prev) = last {
                    worst = worst.max(prev.elapsed());
                }
                last = Some(Instant::now());
            }
            if u.finished {
                break;
            }
            assert!(began.elapsed() < Duration::from_secs(8), "never finished");
            std::thread::sleep(Duration::from_millis(4));
        }
        let secs = player.position();
        assert!((2.8..3.6).contains(&secs), "position {secs}");
        assert!(frames >= 80, "only {frames} distinct frames shown"); // ~90 expected
        assert!(worst < Duration::from_millis(100), "worst gap {worst:?}");
    }
}
