use super::*;

fn clip(len: f64, at: f64) -> Item {
    let mut i = Item::new("a.mp4".into(), "a".into(), Kind::Video, (1920, 1080), len, true);
    i.at = at;
    i
}

/// A: 0..4, B: 4..6, C: 6..9 on one track.
fn trio() -> (Project, u64, u64, u64) {
    let mut p = Project::default();
    let (a, b, c) = (
        p.add(0, clip(4.0, 0.0)),
        p.add(0, clip(2.0, 4.0)),
        p.add(0, clip(3.0, 6.0)),
    );
    (p, a, b, c)
}

fn join(effect: JoinEffect, len: f32) -> Option<Join> {
    Some(Join { effect, len })
}

#[test]
fn predecessor_needs_touching_or_overlapping_clips() {
    let (mut p, a, b, c) = trio();
    assert_eq!(p.predecessor(b), Some(a));
    assert_eq!(p.predecessor(a), None);
    p.get_mut(c).unwrap().at = 7.0; // a one second gap after B
    assert_eq!(p.predecessor(c), None);
}

#[test]
fn overlapping_effect_pulls_the_clip_and_everything_after_it_earlier() {
    let (mut p, _, b, c) = trio();
    p.set_join(b, join(JoinEffect::Dissolve, 1.0));
    assert!((p.get(b).unwrap().at - 3.0).abs() < 1e-9);
    assert!((p.get(c).unwrap().at - 5.0).abs() < 1e-9, "later clips follow");
    assert!((p.total() - 8.0).abs() < 1e-9);
}

#[test]
fn removing_or_changing_a_join_restores_the_geometry() {
    let (mut p, _, b, c) = trio();
    p.set_join(b, join(JoinEffect::WipeLeft, 1.0));
    p.set_join(b, join(JoinEffect::Dip, 1.0)); // dip does not overlap
    assert!((p.get(b).unwrap().at - 4.0).abs() < 1e-9 && (p.get(c).unwrap().at - 6.0).abs() < 1e-9);
    p.set_join(b, join(JoinEffect::Dissolve, 0.5));
    p.set_join(b, join(JoinEffect::Dissolve, 1.5)); // longer: pulled earlier still
    assert!((p.get(b).unwrap().at - 2.5).abs() < 1e-9);
    p.set_join(b, None);
    assert!((p.get(b).unwrap().at - 4.0).abs() < 1e-9 && (p.get(c).unwrap().at - 6.0).abs() < 1e-9);
    assert!(p.get(b).unwrap().join.is_none());
}

#[test]
fn length_is_limited_by_the_shorter_clip() {
    let (mut p, _, b, _) = trio(); // B is 2 s long
    p.set_join(b, join(JoinEffect::Circle, 10.0));
    let len = p.get(b).unwrap().join.unwrap().len;
    assert!((len - 1.9).abs() < 1e-4, "clamped to B's length minus 0.1, got {len}");
}

#[test]
fn a_join_needs_a_clip_before_it() {
    let (mut p, a, ..) = trio();
    p.set_join(a, join(JoinEffect::Dissolve, 1.0));
    assert!(p.get(a).unwrap().join.is_none());
}

#[test]
fn resolved_gives_both_clips_their_phases_and_is_idempotent() {
    let (mut p, a, b, _) = trio();
    p.set_join(b, join(JoinEffect::PushLeft, 1.0));
    let r = p.resolved();
    let (out, enter) = (r.get(a).unwrap().link_out.unwrap(), r.get(b).unwrap().link_in.unwrap());
    assert!(out.effect == Transition::SlideLeft && enter.effect == Transition::SlideLeft);
    assert!((out.len - 1.0).abs() < 1e-6 && (enter.len - 1.0).abs() < 1e-6);
    assert!(r.linked && r.resolved().get(b).unwrap().link_in.is_some());
    assert!(p.get(b).unwrap().link_in.is_none(), "the original is untouched");
}

#[test]
fn dragged_apart_clips_lose_their_transition() {
    let (mut p, _, b, _) = trio();
    p.set_join(b, join(JoinEffect::Dissolve, 1.0));
    p.get_mut(b).unwrap().at = 4.5; // moved away: no overlap left
    assert!(p.resolved().get(b).unwrap().link_in.is_none());
    p.get_mut(b).unwrap().at = 3.5; // overlap shrank to 0.5: the effect shortens to fit
    let len = p.resolved().get(b).unwrap().link_in.unwrap().len;
    assert!((len - 0.5).abs() < 1e-6, "got {len}");
}

#[test]
fn playing_from_inside_the_transition_keeps_it() {
    let (mut p, a, b, _) = trio();
    p.set_join(b, join(JoinEffect::WipeRight, 1.0)); // overlap 3..4
    let rest = p.tail_from(3.5);
    let item = rest.get(b).unwrap();
    assert!((item.cut - 0.5).abs() < 1e-9 && item.link_in.unwrap().effect == Transition::WipeRight);
    assert!(
        (item.link_in.unwrap().len - 1.0).abs() < 1e-6,
        "full length, so the animation continues from half way"
    );
    assert!(rest.get(a).is_some());
}

#[test]
fn joins_survive_the_fingerprint() {
    let (mut p, _, b, _) = trio();
    let before = p.fingerprint();
    p.set_join(b, join(JoinEffect::Zoom, 0.5));
    assert_ne!(before, p.fingerprint());
}
