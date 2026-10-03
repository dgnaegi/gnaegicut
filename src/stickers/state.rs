//! What the Stickers tab shows and is waiting for.

use super::Sticker;
use eframe::egui::TextureHandle;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct StickersUi {
    pub query: String,
    /// Every sticker, once the list has arrived.
    pub all: Vec<Sticker>,
    pub asked_index: bool,
    pub error: Option<String>,
    pub previews: HashMap<String, TextureHandle>,
    /// Previews being fetched right now (by code), so at most a few run at once.
    pub loading: HashSet<String>,
}
