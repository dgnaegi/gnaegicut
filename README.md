# GnaegiCut

> I didn't like CapCut, so I told Claude to build an alternative.

That is the whole origin story. No roadmap, no pitch deck, no "we saw a gap in the market". CapCut annoyed me, I opened a
terminal, and one very long afternoon of prompts later there is a native Mac video editor written in Rust, with a Swiss-design UI
nobody asked for.

![logo](assets/logo.svg)

It was written almost entirely by [Claude Code](https://claude.com/claude-code), steered by a human whose main
contributions were "faster", "more TikTok", "add tabs" and "no helper texts". It has about 90 tests, which is more than
most things I have shipped by hand, and for a good reason: the model needs something to be wrong *against*.

## What it does

Cut vertical videos for TikTok, Reels and Shorts, locally, without a watermark, an account or a subscription.

| | |
|---|---|
| Media | Drag in videos, images and sounds. Several at once stack as layers (or line up, your call) |
| Tracks | As many as you like. Drag things around, drop them on a lane, snap them with the magnet |
| Placement | Move, scale and rotate on the preview, with magic lines when something is centred |
| Text | Any font on your Mac |
| Effects | Punch-in zooms, beat zoom, shake, looks (Vivid, VHS, ...), safe-zone overlay |
| Transitions | Click the **+** between two clips: whip, flash, glitch, spin, zoom, circle, wipes, pushes, dissolve, dip |
| Captions | Local Whisper transcription, burned in. Default look: AL Unica77, `#FF1975`, heavy black shadow |
| Audio | One-click voice clean-up, volume, fades |
| Free sounds | Search effects and music (Freesound, Jamendo via Openverse), audition, add. Credits included |
| Formats | 9:16, 4:5, 16:9 |
| Projects | Save, open, autosave |

Preview and export go through the same ffmpeg filter graph, so what you see is what you get, which CapCut sometimes
treats as a suggestion.

## Run it

macOS only, because the author uses a Mac and the model took that at face value.

```sh
sh scripts/setup.sh                                  # Rust, ffmpeg-full, whisper.cpp, caption model (~140 MB)
export PATH="$(brew --prefix rustup)/bin:$PATH"
cargo run --release                                  # open the editor
cargo run --release -- clip1.mp4 clip2.mov           # ...with files already loaded
cargo run --release -- project.gcut                  # ...or a saved project
```

Then drop some videos on the window. That is the whole onboarding.

### Keys

| Key | Does |
|---|---|
| `Space` | play / pause |
| `S` | split at the playhead |
| `Delete` | remove the selected item |
| `Cmd+S`, `Shift+Cmd+S`, `Cmd+O` | save, save as, open |
| `Alt` while dragging | move without snapping |

## FAQ

**Is it better than CapCut?** No. It is mine.

**Does it have a watermark?** No. There is a logo in the corner of the app, which is where logos belong.

**Does it have undo?** No. Save often. This is the first thing on the list and it is a list of three things.

**Does it upload my videos?** No. Everything runs on your machine. The only time it touches the network is when you search
for free sounds, and then it only sends your search word to [Openverse](https://openverse.org).

**Is it vibe coded?** Yes. Quite carefully, though: the code is split into small files, there are tests that look at actual
pixels and actual audio samples, and the docs say why things are the way they are. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
and [docs/DESIGN.md](docs/DESIGN.md).

**Why Rust?** The model suggested it and I did not argue. Compile errors turned out to be a decent way to find out what
the model was thinking.

**Why does it look like a 1950s Swiss poster?** Because I pasted a design brief and Claude took it very seriously.

## Hacking on it

```sh
cargo fmt                    # style (see rustfmt.toml)
cargo clippy --all-targets   # keep it warning-free
cargo test                   # unit and ffmpeg tests
cargo test -- --ignored      # also: whisper, real-time playback (audible), live sound search
```

House rules, kept so the next prompt does not undo the last one: files stay under 200 lines, logic lives outside the UI,
colours come from `theme.rs` and nowhere else, every effect gets a test that looks at the result.

## Things that are not there yet

- Undo and redo, keyframes, clip speed.
- A `.app` you can double-click. It runs from a checkout and finds its fonts and model next to the source.
- Windows and Linux.
- The caption font AL Unica77 is commercial and **not included**. If it is installed it is used, otherwise the system picks
  something else, and you can pick anything in the Captions tab.

## Thanks to

[ffmpeg](https://ffmpeg.org) does all the actual video work, [whisper.cpp](https://github.com/ggml-org/whisper.cpp) does the
listening, [egui](https://github.com/emilk/egui) draws the buttons, [Openverse](https://openverse.org) finds the sounds, and
[Inter](https://rsms.me/inter/) (SIL Open Font License, see `assets/fonts/LICENSE.txt`) is the bundled font. Claude wrote the rest.

## License

[WTFPL](LICENSE). Do what the fuck you want to.
