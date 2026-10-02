//! The catalogue of transitions between two clips and what each does to the clips on either side.

use super::kinds::{Phase, Transition};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum JoinEffect {
    Dissolve,
    Dip,
    Flash,
    Glitch,
    WhipLeft,
    WhipRight,
    Spin,
    PushLeft,
    PushRight,
    PushUp,
    PushDown,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    Circle,
    Zoom,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Join {
    pub effect: JoinEffect,
    pub len: f32,
}

/// What a join does to the clip before it (`out`) and the clip after it (`enter`).
pub struct Links {
    pub out: Phase,
    pub enter: Phase,
    pub overlaps: bool,
}

impl JoinEffect {
    /// In the order they are offered: the punchy ones first.
    pub const ALL: [JoinEffect; 17] = [
        JoinEffect::WhipLeft,
        JoinEffect::WhipRight,
        JoinEffect::Flash,
        JoinEffect::Glitch,
        JoinEffect::Spin,
        JoinEffect::Zoom,
        JoinEffect::Circle,
        JoinEffect::Dissolve,
        JoinEffect::WipeLeft,
        JoinEffect::WipeRight,
        JoinEffect::WipeUp,
        JoinEffect::WipeDown,
        JoinEffect::PushLeft,
        JoinEffect::PushRight,
        JoinEffect::PushUp,
        JoinEffect::PushDown,
        JoinEffect::Dip,
    ];

    pub fn label(self) -> &'static str {
        match self {
            JoinEffect::Dissolve => "Dissolve",
            JoinEffect::Dip => "Dip to black",
            JoinEffect::Flash => "Flash",
            JoinEffect::Glitch => "Glitch",
            JoinEffect::WhipLeft => "Whip left",
            JoinEffect::WhipRight => "Whip right",
            JoinEffect::Spin => "Spin",
            JoinEffect::PushLeft => "Push left",
            JoinEffect::PushRight => "Push right",
            JoinEffect::PushUp => "Push up",
            JoinEffect::PushDown => "Push down",
            JoinEffect::WipeLeft => "Wipe left",
            JoinEffect::WipeRight => "Wipe right",
            JoinEffect::WipeUp => "Wipe up",
            JoinEffect::WipeDown => "Wipe down",
            JoinEffect::Circle => "Circle",
            JoinEffect::Zoom => "Zoom",
        }
    }

    pub fn links(self, len: f32) -> Links {
        let phase = |effect, len| Phase { effect, len };
        let none = phase(Transition::None, len);
        let (out, enter, overlaps) = match self {
            JoinEffect::Dissolve => (none, phase(Transition::Fade, len), true),
            JoinEffect::Dip => (
                phase(Transition::Fade, len / 2.0),
                phase(Transition::Fade, len / 2.0),
                false,
            ),
            // These cut in the middle and hide the cut with an effect on both sides, so they do not overlap.
            JoinEffect::Flash => (
                phase(Transition::Flash, len / 2.0),
                phase(Transition::Flash, len / 2.0),
                false,
            ),
            JoinEffect::Glitch => (
                phase(Transition::Glitch, len / 2.0),
                phase(Transition::Glitch, len / 2.0),
                false,
            ),
            JoinEffect::WhipLeft => (phase(Transition::WhipLeft, len), phase(Transition::WhipLeft, len), true),
            JoinEffect::WhipRight => (
                phase(Transition::WhipRight, len),
                phase(Transition::WhipRight, len),
                true,
            ),
            JoinEffect::Spin => (none, phase(Transition::Spin, len), true),
            JoinEffect::PushLeft => (
                phase(Transition::SlideLeft, len),
                phase(Transition::SlideLeft, len),
                true,
            ),
            JoinEffect::PushRight => (
                phase(Transition::SlideRight, len),
                phase(Transition::SlideRight, len),
                true,
            ),
            JoinEffect::PushUp => (phase(Transition::SlideUp, len), phase(Transition::SlideUp, len), true),
            JoinEffect::PushDown => (
                phase(Transition::SlideDown, len),
                phase(Transition::SlideDown, len),
                true,
            ),
            JoinEffect::WipeLeft => (none, phase(Transition::WipeLeft, len), true),
            JoinEffect::WipeRight => (none, phase(Transition::WipeRight, len), true),
            JoinEffect::WipeUp => (none, phase(Transition::WipeUp, len), true),
            JoinEffect::WipeDown => (none, phase(Transition::WipeDown, len), true),
            JoinEffect::Circle => (none, phase(Transition::Circle, len), true),
            JoinEffect::Zoom => (none, phase(Transition::Zoom, len), true),
        };
        Links { out, enter, overlaps }
    }
}
