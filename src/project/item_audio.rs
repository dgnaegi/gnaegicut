//! Sound of an item: the filter chain that processes it, and the volume envelope its fades produce.

use super::Item;

impl Item {
    /// Volume from the intro and outro fades, 0..1, `local` seconds after the item starts on the timeline.
    /// The timeline draws its waveform with this, so what you see follows what you hear.
    pub fn fade_gain_at(&self, local: f64) -> f32 {
        let (fade_in, fade_out) = (self.enter().len as f64, self.exit().len as f64);
        let rise = if fade_in > 0.0 {
            (local / fade_in).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let fall = if fade_out > 0.0 {
            ((self.len() - local) / fade_out).clamp(0.0, 1.0)
        } else {
            1.0
        };
        (rise * fall) as f32
    }

    /// Clip audio chain: normalise format, optional voice clean-up, gain, fades, then delay into place.
    pub fn audio_filters(&self, rate: u32) -> String {
        let mut f = vec![
            format!("aresample={rate}"),
            "aformat=channel_layouts=stereo".into(),
            "asetpts=PTS-STARTPTS".into(),
        ];
        if self.reversed {
            f.extend(["areverse", "asetpts=PTS-STARTPTS"].map(String::from));
        }
        if self.enhance {
            // Cut rumble, reduce steady noise, even out loudness, then normalise to -16 LUFS (social media standard).
            f.extend(
                [
                    "highpass=f=80",
                    "afftdn=nr=12:nf=-35",
                    "acompressor=threshold=-20dB:ratio=3:attack=5:release=100",
                ]
                .map(String::from),
            );
            f.extend(["loudnorm=I=-16:TP=-1.5:LRA=11".into(), format!("aresample={rate}")]);
        }
        if (self.volume - 1.0).abs() > 1e-3 {
            f.push(format!("volume={:.3}", self.volume));
        }
        let (fade_in, fade_out) = (self.enter().len, self.exit().len);
        if fade_in > 0.0 || fade_out > 0.0 {
            // Same as the picture: fades run in item time, so a cut-off front does not restart them.
            f.push(format!("asetpts=PTS-STARTPTS+{:.3}/TB", self.cut));
            if fade_in > 0.0 {
                f.push(format!("afade=t=in:st=0:d={fade_in:.3}"));
            }
            if fade_out > 0.0 {
                let start = (self.len() + self.cut - fade_out as f64).max(0.0);
                f.push(format!("afade=t=out:st={start:.3}:d={fade_out:.3}"));
            }
            f.push("asetpts=PTS-STARTPTS".into());
        }
        let ms = (self.at * 1000.0).round() as i64;
        f.push(format!("adelay={ms}|{ms}"));
        f.join(",")
    }
}
