use super::*;

pub(super) fn video(len: f64) -> Item {
    Item::new("a.mp4".into(), "a".into(), Kind::Video, (1920, 1080), len, true)
}

pub(super) fn project() -> (Project, u64, u64) {
    let mut p = Project::default();
    let a = p.add(0, video(4.0));
    let mut b = video(2.0);
    b.at = 4.0;
    let b = p.add(0, b);
    (p, a, b)
}

#[test]
fn total_and_track_end() {
    let (p, ..) = project();
    assert!((p.total() - 6.0).abs() < 1e-9);
    assert!((p.track_end(0) - 6.0).abs() < 1e-9);
    assert_eq!(p.track_end(5), 0.0);
    assert!(Project::default().total() == 0.0);
}

#[test]
fn split_keeps_total_and_rejects_edges() {
    let (mut p, a, _) = project();
    let right = p.split(a, 1.5).unwrap();
    assert!((p.total() - 6.0).abs() < 1e-9);
    assert!((p.get(right).unwrap().start - 1.5).abs() < 1e-9);
    assert!((p.get(a).unwrap().end - 1.5).abs() < 1e-9);
    assert!(p.split(a, 0.05).is_none());
}

#[test]
fn place_moves_between_tracks_and_sorts() {
    let (mut p, a, b) = project();
    p.tracks.push(Default::default());
    p.place(b, 1, 1.0);
    assert_eq!(p.find(b), Some((1, 0)));
    p.place(a, 1, 0.5);
    assert_eq!(p.tracks[1].items[0].id, a); // sorted by start time
    p.place(a, 0, -3.0);
    assert_eq!(p.get(a).unwrap().at, 0.0);
}

#[test]
fn ripple_remove_closes_the_gap_on_its_track_only() {
    let (mut p, a, b) = project(); // a: 0..4, b: 4..6 on track 0
    p.tracks.push(Default::default());
    let mut other = video(1.0);
    other.at = 5.0;
    let other = p.add(1, other);
    p.ripple_remove(a);
    assert!(p.get(a).is_none());
    assert!(
        (p.get(b).unwrap().at).abs() < 1e-9,
        "following item moved left by the removed length"
    );
    assert!((p.get(other).unwrap().at - 5.0).abs() < 1e-9, "other tracks untouched");
}

#[test]
fn tail_from_drops_trims_and_rebases() {
    let (p, _, b) = project();
    let rest = p.tail_from(5.0);
    assert_eq!(rest.items().count(), 1);
    let item = rest.get(b).unwrap();
    assert!((item.at).abs() < 1e-9 && (item.start - 1.0).abs() < 1e-9 && (item.cut - 1.0).abs() < 1e-9);
    let later = p.tail_from(1.0);
    assert!((later.get(b).unwrap().at - 3.0).abs() < 1e-9);
}

#[test]
fn geometry_contains_and_covers() {
    let item = video(1.0); // 16:9 source
    let (fw, fh) = item.frac(Aspect::Vertical);
    assert!((fw - 1.0).abs() < 1e-4 && fh < 0.4); // fits width, letterboxed
    let mut cover = item.clone();
    cover.scale = item.cover_scale(Aspect::Vertical);
    let (_, fh) = cover.frac(Aspect::Vertical);
    assert!((fh - 1.0).abs() < 1e-4);
}

#[test]
fn fingerprint_tracks_visual_changes() {
    let (mut p, a, _) = project();
    let before = p.fingerprint();
    assert_eq!(before, p.fingerprint());
    p.get_mut(a).unwrap().x = 0.4;
    assert_ne!(before, p.fingerprint());
}

#[test]
fn zoom_expr_only_when_needed() {
    let mut i = video(2.0);
    assert!(i.zoom_expr().is_none());
    i.zoom = 1.5;
    assert_eq!(i.zoom_expr().as_deref(), Some("1.5000"));
    i.effect = ZoomEffect::In;
    let punch = i.zoom_expr().unwrap();
    assert!(
        punch.starts_with("1.5000*(1+0.300*(1-pow(1-"),
        "punch-in eases out: {punch}"
    );
    i.effect = ZoomEffect::Pulse;
    assert!(
        i.zoom_expr().unwrap().contains("cos(2*PI*2.5*"),
        "the beat pulses at 2.5 Hz"
    );
}

#[test]
fn library_keeps_files_after_their_items_are_removed() {
    let (mut p, a, _) = project(); // two items, both "a.mp4"
    let item = p.get(a).unwrap().clone();
    p.register(&item);
    p.register(&item); // same file again: still one entry
    assert_eq!(p.media.len(), 1);
    assert_eq!(p.usage("a.mp4"), 2);
    p.remove(a);
    assert_eq!(p.usage("a.mp4"), 1);
    assert_eq!(p.media.len(), 1, "the library entry stays");
    assert_eq!(p.first_use("a.mp4").map(|i| i.at), Some(4.0));
    let fresh = p.media[0].to_item();
    assert!((fresh.len() - 4.0).abs() < 1e-9 && fresh.at == 0.0);
}

#[test]
fn text_is_not_library_media_and_sync_fills_old_projects() {
    let mut p = Project::default();
    let text = Item::new(String::new(), "Hi".into(), Kind::Text, (10, 10), 0.0, false);
    p.add(0, text.clone());
    p.add(0, video(2.0));
    p.sync_media();
    assert_eq!(p.media.len(), 1);
    p.register(&text);
    assert_eq!(p.media.len(), 1);
}

#[test]
fn only_empty_tracks_can_be_removed_and_never_the_last() {
    let (mut p, ..) = project();
    assert!(!p.remove_empty_track(0), "has items");
    p.tracks.push(Default::default());
    assert!(p.remove_empty_track(1));
    assert_eq!(p.tracks.len(), 1);
    let mut empty = Project::default();
    assert!(!empty.remove_empty_track(0), "the last track stays");
}

#[test]
fn the_edges_are_the_seams_of_clips_and_captions_without_duplicates() {
    let (mut p, ..) = project(); // a: 0..4, b: 4..6 meet at 4
    p.captions = vec![
        Caption {
            start: 1.0,
            end: 4.0,
            text: "x".into(),
        },
        Caption {
            start: 4.0,
            end: 5.5,
            text: "y".into(),
        },
    ];
    assert_eq!(
        p.edges(),
        vec![0.0, 1.0, 4.0, 5.5, 6.0],
        "the seam at 4 s appears once, in order"
    );
    assert_eq!(Project::default().edges(), vec![0.0]);
}

#[test]
fn a_nearby_playhead_lands_on_the_seam_and_a_far_one_stays_put() {
    use crate::ui::snap;
    let (p, ..) = project();
    let edges = p.edges();
    assert_eq!(snap(4.03, 0.0, &edges, 100.0), 4.0, "within 8 px at 100 px/s");
    assert_eq!(snap(4.5, 0.0, &edges, 100.0), 4.5, "far from every seam");
    assert_eq!(snap(5.96, 0.0, &edges, 100.0), 6.0, "the end counts too");
}

#[test]
fn trimmed_video_grows_back_to_its_source_but_no_further() {
    let (mut p, a, _) = project(); // a: video at 0, 4 s long
    let src = p.get(a).unwrap().src_len;
    p.get_mut(a).unwrap().start = 1.0;
    p.get_mut(a).unwrap().at = 2.0;
    p.trim_edge(a, true, 1.5); // pull the front out by 0.5 s
    let i = p.get(a).unwrap();
    assert_eq!(
        (i.start, i.at),
        (0.5, 1.5),
        "cut-off part returns, the picture stays in sync"
    );
    p.trim_edge(a, true, -9.0);
    let i = p.get(a).unwrap();
    assert_eq!((i.start, i.at), (0.0, 1.0), "stops at the start of the source");
    p.trim_edge(a, false, 99.0);
    assert_eq!(p.get(a).unwrap().end, src, "stops at the end of the source");
}
