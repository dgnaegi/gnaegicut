//! Waveforms for the timeline: computed once per file on a worker thread and kept as peak lists.

use crate::app::Event;
use crate::media::waveform::peaks;
use eframe::egui::Context;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::Sender;

#[derive(Default)]
pub struct Waveforms {
    peaks: HashMap<String, Vec<f32>>,
    asked: HashSet<String>,
}

impl Waveforms {
    /// Starts computing the waveform of `path` unless that has already been done or asked for.
    pub fn request(&mut self, path: &str, tx: &Sender<Event>, ctx: &Context) {
        if !self.asked.insert(path.to_string()) {
            return;
        }
        let (path, tx, ctx) = (path.to_string(), tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            if let Ok(list) = peaks(&path) {
                let _ = tx.send(Event::Waveform(path, list));
                ctx.request_repaint();
            }
        });
    }

    pub fn get(&self, path: &str) -> Option<&[f32]> {
        self.peaks.get(path).map(Vec::as_slice)
    }

    pub fn insert(&mut self, path: String, list: Vec<f32>) {
        self.peaks.insert(path, list);
    }
}
