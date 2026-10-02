//! Changing single captions on the timeline: resizing either end, moving, removing. Captions never overlap one
//! another and are never shorter than `MIN_CAPTION`, however far they are dragged.

use crate::project::Project;

pub const MIN_CAPTION: f64 = 0.1;

impl Project {
    /// The room around caption `index`: where the previous caption ends and the next one starts.
    fn caption_room(&self, index: usize) -> (f64, f64) {
        let Some(me) = self.captions.get(index) else {
            return (0.0, f64::INFINITY);
        };
        let others = || {
            self.captions
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != index)
                .map(|(_, c)| c)
        };
        let before = others()
            .filter(|c| c.end <= me.start + 1e-9)
            .map(|c| c.end)
            .fold(0.0, f64::max);
        let after = others()
            .filter(|c| c.start >= me.end - 1e-9)
            .map(|c| c.start)
            .fold(f64::INFINITY, f64::min);
        (before, after)
    }

    /// Moves the start of caption `index` (making it longer or shorter). Returns false if there is no such caption.
    pub fn resize_caption_start(&mut self, index: usize, start: f64) -> bool {
        let (before, _) = self.caption_room(index);
        let Some(c) = self.captions.get_mut(index) else {
            return false;
        };
        c.start = start.clamp(before, c.end - MIN_CAPTION);
        true
    }

    /// Moves the end of caption `index`.
    pub fn resize_caption_end(&mut self, index: usize, end: f64) -> bool {
        let (_, after) = self.caption_room(index);
        let Some(c) = self.captions.get_mut(index) else {
            return false;
        };
        c.end = end.clamp(c.start + MIN_CAPTION, after);
        true
    }

    /// Moves caption `index` so it starts at `start`, keeping its length, stopping at its neighbours.
    pub fn move_caption(&mut self, index: usize, start: f64) -> bool {
        let (before, after) = self.caption_room(index);
        let Some(c) = self.captions.get_mut(index) else {
            return false;
        };
        let len = c.end - c.start;
        c.start = start.clamp(before, (after - len).max(before));
        c.end = c.start + len;
        true
    }

    pub fn remove_caption(&mut self, index: usize) -> bool {
        (index < self.captions.len())
            .then(|| self.captions.remove(index))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::Caption;

    /// a: 1-2 s, b: 3-4 s, c: 4-6 s (b and c touch).
    fn project() -> Project {
        let caption = |text: &str, start, end| Caption {
            start,
            end,
            text: text.into(),
        };
        Project {
            captions: vec![caption("a", 1.0, 2.0), caption("b", 3.0, 4.0), caption("c", 4.0, 6.0)],
            ..Default::default()
        }
    }

    fn span(p: &Project, i: usize) -> (f64, f64) {
        (p.captions[i].start, p.captions[i].end)
    }

    #[test]
    fn either_end_can_be_dragged_to_make_a_caption_longer_or_shorter() {
        let mut p = project();
        assert!(p.resize_caption_end(0, 2.6));
        assert!(p.resize_caption_start(0, 0.5));
        assert_eq!(span(&p, 0), (0.5, 2.6));
        p.resize_caption_end(0, 1.5);
        assert_eq!(span(&p, 0), (0.5, 1.5), "shorter again");
    }

    #[test]
    fn a_caption_stops_at_its_neighbours_and_at_zero() {
        let mut p = project();
        p.resize_caption_end(0, 50.0);
        assert_eq!(span(&p, 0), (1.0, 3.0), "stops where the next caption starts");
        let mut p = project();
        p.resize_caption_start(1, -5.0);
        assert_eq!(span(&p, 1), (2.0, 4.0), "stops where the one before ends");
        p.resize_caption_start(0, -5.0);
        assert_eq!(span(&p, 0).0, 0.0, "and never goes before the start of the timeline");
        p.resize_caption_start(2, 0.0);
        assert_eq!(
            span(&p, 2),
            (4.0, 6.0),
            "touching captions stay touching, not overlapping"
        );
    }

    #[test]
    fn a_caption_never_becomes_shorter_than_the_minimum() {
        let mut p = project();
        p.resize_caption_end(0, 0.0);
        assert!((span(&p, 0).1 - 1.0 - MIN_CAPTION).abs() < 1e-9);
        p.resize_caption_start(0, 99.0);
        assert!((span(&p, 0).1 - span(&p, 0).0 - MIN_CAPTION).abs() < 1e-9);
    }

    #[test]
    fn moving_keeps_the_length_and_stops_at_the_neighbours() {
        let mut p = project();
        p.move_caption(0, 10.0);
        assert_eq!(span(&p, 0), (2.0, 3.0), "pushed against the next caption, same length");
        p.move_caption(1, 0.0);
        assert_eq!(span(&p, 1), (3.0, 4.0), "no room to move: it stays");
        let mut p = project();
        p.move_caption(0, 0.2);
        assert_eq!(span(&p, 0), (0.2, 1.2));
    }

    #[test]
    fn removing_takes_out_exactly_that_caption() {
        let mut p = project();
        assert!(p.remove_caption(1));
        assert_eq!(
            p.captions.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(),
            ["a", "c"]
        );
        assert!(!p.remove_caption(9));
        assert!(
            !p.resize_caption_end(9, 1.0) && !p.move_caption(9, 1.0),
            "unknown captions are ignored"
        );
    }
}
