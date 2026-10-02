//! Undo and redo. The history holds whole copies of the project: it is small, copies are cheap, and it means every
//! kind of edit (moving, trimming, captions, transitions, the library) is undoable without writing an inverse for each.
//!
//! A step is recorded once an edit has *settled*: the mouse is up and nothing changed for a moment. So dragging an
//! item across the timeline, or sliding a slider, is one step however many frames it took.

use crate::project::Project;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::{Duration, Instant};

const MAX_STEPS: usize = 100;
const SETTLE: Duration = Duration::from_millis(150);

pub struct History {
    undo: Vec<Project>,
    redo: Vec<Project>,
    /// The project as of the last recorded step: what an undo goes back to.
    committed: Project,
    committed_key: u64,
    last_key: u64,
    changed_at: Instant,
}

/// A fingerprint of everything the project saves. Selection, playhead and zoom are not in it.
fn key(p: &Project) -> u64 {
    let mut h = DefaultHasher::new();
    serde_json::to_string(p).unwrap_or_default().hash(&mut h);
    h.finish()
}

impl History {
    pub fn new(project: &Project) -> Self {
        let k = key(project);
        Self {
            undo: vec![],
            redo: vec![],
            committed: project.clone(),
            committed_key: k,
            last_key: k,
            changed_at: Instant::now(),
        }
    }

    /// Forgets the history, e.g. after opening another project.
    pub fn reset(&mut self, project: &Project) {
        *self = Self::new(project);
    }

    /// Call every frame with the live project. Records a step when it differs from the last one and has stopped
    /// changing. Returns true while an unrecorded change is waiting, so the caller can schedule another look.
    pub fn observe(&mut self, project: &Project, now: Instant, pointer_down: bool) -> bool {
        let k = key(project);
        if k != self.last_key {
            (self.last_key, self.changed_at) = (k, now);
        }
        if k == self.committed_key {
            return false;
        }
        if !pointer_down && now.saturating_duration_since(self.changed_at) >= SETTLE {
            self.record(project, k);
            return false;
        }
        true
    }

    fn record(&mut self, project: &Project, k: u64) {
        self.undo.push(std::mem::replace(&mut self.committed, project.clone()));
        if self.undo.len() > MAX_STEPS {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.committed_key = k;
    }

    /// Records a change that has not settled yet, so undo never skips over it.
    fn flush(&mut self, project: &Project) {
        let k = key(project);
        if k != self.committed_key {
            self.record(project, k);
        }
    }

    /// Replaces `project` with the previous step. Returns false if there is nothing to undo.
    pub fn undo(&mut self, project: &mut Project) -> bool {
        self.flush(project);
        let Some(previous) = self.undo.pop() else { return false };
        self.redo.push(std::mem::replace(project, previous));
        self.sync(project);
        true
    }

    pub fn redo(&mut self, project: &mut Project) -> bool {
        self.flush(project);
        let Some(next) = self.redo.pop() else { return false };
        self.undo.push(std::mem::replace(project, next));
        self.sync(project);
        true
    }

    fn sync(&mut self, project: &Project) {
        self.committed = project.clone();
        self.committed_key = key(project);
        self.last_key = self.committed_key;
    }
}

#[cfg(test)]
mod tests;
