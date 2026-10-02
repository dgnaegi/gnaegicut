# GnaegiCut

I didn't like CapCut, so I told Claude to build an alternative.

A video editor for TikTok, Reels and Shorts. Native macOS app, written in Rust. Everything runs locally.

![logo](assets/logo.svg)

## Features

- Videos, images and sounds on any number of tracks, all by drag and drop
- 9:16, 4:5 and 16:9
- Move, scale and rotate on the preview, with alignment lines and safe zones
- Text in any installed font
- Zoom, shake, looks and transitions between clips (whip, flash, glitch, spin, wipe and more)
- Captions from local Whisper, burned in
- Voice clean-up, volume, fades
- Free sound and music search (Openverse)
- Save, open, autosave
- Preview and export use the same ffmpeg graph, so they match

## Run

macOS only.

```sh
sh scripts/setup.sh                                  # Rust, ffmpeg-full, whisper.cpp, caption model
export PATH="$(brew --prefix rustup)/bin:$PATH"
cargo run --release
cargo run --release -- clip.mp4 other.mov            # with files
cargo run --release -- project.gcut                  # with a project
```

Keys: `Space` play/pause, `S` split, `Delete` remove, `Cmd+Z` undo, `Shift+Cmd+Z` redo, `Cmd+C` `Cmd+X` `Cmd+V` copy, cut, paste, `F` full screen preview, `Cmd+S` save, `Cmd+O` open, `Cmd+plus` `Cmd+minus` zoom the tracks, `Cmd+0` fit, `Alt` while dragging to skip snapping. Pinch or Cmd+scroll zooms the tracks; the mouse wheel over the preview resizes the selected element; drag the bar on top of the timeline to make it taller.

## Develop

```sh
cargo fmt && cargo clippy --all-targets && cargo test
```

Notes on how it fits together are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) and [docs/DESIGN.md](docs/DESIGN.md).

## Missing

Keyframes, clip speed, a double-clickable `.app`, Windows and Linux.

The default caption font, AL Unica77, is commercial and not included. It is used if it is installed.

## License

[WTFPL](LICENSE)
