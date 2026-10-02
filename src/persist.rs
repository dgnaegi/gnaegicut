//! Saving and opening projects as JSON (`.gcut`), plus the autosave file used to restore the last session.

use crate::project::{Kind, Project};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const VERSION: u32 = 1;
pub const EXTENSION: &str = "gcut";

#[derive(Serialize)]
struct Out<'a> {
    version: u32,
    project: &'a Project,
}

#[derive(Deserialize)]
struct In {
    version: u32,
    project: Project,
}

pub fn save(project: &Project, path: &Path) -> Result<(), String> {
    let json = serde_json::to_string_pretty(&Out {
        version: VERSION,
        project,
    })
    .map_err(|e| e.to_string())?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, json).map_err(|e| format!("could not write {}: {e}", path.display()))
}

pub fn load(path: &Path) -> Result<Project, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let file: In = serde_json::from_str(&text).map_err(|e| format!("not a GnaegiCut project: {e}"))?;
    if file.version > VERSION {
        return Err(format!("project was saved by a newer version (v{})", file.version));
    }
    Ok(file.project)
}

pub fn autosave_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    home.join("Library/Application Support/GnaegiCut/autosave.gcut")
}

/// Names of media files that no longer exist. Text items are re-rendered on load, so they are skipped.
pub fn missing_media(project: &Project) -> Vec<String> {
    project
        .items()
        .filter(|i| i.kind != Kind::Text && !Path::new(&i.path).exists())
        .map(|i| i.name.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::{Caption, Item, Transition, ZoomEffect};

    fn sample() -> Project {
        let mut p = Project::default();
        let mut v = Item::new(
            "/nope/a.mp4".into(),
            "a.mp4".into(),
            Kind::Video,
            (1920, 1080),
            4.0,
            true,
        );
        (v.x, v.rotation, v.zoom, v.effect, v.transition, v.enhance) =
            (0.3, 12.5, 1.4, ZoomEffect::Pulse, Transition::SlideUp, true);
        p.add(0, v);
        p.add(
            1,
            Item::new(String::new(), "Hi".into(), Kind::Text, (300, 100), 0.0, false),
        );
        p.captions = vec![Caption {
            start: 0.0,
            end: 1.0,
            text: "hello".into(),
        }];
        p.caption_layout.family = "Helvetica".into();
        p
    }

    #[test]
    fn round_trips_everything_that_affects_the_output() {
        let path = std::env::temp_dir().join(format!("gc_persist_{}.gcut", std::process::id()));
        let original = sample();
        save(&original, &path).unwrap();
        let back = load(&path).unwrap();
        assert_eq!(back.fingerprint(), original.fingerprint());
        assert_eq!(back.items().count(), 2);
        assert_eq!(back.tracks.len(), 2);
        assert_eq!(back.captions[0].text, "hello");
        assert_eq!(back.caption_layout.family, "Helvetica");
    }

    #[test]
    fn ids_stay_unique_after_loading() {
        let path = std::env::temp_dir().join(format!("gc_persist_ids_{}.gcut", std::process::id()));
        save(&sample(), &path).unwrap();
        let mut back = load(&path).unwrap();
        let new = back.add(0, Item::new("x".into(), "x".into(), Kind::Image, (10, 10), 0.0, false));
        assert!(back.items().filter(|i| i.id == new).count() == 1);
    }

    #[test]
    fn rejects_garbage_and_newer_versions() {
        let dir = std::env::temp_dir();
        let bad = dir.join(format!("gc_bad_{}.gcut", std::process::id()));
        std::fs::write(&bad, "{ nope").unwrap();
        assert!(load(&bad).err().unwrap().contains("not a GnaegiCut project"));
        let json = serde_json::to_string(&Out {
            version: VERSION,
            project: &sample(),
        })
        .unwrap();
        std::fs::write(&bad, json.replace("\"version\":1", "\"version\":99")).unwrap();
        assert!(load(&bad).err().unwrap().contains("newer"));
    }

    #[test]
    fn reports_missing_media_but_not_text() {
        assert_eq!(missing_media(&sample()), vec!["a.mp4".to_string()]);
    }

    /// Builds a sample project for manual checks: `GC_DEMO_OUT=x.gcut GC_DEMO_A=a.mp4 GC_DEMO_B=b.mp4 cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn writes_a_demo_project_with_a_transition() {
        let (Ok(out), Ok(a), Ok(b)) = (
            std::env::var("GC_DEMO_OUT"),
            std::env::var("GC_DEMO_A"),
            std::env::var("GC_DEMO_B"),
        ) else {
            return;
        };
        let mut p = Project::default();
        for (path, at) in [(a, 0.0), (b, 8.0)] {
            let mut item = crate::media::probe::probe(Path::new(&path)).unwrap();
            item.at = at;
            p.register(&item);
            p.add(0, item);
        }
        let b = p.items().nth(1).map(|i| i.id).unwrap();
        p.set_join(
            b,
            Some(crate::project::Join {
                effect: crate::project::JoinEffect::WipeRight,
                len: 1.5,
            }),
        );
        save(&p, Path::new(&out)).unwrap();
    }
}
