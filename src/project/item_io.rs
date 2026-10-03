//! What an item hands to ffmpeg and to the change detection.

use super::Item;
use std::hash::Hasher;

impl Item {
    /// ffmpeg input options that open exactly the trimmed part of this item.
    pub fn input_args(&self) -> Vec<String> {
        let (len, path) = (format!("{:.3}", self.len()), self.path.clone());
        if self.kind.is_still() {
            let looped = ["-loop", "1", "-framerate", "30", "-t"].map(String::from);
            looped.into_iter().chain([len, "-i".into(), path]).collect()
        } else {
            [
                "-ss".into(),
                format!("{:.3}", self.start),
                "-t".into(),
                len,
                "-i".into(),
                path,
            ]
            .into()
        }
    }

    /// Feeds everything that changes how this item looks or sounds into `h`.
    pub fn hash_into(&self, h: &mut impl Hasher) {
        h.write(self.path.as_bytes());
        for v in [self.start, self.end, self.at, self.cut] {
            h.write_u64(v.to_bits());
        }
        for v in [
            self.x,
            self.y,
            self.scale,
            self.rotation,
            self.zoom,
            self.amount,
            self.shake,
            self.volume,
            self.fade_in,
            self.fade_out,
        ] {
            h.write_u32(v.to_bits());
        }
        h.write_u8(self.effect as u8);
        h.write_u8(self.enhance as u8);
        h.write_u8(self.transition as u8);
        h.write_u8(self.outro.map_or(u8::MAX, |t| t as u8));
        h.write_u8(self.look as u8);
        h.write_u8(self.reversed as u8);
        self.crop.hash_into(h);
        self.edge.hash_into(h);
        if let Some(j) = self.join {
            h.write_u8(j.effect as u8);
            h.write_u32(j.len.to_bits());
        }
    }
}
