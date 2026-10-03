//! One placed piece of media: a video or image on a track, with its trim, position and zoom.

use super::kinds::Phase;
use super::{Crop, Edge, Join, Kind, Look, Transition, ZoomEffect};
use serde::{Deserialize, Serialize};

pub const IMAGE_SECS: f64 = 3.0; // default length of a dropped image
const IMAGE_MAX_SECS: f64 = 600.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub path: String,
    pub name: String,
    pub kind: Kind,
    pub src_w: u32,
    pub src_h: u32,
    pub src_len: f64,
    pub has_audio: bool,
    /// Trim inside the source, seconds. Images use `0..duration`.
    pub start: f64,
    pub end: f64,
    /// Position on the timeline, seconds.
    pub at: f64,
    /// Centre of the item as a fraction of the frame; (0.5, 0.5) is centred.
    pub x: f32,
    pub y: f32,
    /// 1.0 = the whole item fits inside the frame. Larger values crop at the frame edge.
    pub scale: f32,
    /// Clockwise rotation in degrees around the item's centre.
    pub rotation: f32,
    /// Magnification of the content inside the item's box (a punch-in), >= 1.
    pub zoom: f32,
    pub effect: ZoomEffect,
    pub amount: f32,
    /// Camera shake, 0 (none) to 1 (violent).
    #[serde(default)]
    pub shake: f32,
    /// Smooths out shaky footage (video only).
    #[serde(default)]
    pub stabilize: bool,
    /// The part of the picture that is cut away on each side.
    #[serde(default)]
    pub crop: Crop,
    /// A soft fade and a border at the edge of the picture.
    #[serde(default)]
    pub edge: Edge,
    /// Colour treatment.
    #[serde(default)]
    pub look: Look,
    /// Plays the trimmed part backwards (picture and sound).
    #[serde(default)]
    pub reversed: bool,
    /// Seconds already cut off the front by `Project::tail_from`, so zoom effects stay continuous.
    pub cut: f64,
    /// Text items only: content and look. `path` points at the rendered PNG.
    pub text: String,
    pub font: String,   // PostScript name, resolved by `fonts.rs`
    pub font_size: f32, // px on a 1080-wide frame
    pub color: [u8; 3],
    pub outline: bool,
    /// A solid coloured bar behind the text (a lower third).
    #[serde(default)]
    pub bar: bool,
    /// Audio: gain (1.0 = unchanged) and one-click voice clean-up.
    pub volume: f32,
    pub enhance: bool,
    /// Entrance and exit lengths in seconds; the picture uses `transition`, the sound always fades.
    pub fade_in: f32,
    pub fade_out: f32,
    pub transition: Transition,
    /// The outro; `None` means the same as the intro (how older projects were saved).
    #[serde(default)]
    pub outro: Option<Transition>,
    /// Where a downloaded sound came from and how to credit it (a licence requirement for most free sounds).
    #[serde(default)]
    pub credit: Option<String>,
    /// A transition from the previous clip on this track into this one.
    #[serde(default)]
    pub join: Option<Join>,
    /// How this item enters / leaves because of its joins; filled in by `Project::resolved`, never saved.
    #[serde(skip)]
    pub link_in: Option<Phase>,
    #[serde(skip)]
    pub link_out: Option<Phase>,
}

impl Item {
    pub fn new(path: String, name: String, kind: Kind, size: (u32, u32), len: f64, has_audio: bool) -> Self {
        let (src_len, end) = if kind.is_still() {
            (IMAGE_MAX_SECS, IMAGE_SECS)
        } else {
            (len, len)
        };
        Self {
            id: 0,
            path,
            name,
            kind,
            src_w: size.0.max(1),
            src_h: size.1.max(1),
            src_len,
            has_audio,
            start: 0.0,
            end,
            at: 0.0,
            x: 0.5,
            y: 0.5,
            scale: 1.0,
            rotation: 0.0,
            zoom: 1.0,
            effect: ZoomEffect::None,
            amount: 0.3,
            shake: 0.0,
            stabilize: false,
            crop: Crop::default(),
            edge: Edge::default(),
            look: Look::None,
            reversed: false,
            cut: 0.0,
            text: String::new(),
            font: String::new(),
            font_size: 96.0,
            color: [255, 255, 255],
            outline: true,
            bar: false,
            volume: 1.0,
            enhance: false,
            fade_in: 0.0,
            fade_out: 0.0,
            transition: Transition::Fade,
            outro: None,
            credit: None,
            join: None,
            link_in: None,
            link_out: None,
        }
    }

    pub fn len(&self) -> f64 {
        self.end - self.start
    }

    pub fn end_at(&self) -> f64 {
        self.at + self.len()
    }

    /// How the item enters: a join's effect if it has one, otherwise its own intro.
    pub fn enter(&self) -> Phase {
        self.link_in.unwrap_or(Phase {
            effect: self.transition,
            len: self.fade_in,
        })
    }

    /// The item's own outro. Projects from before intro and outro were separate leave it unset and use the intro's
    /// effect; one that only works as an entrance (wipe, zoom, spin) fades out instead.
    pub fn outro_effect(&self) -> Transition {
        match self.outro.unwrap_or(self.transition) {
            t if t.intro_only() => Transition::Fade,
            t => t,
        }
    }

    /// How the item leaves: a join's effect if it has one, otherwise its own outro.
    pub fn exit(&self) -> Phase {
        self.link_out.unwrap_or(Phase {
            effect: self.outro_effect(),
            len: self.fade_out,
        })
    }
}
