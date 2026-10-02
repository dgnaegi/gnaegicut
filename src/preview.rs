//! Still-frame preview of the full composite. A worker renders only the newest request, so scrubbing
//! never lags behind. The texture is updated in place: re-allocating it every frame makes playback stutter.

use crate::media::frame::frame;
use crate::project::{Aspect, Project};
use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::sync::mpsc::{Receiver, Sender, channel};

const DIVISOR: u32 = 3; // preview is the export size / 3

/// Preview resolution for an output aspect.
pub fn size(aspect: Aspect) -> (u32, u32) {
    let (w, h) = aspect.size();
    (w / DIVISOR, h / DIVISOR)
}

struct Request {
    project: Project,
    t: f64,
    w: u32,
    h: u32,
}

pub struct Preview {
    pub texture: Option<TextureHandle>,
    asked: Option<(u64, i64)>,
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
    pub fn show(&mut self, ctx: &Context, project: &Project, t: f64) {
        while let Ok((px, w, h)) = self.rx.try_recv() {
            self.upload(ctx, &px, w, h);
        }
        let key = (project.fingerprint(), (t * 20.0) as i64);
        if self.asked != Some(key) {
            let (w, h) = size(project.aspect);
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
