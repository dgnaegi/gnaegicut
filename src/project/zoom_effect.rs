//! The zoom treatments an item can have.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum ZoomEffect {
    None,
    In,
    Out,
    Pulse,
    /// A hard punch in that overshoots and springs back.
    Slam,
}

impl ZoomEffect {
    pub const ALL: [ZoomEffect; 5] = [
        ZoomEffect::None,
        ZoomEffect::In,
        ZoomEffect::Out,
        ZoomEffect::Pulse,
        ZoomEffect::Slam,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ZoomEffect::None => "None",
            ZoomEffect::In => "Punch in",
            ZoomEffect::Out => "Punch out",
            ZoomEffect::Pulse => "Beat",
            ZoomEffect::Slam => "Slam in",
        }
    }
}
