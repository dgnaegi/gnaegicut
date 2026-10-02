use super::freeze::FREEZE_SECS;
use super::{Item, Kind, Project};

fn clip(len: f64) -> Item {
    Item::new("a.mp4".into(), "a".into(), Kind::Video, (100, 100), len, true)
}

fn still() -> Option<Item> {
    Some(Item::new(
        "s.png".into(),
        "s".into(),
        Kind::Image,
        (100, 100),
        0.0,
        false,
    ))
}

#[test]
fn freeze_inserts_a_still_and_pushes_the_rest_later() {
    let mut p = Project::default();
    let a = p.add(0, clip(4.0));
    let b = p.add(0, clip(2.0));
    p.get_mut(b).unwrap().at = 4.0;
    let mut asked = 0.0;
    let s = p
        .freeze(a, 1.5, |t| {
            asked = t;
            still()
        })
        .unwrap();
    assert_eq!(asked, 1.5, "the frame comes from the playhead");
    let (st, right) = (p.get(s).unwrap(), p.items().find(|i| i.at > 3.0 && i.id != b).unwrap());
    assert_eq!((st.at, st.len()), (1.5, FREEZE_SECS));
    assert_eq!((right.at, right.start), (1.5 + FREEZE_SECS, 1.5));
    assert_eq!(p.get(b).unwrap().at, 4.0 + FREEZE_SECS);
    assert_eq!(p.total(), 6.0 + FREEZE_SECS);
}

#[test]
fn freeze_outside_the_clip_or_without_a_frame_changes_nothing() {
    let mut p = Project::default();
    let a = p.add(0, clip(4.0));
    assert!(p.freeze(a, 4.0, |_| still()).is_none());
    assert!(p.freeze(a, 2.0, |_| None).is_none());
    assert_eq!(p.items().count(), 1);
}

#[test]
fn splitting_a_reversed_clip_keeps_the_playing_order() {
    let mut p = Project::default();
    let a = p.add(0, clip(4.0));
    p.get_mut(a).unwrap().reversed = true;
    let b = p.split(a, 1.0).unwrap();
    let (left, right) = (p.get(a).unwrap(), p.get(b).unwrap());
    assert_eq!((left.start, left.end), (3.0, 4.0), "plays the end of the source first");
    assert_eq!((right.start, right.end), (0.0, 3.0));
}
