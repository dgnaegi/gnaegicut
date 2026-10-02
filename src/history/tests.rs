use super::*;
use crate::project::{Item, Kind};

fn clip(name: &str) -> Item {
    Item::new(format!("{name}.mp4"), name.into(), Kind::Video, (160, 90), 4.0, true)
}

/// Observes `project` "a while later", with the mouse up, so any change is recorded.
fn settle(history: &mut History, project: &Project, at: Instant) {
    history.observe(project, at, false);
    history.observe(project, at + Duration::from_millis(300), false);
}

#[test]
fn an_edit_becomes_one_step_that_undo_and_redo_walk_through() {
    let mut project = Project::default();
    let mut history = History::new(&project);
    let t0 = Instant::now();
    project.add(0, clip("a"));
    settle(&mut history, &project, t0);
    assert_eq!(project.items().count(), 1);

    assert!(history.undo(&mut project));
    assert_eq!(project.items().count(), 0, "undone");
    assert!(history.redo(&mut project));
    assert_eq!(project.items().count(), 1, "redone");
    assert!(!history.redo(&mut project), "nothing further to redo");
}

#[test]
fn many_changes_while_the_mouse_is_down_are_a_single_step() {
    let mut project = Project::default();
    let id = project.add(0, clip("a"));
    let mut history = History::new(&project);
    let t0 = Instant::now();
    for i in 0..30 {
        project.get_mut(id).unwrap().x = 0.5 + (i + 1) as f32 * 0.01; // a drag, frame by frame
        assert!(
            history.observe(&project, t0 + Duration::from_millis(16 * i), true),
            "still waiting"
        );
    }
    settle(&mut history, &project, t0 + Duration::from_secs(1));
    assert!(history.undo(&mut project));
    assert_eq!(project.get(id).unwrap().x, 0.5, "one undo takes the whole drag back");
    assert!(!history.undo(&mut project), "and that was the only step");
}

#[test]
fn undo_right_after_an_edit_does_not_wait_for_it_to_settle() {
    let mut project = Project::default();
    let mut history = History::new(&project);
    project.add(0, clip("a"));
    history.observe(&project, Instant::now(), true); // still being edited
    assert!(history.undo(&mut project));
    assert!(project.is_empty(), "the unsettled edit was recorded and undone");
}

#[test]
fn a_new_edit_after_an_undo_clears_the_redo_steps() {
    let mut project = Project::default();
    let mut history = History::new(&project);
    let t0 = Instant::now();
    project.add(0, clip("a"));
    settle(&mut history, &project, t0);
    history.undo(&mut project);
    project.add(0, clip("b"));
    settle(&mut history, &project, t0 + Duration::from_secs(2));
    assert!(!history.redo(&mut project), "the undone branch is gone");
    assert_eq!(project.items().next().map(|i| i.name.as_str()), Some("b"));
}

#[test]
fn every_kind_of_change_is_undoable_not_just_clip_edits() {
    let mut project = Project::default();
    let mut history = History::new(&project);
    let t0 = Instant::now();
    project.tracks.push(Default::default()); // an empty new track
    settle(&mut history, &project, t0);
    project.captions.push(crate::project::Caption {
        start: 0.0,
        end: 1.0,
        text: "hi".into(),
    });
    settle(&mut history, &project, t0 + Duration::from_secs(1));
    assert!(history.undo(&mut project));
    assert!(
        project.captions.is_empty() && project.tracks.len() == 2,
        "captions undone first"
    );
    assert!(history.undo(&mut project));
    assert_eq!(project.tracks.len(), 1, "then the new track");
}

#[test]
fn history_is_capped_and_can_be_reset() {
    let mut project = Project::default();
    let mut history = History::new(&project);
    let mut at = Instant::now();
    for i in 0..(MAX_STEPS + 20) {
        project.add(0, clip(&format!("c{i}")));
        at += Duration::from_secs(1);
        settle(&mut history, &project, at);
    }
    let mut steps = 0;
    while history.undo(&mut project) {
        steps += 1;
    }
    assert_eq!(steps, MAX_STEPS, "the oldest steps are dropped");
    history.reset(&project);
    assert!(!history.undo(&mut project), "a reset history has nothing to undo");
}

#[test]
fn nothing_changing_records_nothing() {
    let project = Project::default();
    let mut history = History::new(&project);
    assert!(!history.observe(&project, Instant::now() + Duration::from_secs(5), false));
    let mut again = project.clone();
    assert!(!history.undo(&mut again));
}
