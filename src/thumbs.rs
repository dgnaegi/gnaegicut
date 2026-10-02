//! Library thumbnails: decoded on worker threads, kept as textures.

use crate::app::Event;
use crate::media::frame::thumbnail;
use crate::project::{Kind, MediaRef};
use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

pub const BOX: u32 = 96;

/// The thumbnail size for a source: fitted into a square, even dimensions.
pub fn fit(w: u32, h: u32) -> (u32, u32) {
    let k = (BOX as f32 / w.max(1) as f32).min(BOX as f32 / h.max(1) as f32);
    let even = |v: f32| ((v.round() as u32).max(2)) & !1;
    (even(w as f32 * k), even(h as f32 * k))
}

#[derive(Default)]
pub struct Thumbs {
    textures: HashMap<String, TextureHandle>,
    asked: HashSet<String>,
}

impl Thumbs {
    /// The texture for `media` if it is ready; otherwise starts loading it once.
    pub fn get(&mut self, media: &MediaRef, tx: &Sender<Event>, ctx: &Context) -> Option<&TextureHandle> {
        if !self.textures.contains_key(&media.path) && self.asked.insert(media.path.clone()) {
            let (tx, ctx, m) = (tx.clone(), ctx.clone(), media.clone());
            std::thread::spawn(move || {
                let size = fit(m.src_w, m.src_h);
                let at = if m.kind == Kind::Video {
                    (m.len / 2.0).min(1.0)
                } else {
                    0.0
                };
                if let Some(px) = thumbnail(&m.path, at, size) {
                    let _ = tx.send(Event::Thumb(m.path, px, size.0, size.1));
                    ctx.request_repaint();
                }
            });
        }
        self.textures.get(&media.path)
    }

    pub fn insert(&mut self, ctx: &Context, path: String, rgba: &[u8], w: u32, h: u32) {
        let img = ColorImage::from_rgba_premultiplied([w as usize, h as usize], rgba);
        self.textures
            .insert(path, ctx.load_texture("thumb", img, TextureOptions::LINEAR));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::testutil::{image, video};

    #[test]
    fn fits_into_the_box_keeping_aspect() {
        assert_eq!(fit(1920, 1080), (96, 54));
        assert_eq!(fit(1080, 1920), (54, 96));
        assert_eq!(fit(64, 64), (96, 96));
    }

    #[test]
    fn thumbnails_decode_for_videos_and_images() {
        let v = video("thumb", "blue", (640, 360), 2, false);
        let (w, h) = fit(v.src_w, v.src_h);
        let px = thumbnail(&v.path, 1.0, (w, h)).unwrap();
        assert_eq!(px.len(), (w * h * 4) as usize);
        assert!(
            px[0] < 20 && px[1] < 20 && px[2] > 235,
            "blue within YUV rounding: {:?}",
            &px[..3]
        );
        let img = image("thumb_red", "red", (200, 100));
        assert!(thumbnail(&img.path, 0.0, fit(200, 100)).is_some());
    }
}
