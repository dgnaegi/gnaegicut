//! Clips on one track never lie on top of each other. The only overlap allowed is the one a transition
//! (`Join`) creates, and only for as long as the transition lasts. Overlays (text, sound) are not affected.

use super::{Item, Kind, Project};

const SLACK: f64 = 1e-3;

fn solid(item: &Item) -> bool {
    matches!(item.kind, Kind::Video | Kind::Image)
}

/// Whether two clips lie on top of each other beyond what the later clip's transition accounts for.
fn clash(a: &Item, b: &Item) -> bool {
    let (first, second) = if a.at <= b.at { (a, b) } else { (b, a) };
    let overlap = first.end_at() - second.at;
    overlap > SLACK && !second.join.is_some_and(|j| overlap <= j.len as f64 + SLACK)
}

impl Project {
    /// The start nearest to `at` where `item` (ignoring itself) fits on `track` without clashing, found by
    /// trying to sit right behind or right in front of every clip there.
    pub fn fit_at(&self, track: usize, item: &Item, at: f64) -> f64 {
        let at = at.max(0.0);
        let Some(lane) = self.tracks.get(track).filter(|_| solid(item)) else {
            return at;
        };
        let others: Vec<&Item> = lane.items.iter().filter(|o| o.id != item.id && solid(o)).collect();
        let fits = |t: f64| {
            let mut probe = item.clone();
            probe.at = t;
            t >= 0.0 && others.iter().all(|o| !clash(o, &probe))
        };
        if fits(at) {
            return at;
        }
        others
            .iter()
            .flat_map(|o| [o.end_at(), o.at - item.len()])
            .filter(|&t| fits(t))
            .min_by(|a, b| (a - at).abs().total_cmp(&(b - at).abs()))
            .unwrap_or(at)
    }

    /// How far the edges of clip `id` may reach before touching its neighbours: (earliest start, latest end).
    pub fn room_around(&self, id: u64) -> (f64, f64) {
        let Some((ti, _)) = self.find(id) else {
            return (0.0, f64::MAX);
        };
        let Some(me) = self.get(id).filter(|i| solid(i)) else {
            return (0.0, f64::MAX);
        };
        let lane = || self.tracks[ti].items.iter().filter(|o| o.id != id && solid(o));
        let before = lane().filter(|o| o.at < me.at).map(Item::end_at).fold(0.0, f64::max);
        let after = lane().filter(|o| o.at >= me.at).map(|o| o.at).fold(f64::MAX, f64::min);
        let own = me.join.map_or(0.0, |j| j.len as f64);
        let later = lane()
            .filter(|o| o.at >= me.at)
            .min_by(|a, b| a.at.total_cmp(&b.at))
            .and_then(|o| o.join)
            .map_or(0.0, |j| j.len as f64);
        (
            (before - own).max(0.0),
            if after == f64::MAX { after } else { after + later },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(at: f64, len: f64) -> Item {
        let mut i = Item::new("/x.mp4".into(), "x".into(), Kind::Video, (160, 90), len, false);
        i.at = at;
        i
    }

    fn project() -> (Project, u64, u64) {
        let mut p = Project::default();
        let a = p.add(0, clip(0.0, 3.0));
        let b = p.add(0, clip(5.0, 2.0));
        (p, a, b)
    }

    #[test]
    fn a_clip_dropped_on_another_goes_to_the_nearest_free_spot() {
        let (p, ..) = project();
        let new = clip(0.0, 1.0);
        assert_eq!(p.fit_at(0, &new, 3.5), 3.5, "free gap stays where it is");
        assert_eq!(p.fit_at(0, &new, 1.0), 3.0, "behind the first clip");
        assert_eq!(
            p.fit_at(0, &new, 4.6),
            4.0,
            "in front of the second, since it is closer"
        );
        assert_eq!(
            p.fit_at(0, &clip(0.0, 3.0), 3.5),
            7.0,
            "too long for the gap: after everything"
        );
    }

    #[test]
    fn overlays_and_empty_lanes_are_not_restricted() {
        let (p, ..) = project();
        let mut text = clip(0.0, 1.0);
        text.kind = Kind::Text;
        assert_eq!(p.fit_at(0, &text, 1.0), 1.0);
        assert_eq!(p.fit_at(3, &clip(0.0, 1.0), 1.0), 1.0);
    }

    #[test]
    fn room_around_a_clip_ends_at_its_neighbours() {
        let (p, a, b) = project();
        assert_eq!(p.room_around(a), (0.0, 5.0));
        assert_eq!(p.room_around(b), (3.0, f64::MAX));
    }
}
