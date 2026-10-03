//! Loading the animated stickers and putting one on the timeline.

use crate::app::{App, Event};
use crate::media::probe::probe;
use crate::stickers::{self, CREDIT, Sticker};
use eframe::egui::{ColorImage, TextureOptions};

/// Stickers loop to fill at least this long when added.
const MIN_SECS: f64 = 3.0;
/// A sticker starts this big, a share of what would fit the frame.
const START_SCALE: f32 = 0.4;
/// Previews fetched at the same time.
const AT_ONCE: usize = 12;

impl App {
    /// Starts loading the list of stickers the first time the tab is opened.
    pub fn ensure_stickers(&mut self) {
        if std::mem::replace(&mut self.sounds.stickers.asked_index, true) {
            return;
        }
        self.spawn(|tx| {
            let _ = tx.send(Event::StickerIndex(stickers::load_index()));
        });
    }

    /// Fetches a sticker's preview picture unless it is there or on its way.
    pub fn request_preview(&mut self, sticker: &Sticker) {
        let ui = &mut self.sounds.stickers;
        if ui.previews.contains_key(&sticker.code)
            || ui.loading.len() >= AT_ONCE
            || !ui.loading.insert(sticker.code.clone())
        {
            return;
        }
        let sticker = sticker.clone();
        self.spawn(move |tx| {
            let _ = tx.send(Event::StickerPreview(
                sticker.code.clone(),
                stickers::load_preview(&sticker),
            ));
        });
    }

    /// Downloads a sticker and puts it on the timeline at the playhead, on the first free track.
    pub fn add_sticker(&mut self, sticker: Sticker) {
        let key = format!("sticker-{}", sticker.code);
        if !self.sounds.busy.insert(key.clone()) {
            return;
        }
        self.spawn(move |tx| {
            let result = stickers::download(&sticker)
                .and_then(|path| probe(&path))
                .and_then(|mut item| {
                    let once = item.end;
                    if once <= 0.0 {
                        return Err("this sticker has no length".to_string());
                    }
                    item.name = sticker.words.clone();
                    (item.looped, item.scale) = (true, START_SCALE); // `src_len` stays the length of one turn
                    item.end = once * (MIN_SECS / once).ceil().max(1.0); // whole turns of the animation
                    item.credit = Some(CREDIT.into());
                    Ok(item)
                });
            let _ = tx.send(match result {
                Ok(item) => Event::SoundItem(key, Box::new(item)),
                Err(e) => Event::SoundFailed(key, e),
            });
        });
    }

    pub(crate) fn on_sticker_event(&mut self, event: Event) {
        let ui = &mut self.sounds.stickers;
        match event {
            Event::StickerIndex(Ok(list)) => ui.all = list,
            Event::StickerIndex(Err(e)) => ui.error = Some(e),
            Event::StickerPreview(code, result) => {
                ui.loading.remove(&code);
                if let Ok((rgba, w, h)) = result {
                    let image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                    ui.previews
                        .insert(code, self.ctx.load_texture("sticker", image, TextureOptions::LINEAR));
                }
            }
            _ => {}
        }
    }
}
