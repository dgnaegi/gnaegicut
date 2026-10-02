mod actions;
mod app;
mod app_types;
mod drop;
mod fonts;
mod logo;
mod media;
mod patterns;
mod persist;
mod player;
mod preview;
mod project;
mod session;
mod settle;
mod sound_actions;
mod sound_preview;
mod sound_state;
mod sounds;
mod text;
mod theme;
mod thumbs;
mod ui;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    // GUI apps launched from Finder don't inherit the shell PATH, so make sure ffmpeg is findable.
    let path = std::env::var("PATH").unwrap_or_default();
    let tools = "/opt/homebrew/opt/ffmpeg-full/bin:/opt/homebrew/bin:/usr/local/bin";
    // SAFETY: called once at startup, before any other thread exists.
    unsafe { std::env::set_var("PATH", format!("{tools}:{path}")) };

    let viewport = egui::ViewportBuilder::default()
        .with_title("GnaegiCut")
        .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png")).expect("valid icon"))
        .with_inner_size([1280.0, 820.0])
        .with_min_inner_size([900.0, 600.0]);
    eframe::run_native(
        "GnaegiCut",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(|cc| {
            theme::apply(&cc.egui_ctx);
            let mut app = app::App::new(cc.egui_ctx.clone());
            let files: Vec<std::path::PathBuf> = std::env::args_os().skip(1).map(Into::into).collect();
            if let Some(project) = files
                .iter()
                .find(|f| f.extension().is_some_and(|e| e == persist::EXTENSION))
            {
                app.open_path(project.clone());
            } else if files.is_empty() {
                app.restore_autosave();
            } else {
                app.stack_drops = false; // files named on the command line form a montage, one after another
                app.add_files(&files, None);
                app.stack_drops = true;
            }
            Ok(Box::new(app))
        }),
    )
}
