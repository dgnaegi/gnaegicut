use super::*;

fn clip(len: f64) -> Item {
    Item::new("a.mp4".into(), "a".into(), Kind::Video, (1920, 1080), len, true)
}

fn picture() -> Item {
    Item::new("p.png".into(), "p".into(), Kind::Image, (100, 100), 0.0, false)
}

fn track_of(p: &Project, id: u64) -> usize {
    p.find(id).unwrap().0
}

#[test]
fn stacking_puts_every_file_on_its_own_track_at_the_same_time() {
    let mut p = Project::default();
    let ids = p.place_batch(vec![clip(4.0), clip(3.0), clip(2.0)], 0, 1.5, true);
    assert_eq!(ids.iter().map(|&i| track_of(&p, i)).collect::<Vec<_>>(), [0, 1, 2]);
    assert!(ids.iter().all(|&i| (p.get(i).unwrap().at - 1.5).abs() < 1e-9));
    assert_eq!(p.tracks.len(), 3, "missing tracks are created");
}

#[test]
fn stacking_skips_tracks_that_are_busy_at_that_time() {
    let mut p = Project::default();
    p.add(0, clip(5.0)); // track 0 busy 0..5
    let ids = p.place_batch(vec![clip(2.0), clip(2.0)], 0, 1.0, true);
    assert_eq!(
        ids.iter().map(|&i| track_of(&p, i)).collect::<Vec<_>>(),
        [1, 2],
        "above the busy track"
    );
    let later = p.place_batch(vec![clip(1.0)], 0, 5.0, true); // track 0 is free again from 5 s
    assert_eq!(track_of(&p, later[0]), 0);
}

#[test]
fn sequence_places_files_back_to_back_on_one_track() {
    let mut p = Project::default();
    let ids = p.place_batch(vec![clip(2.0), clip(3.0), clip(1.0)], 0, 4.0, false);
    let at: Vec<f64> = ids.iter().map(|&i| p.get(i).unwrap().at).collect();
    assert_eq!(at, [4.0, 6.0, 9.0]);
    assert!(ids.iter().all(|&i| track_of(&p, i) == 0));
}

#[test]
fn overlay_pictures_start_small_but_the_base_track_stays_full() {
    let mut p = Project::default();
    let ids = p.place_batch(vec![picture(), picture()], 0, 0.0, true);
    assert_eq!(p.get(ids[0]).unwrap().scale, 1.0);
    assert_eq!(p.get(ids[1]).unwrap().scale, 0.5);
}

#[test]
fn negative_times_clamp_and_an_empty_batch_is_harmless() {
    let mut p = Project::default();
    assert!(p.place_batch(vec![], 0, 3.0, true).is_empty());
    let id = p.place_batch(vec![clip(1.0)], 0, -2.0, false)[0];
    assert_eq!(p.get(id).unwrap().at, 0.0);
}

#[test]
fn a_pasted_copy_keeps_its_look_but_not_its_transition_and_avoids_busy_tracks() {
    let mut p = Project::default();
    let first = p.place_batch(vec![clip(4.0)], 0, 0.0, false)[0];
    let mut original = clip(2.0);
    (original.scale, original.rotation, original.volume) = (0.7, 12.0, 0.5);
    original.join = Some(Join {
        effect: JoinEffect::Flash,
        len: 0.3,
    });
    let copy = p.paste_item(original, 0, 1.0); // track 0 is busy 0..4
    let pasted = p.get(copy).unwrap();
    assert_eq!(p.find(copy).map(|(t, _)| t), Some(1), "moved up to a free track");
    assert!((pasted.at - 1.0).abs() < 1e-9);
    assert!((pasted.scale - 0.7).abs() < 1e-6 && pasted.rotation == 12.0 && pasted.volume == 0.5);
    assert!(pasted.join.is_none(), "the transition belonged to the old neighbour");
    assert_ne!(copy, first, "a new item, not the same one");
    let again = p.paste_item(p.get(copy).unwrap().clone(), 0, 5.0);
    assert_eq!(p.find(again).map(|(t, _)| t), Some(0), "track 0 is free again from 4 s");
}
