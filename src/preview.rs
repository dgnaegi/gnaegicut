//! Still-frame preview of the full composite. A worker renders only the newest request, so scrubbing
//! never lags behind. The texture is updated in place: re-allocating it every frame makes playback stutter.

use crate::media::frame::frame;
use crate::project::{Aspect, Project};
use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::sync::mpsc::{Receiver, Sender, channel};

const DIVISOR: u32 = 2; // preview is the export size / 2: sharp enough to judge text, light enough to play in real time

/// Preview resolution for an output aspect.
pub fn size(aspect: Aspect) -> (u32, u32) {
    let (w, h) = aspect.size();
    ((w / DIVISOR) & !1, (h / DIVISOR) & !1) // even, as yuv420 requires: 4:5 would give 675 rows
}

/// The size for watching full screen: a long side of 1280 px. Sharper than the editing preview, and still light enough to
/// play in real time.
pub fn fullscreen_size(aspect: Aspect) -> (u32, u32) {
    const LONG_SIDE: f32 = 1280.0;
    let (w, h) = aspect.size();
    let k = LONG_SIDE / w.max(h) as f32;
    (((w as f32 * k) as u32) & !1, ((h as f32 * k) as u32) & !1)
}

struct Request {
    project: Project,
    t: f64,
    w: u32,
    h: u32,
}

pub struct Preview {
    pub texture: Option<TextureHandle>,
    asked: Option<(u64, i64, (u32, u32))>,
    tx: Sender<Request>,
    rx: Receiver<(Vec<u8>, u32, u32)>,
}

impl Preview {
    pub fn new(ctx: Context) -> Self {
        let (tx, req_rx) = channel::<Request>();
        let (frame_tx, rx) = channel();
        std::thread::spawn(move || {
            while let Ok(mut r) = req_rx.recv() {
                while let Ok(newer) = req_rx.try_recv() {
                    r = newer;
                }
                if let Some(px) = frame(&r.project, r.t, r.w, r.h) {
                    let _ = frame_tx.send((px, r.w, r.h));
                    ctx.request_repaint();
                }
            }
        });
        Self {
            texture: None,
            asked: None,
            tx,
            rx,
        }
    }

    /// Shows a frame that arrived from somewhere else (the player).
    pub fn set_frame(&mut self, ctx: &Context, rgba: &[u8], w: u32, h: u32) {
        self.upload(ctx, rgba, w, h);
        self.asked = None; // a paused view must re-request its exact frame
    }

    fn upload(&mut self, ctx: &Context, rgba: &[u8], w: u32, h: u32) {
        let img = ColorImage::from_rgba_premultiplied([w as usize, h as usize], rgba);
        match &mut self.texture {
            Some(tex) if tex.size() == img.size => tex.set(img, TextureOptions::LINEAR),
            _ => self.texture = Some(ctx.load_texture("preview", img, TextureOptions::LINEAR)),
        }
    }

    /// Uploads any finished frame, then asks for the composite at `t` unless it was already asked for.
    pub fn show(&mut self, ctx: &Context, project: &Project, t: f64, size: (u32, u32)) {
        while let Ok((px, w, h)) = self.rx.try_recv() {
            self.upload(ctx, &px, w, h);
        }
        let key = (project.fingerprint(), (t * 20.0) as i64, size);
        if self.asked != Some(key) {
            let (w, h) = size;
            let _ = self.tx.send(Request {
                project: project.snapshot(),
                t,
                w,
                h,
            });
            self.asked = Some(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_sizes_are_even_and_sharp_enough_for_text() {
        for aspect in Aspect::ALL {
            let (w, h) = size(aspect);
            assert!(w % 2 == 0 && h % 2 == 0, "{}: {w}x{h}", aspect.label());
        }
        assert_eq!(size(Aspect::Vertical), (540, 960), "half the export size");
        assert_eq!(size(Aspect::Portrait), (540, 674));
    }

    #[test]
    fn the_full_screen_preview_is_larger_even_and_keeps_the_shape() {
        for aspect in Aspect::ALL {
            let (w, h) = fullscreen_size(aspect);
            assert!(
                w % 2 == 0 && h % 2 == 0 && w.max(h) == 1280,
                "{}: {w}x{h}",
                aspect.label()
            );
            let (ew, eh) = aspect.size();
            assert!(
                ((w as f32 / h as f32) - ew as f32 / eh as f32).abs() < 0.01,
                "shape of {}",
                aspect.label()
            );
            assert!(
                w * h > size(aspect).0 * size(aspect).1,
                "sharper than the editing preview"
            );
        }
    }
}
