//! Real-time playback: audio is the master clock, video frames are streamed and dropped to keep up.
//!
//! Starting is quick because nothing blocks: the audio device is opened once at startup (`audio_out`), the sound is
//! streamed in short chunks and the clock starts as soon as the first chunk and the first frame are there.

use crate::media::stream::{self, FPS, SAMPLE_RATE};
use crate::project::Project;
use rodio::Player as Sink;
use rodio::buffer::SamplesBuffer;
use rodio::mixer::Mixer;
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
    audio: Receiver<Vec<f32>>,
    first: Option<Vec<u8>>,
    clock: Option<Instant>,
    shown: u64,
    base: f64,
    /// The edit this playback was started from; if the project no longer matches, it is stale.
    pub fingerprint: u64,
    sink: Option<Sink>,
}

impl Player {
    /// Starts decoding `project.tail_from(base)`. Playback begins once the first sound chunk and frame are ready.
    /// Without a `mixer` (no audio device) it plays silently.
    pub fn start(project: &Project, base: f64, w: u32, h: u32, mixer: Option<&Mixer>) -> Self {
        let rest = project.tail_from(base);
        let sink = mixer.map(|m| {
            let s = Sink::connect_new(m);
            s.pause();
            s
        });
        Self {
            frames: stream::video(&rest, w, h),
            audio: stream::audio_chunks(&rest),
            first: None,
            clock: None,
            shown: 0,
            base,
            fingerprint: project.fingerprint(),
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
        while let Ok(chunk) = self.audio.try_recv() {
            self.queue(chunk); // the rest of the sound follows while the first chunks play
        }
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

    fn queue(&self, samples: Vec<f32>) {
        if let Some(sink) = &self.sink {
            let (ch, rate) = (NonZero::new(2).unwrap(), NonZero::new(SAMPLE_RATE).unwrap());
            sink.append(SamplesBuffer::new(ch, rate, samples));
        }
    }

    /// Starts the clock (and sound) once the first sound chunk and the first frame have arrived.
    fn try_begin(&mut self) -> Result<(), String> {
        if self.first.is_none() {
            self.first = self.frames.try_recv().ok();
        }
        if self.first.is_none() {
            return Ok(());
        }
        match self.audio.try_recv() {
            Ok(chunk) => self.queue(chunk),
            Err(TryRecvError::Empty) => return Ok(()), // the sound is still starting up
            Err(TryRecvError::Disconnected) => {}      // no sound beats no playback
        }
        if let Some(sink) = &self.sink {
            sink.play();
        }
        self.clock = Some(Instant::now());
        self.shown = 1; // the first frame counts as shown at t = 0
        Ok(())
    }
}

impl Drop for Player {
    /// Pausing must be instant: silence the queued sound right now instead of letting it drain.
    fn drop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
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
        project.add(0, video("play", "blue", (1280, 720), 3, true));

        let device = rodio::DeviceSinkBuilder::open_default_sink().ok();
        let (w, h) = crate::preview::size(crate::project::Aspect::Vertical); // the size playback really uses
        let mut player = Player::start(&project, 0.0, w, h, device.as_ref().map(|d| d.mixer()));
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

    /// Pressing play must not wait for the whole edit: starting is quick however long the project is.
    #[test]
    fn playback_starts_quickly_even_for_a_long_project() {
        let mut project = Project::default();
        project.add(0, video("long", "blue", (640, 360), 40, true));
        project.add(1, crate::media::testutil::sound("long_music", 40));

        let asked = Instant::now();
        let mut player = Player::start(&project, 0.0, 90, 160, None);
        let returned = asked.elapsed();
        assert!(
            returned < Duration::from_millis(150),
            "start() must not block the UI: {returned:?}"
        );

        while player.clock.is_none() {
            player.poll().unwrap();
            assert!(asked.elapsed() < Duration::from_secs(4), "playback never started");
            std::thread::sleep(Duration::from_millis(5));
        }
        let to_start = asked.elapsed();
        println!("start() returned in {returned:?}; playback began after {to_start:?}");
        assert!(
            to_start < Duration::from_millis(1800),
            "time until playback begins: {to_start:?}"
        );
    }
}
