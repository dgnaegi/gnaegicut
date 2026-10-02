//! Where sounds come from.

#[derive(Clone, Copy, PartialEq, Default)]
pub enum Source {
    #[default]
    Effects,
    Music,
}

impl Source {
    pub(super) fn provider(self) -> &'static str {
        match self {
            Source::Effects => "freesound",
            Source::Music => "jamendo",
        }
    }

    /// One-tap searches that suit short-form video.
    pub fn suggestions(self) -> &'static [&'static str] {
        match self {
            Source::Effects => &[
                "whoosh",
                "impact",
                "riser",
                "glitch",
                "pop",
                "click",
                "bass drop",
                "swoosh",
                "applause",
                "laugh",
            ],
            Source::Music => &[
                "lofi",
                "upbeat",
                "phonk",
                "cinematic",
                "chill",
                "trap",
                "house",
                "acoustic",
            ],
        }
    }
}
