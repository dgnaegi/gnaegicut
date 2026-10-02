//! Small enums describing an item.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Video,
    Image,
    Text,  // rendered to an image by `text.rs`, then treated like one
    Audio, // sound only: music, effects, voice-over
}

impl Kind {
    /// Images and text are single frames held for a duration.
    pub fn is_still(self) -> bool {
        matches!(self, Kind::Image | Kind::Text)
    }

    /// Everything except audio draws something on the frame.
    pub fn is_visual(self) -> bool {
        self != Kind::Audio
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum ZoomEffect {
    None,
    In,
    Out,
    Pulse,
}

impl ZoomEffect {
    pub const ALL: [ZoomEffect; 4] = [ZoomEffect::None, ZoomEffect::In, ZoomEffect::Out, ZoomEffect::Pulse];

    pub fn label(self) -> &'static str {
        match self {
            ZoomEffect::None => "None",
            ZoomEffect::In => "Punch in",
            ZoomEffect::Out => "Punch out",
            ZoomEffect::Pulse => "Beat",
        }
    }
}

/// What happens to an item while it enters or leaves. Fade, Slide, Flash and Glitch work for both; the rest only
/// bring an item in (they come from transitions between two clips).
#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum Transition {
    /// Picture untouched; only the sound fades. Used under another clip's reveal.
    None,
    Fade,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    /// A slide with motion blur: fast and snappy.
    WhipLeft,
    WhipRight,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    Circle,
    Zoom,
    /// Comes in spinning and settling, zooming and fading in.
    Spin,
    /// Burns out to white (or in from white).
    Flash,
    /// RGB split, noise and jitter.
    Glitch,
}

impl Transition {
    /// The choices for an item's own intro / outro.
    pub const BASIC: [Transition; 7] = [
        Transition::Fade,
        Transition::SlideLeft,
        Transition::SlideRight,
        Transition::SlideUp,
        Transition::SlideDown,
        Transition::Flash,
        Transition::Glitch,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Transition::None => "None",
            Transition::Fade => "Fade",
            Transition::SlideLeft => "Slide left",
            Transition::SlideRight => "Slide right",
            Transition::SlideUp => "Slide up",
            Transition::SlideDown => "Slide down",
            Transition::WhipLeft => "Whip left",
            Transition::WhipRight => "Whip right",
            Transition::WipeLeft | Transition::WipeRight | Transition::WipeUp | Transition::WipeDown => "Wipe",
            Transition::Circle => "Circle",
            Transition::Zoom => "Zoom",
            Transition::Spin => "Spin",
            Transition::Flash => "Flash",
            Transition::Glitch => "Glitch",
        }
    }

    /// Unit direction of travel for slides and whips: (1, 0) moves right, (0, -1) moves up.
    pub fn direction(self) -> (f32, f32) {
        match self {
            Transition::SlideLeft | Transition::WhipLeft => (-1.0, 0.0),
            Transition::SlideRight | Transition::WhipRight => (1.0, 0.0),
            Transition::SlideUp => (0.0, -1.0),
            Transition::SlideDown => (0.0, 1.0),
            _ => (0.0, 0.0),
        }
    }

    /// Wipes and the circle reveal an item through a moving mask.
    pub fn is_reveal(self) -> bool {
        matches!(
            self,
            Transition::WipeLeft
                | Transition::WipeRight
                | Transition::WipeUp
                | Transition::WipeDown
                | Transition::Circle
        )
    }

    /// Whips smear the picture sideways while it moves.
    pub fn blurs(self) -> bool {
        matches!(self, Transition::WhipLeft | Transition::WhipRight)
    }
}

/// A colour treatment applied to the item's picture.
#[derive(Clone, Copy, PartialEq, Default, Hash, Serialize, Deserialize)]
pub enum Look {
    #[default]
    None,
    Vivid,
    Warm,
    Cool,
    Mono,
    Vhs,
    Vignette,
}

impl Look {
    pub const ALL: [Look; 7] = [
        Look::None,
        Look::Vivid,
        Look::Warm,
        Look::Cool,
        Look::Mono,
        Look::Vhs,
        Look::Vignette,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Look::None => "None",
            Look::Vivid => "Vivid",
            Look::Warm => "Warm",
            Look::Cool => "Cool",
            Look::Mono => "Mono",
            Look::Vhs => "VHS",
            Look::Vignette => "Vignette",
        }
    }

    /// ffmpeg filters for the look.
    pub fn filters(self) -> Vec<&'static str> {
        match self {
            Look::None => vec![],
            Look::Vivid => vec!["eq=saturation=1.45:contrast=1.12:brightness=0.02"],
            Look::Warm => vec!["colorbalance=rs=0.08:bs=-0.08:rm=0.06:bm=-0.06", "eq=saturation=1.15"],
            Look::Cool => vec!["colorbalance=rs=-0.08:bs=0.1:rm=-0.05:bm=0.07", "eq=saturation=1.1"],
            Look::Mono => vec!["hue=s=0", "eq=contrast=1.15"],
            Look::Vhs => vec![
                "chromashift=crh=6:cbh=-6",
                "noise=alls=14:allf=t",
                "eq=saturation=1.25:contrast=1.05",
                "vignette=angle=PI/5",
            ],
            Look::Vignette => vec!["vignette=angle=PI/4"],
        }
    }
}

/// One entrance or exit: an effect and how long it takes, in seconds.
#[derive(Clone, Copy, PartialEq)]
pub struct Phase {
    pub effect: Transition,
    pub len: f32,
}
