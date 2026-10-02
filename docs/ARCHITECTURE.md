# Architecture

One rule drives the layout: **data and logic never import UI; UI never builds ffmpeg commands.**

```
src/
  main.rs            entry point, window, PATH for GUI launches
  app.rs             App state, selection, inspector tab, per-frame tick, player polling
  actions.rs         user operations (import, text, split, export, transcribe); spawn background jobs
  session.rs         save / open / autosave / restore, and quiet-time housekeeping (resume, autosave)
  persist.rs         project <-> JSON (.gcut), versioned
  settle.rs          "has this value stopped changing?" for debouncing
  project/           the edit as plain data + pure operations (unit-tested)
    mod.rs             Project, Track, lookups, fingerprint
    item.rs            Item: media on a track (trim, position, scale, rotation, zoom, audio, text look)
    item_geometry.rs   box size, rotation and position (shared by overlay and graph)
    item_motion.rs     zoom, rotation, slide/shake/glitch jitter, fades and reveals as ffmpeg expressions (eased)
    item_fx.rs         looks, glitch, flash and whip blur filters
    place.rs           placing several items at once: stacked as layers or in sequence
    kinds.rs           Kind (video/image/text), ZoomEffect, Transition
    aspect.rs          Aspect (9:16, 4:5, 16:9) and platform safe-zone margins
    caption.rs         Caption, CaptionStyle, CaptionLayout (font, position)
    media.rs           MediaRef: the project's library of source files (kept when items are deleted)
    join.rs            transitions between two clips: set_join, resolved()
    join_effects.rs    the catalogue of transitions and what each does to the clips on either side
    edit.rs            split, place, ripple_remove, tail_from, topmost_at
  media/             everything that shells out
    graph.rs           Project -> ffmpeg filter graph (the single source of truth)
    reveal.rs          grey masks for wipes and the circle (applied with alphamerge)
    probe.rs           ffprobe -> Item (handles phone rotation)
    frame.rs           one composite frame (paused preview)
    stream.rs          raw frames + raw audio piped from ffmpeg (playback)
    export.rs          final MP4
    ass.rs             captions -> ASS subtitle file
    whisper.rs         audio -> whisper-cli -> SRT -> captions
  fonts.rs           installed + bundled fonts (fontdb)
  thumbs.rs          library thumbnails, decoded on worker threads
  drop.rs            where dropped or dragged media lands (lane and time under the pointer)
  sounds/            free sound search (Openverse), download cache, attribution
  sound_actions.rs   search / preview / add sounds; sound_preview.rs plays auditions; sound_state.rs panel state
  text.rs            text -> transparent PNG (fontdue), then treated like an image
  player.rs          real-time playback clock
  preview.rs         still-frame worker + texture
  theme.rs / widgets.rs / patterns.rs / logo.rs   design system
  ui/                one file per panel: toolbar, timeline (+items), stage (+edit, geometry, guides, overlay),
                     inspector (tabs) with item_panel, text_panel, captions, font_picker
```

## Data flow

- **Edit model.** `Project` holds clips (source file + in/out seconds), captions (timeline seconds), aspect and
  fit mode. The timeline is the clips laid end to end.
- **Background work.** `App::spawn` runs a job on a thread and wakes the UI. Jobs report through an `Event`
  channel (`Status`, `Captions`) that `App::tick` drains. The UI thread never blocks on ffmpeg.
- **Jobs get a snapshot** (`Project::snapshot`), so you can keep editing while an export runs.

## One graph for everything

`media/graph.rs` is the only place that builds ffmpeg filter graphs. Layers composite bottom to top over a black
base: each item is opened with `-ss/-t` (fast seek), scaled to its box, optionally zoomed inside it
(`scale ... eval=frame` then `crop`, driven by an expression of time), shifted to its timeline position with `setpts`
and overlaid. Audio is processed per item (optional voice clean-up, gain, fades), delayed into place and mixed.
Captions are added as an `ass` filter on top.

Preview frames, playback, export and transcription all call `graph::build`, so they cannot disagree. A fingerprint of
the project (`Project::fingerprint`) keys the preview cache and detects edits made during playback.

Text items are rendered to a PNG by `text.rs` with the chosen font and then flow through the image path unchanged.

## Preview editing

`ui/stage_geometry.rs` models an item's box on the preview (centre, half-size, rotation) for corners, hit tests and the
rotation handle. `ui/stage_guides.rs` snaps a dragged box's centre or edges to targets (frame centre and edges, safe
zone, other items) and returns the lines to draw. Both are pure and unit-tested; the unsnapped drag position is kept
separately so snapping never swallows slow drags.

## Sessions

`Project` serialises with serde. `session.rs` saves and opens `.gcut` files and autosaves two seconds after the last
edit to `~/Library/Application Support/GnaegiCut/autosave.gcut`, which is restored on a launch without arguments.
Text items are re-rendered after loading because their PNGs live in the temp folder. If an edit happens during
playback, the player stops and `resume` is set; once the pointer is up and the edit has been quiet for 250 ms,
playback continues from the playhead.

## Playback

1. `Player::start` takes `project.tail_from(playhead)`, which drops, trims and re-bases items (and captions), so
   ffmpeg never needs to seek inside a composite. Zoom effects stay continuous through `Item::cut`.
2. `stream::video` pipes RGBA frames at 30 fps through a bounded channel. The bound is the backpressure that keeps
   ffmpeg only slightly ahead of the clock.
3. `stream::audio` renders the mixed audio to f32 PCM; `rodio` plays it.
4. When audio and the first frame are both ready the clock starts. Each tick shows the newest frame whose index is
   at or before `elapsed * 30`, dropping older ones. Audio is the master clock, so late frames are skipped
   rather than slowing sound. The texture is updated in place; re-allocating it per frame caused stutter.
5. Dropping the `Player` drops the frame receiver; the stream thread notices, kills ffmpeg and exits.

## Transitions between clips

A join belongs to the later clip and is measured against the clip before it (`Project::predecessor`). Overlapping
effects (everything except Dip) move the later clip, and everything after it on the track, earlier by the transition
length, so the two clips play together; `set_join` keeps that geometry right when the effect or length changes or the
join is removed. `Project::resolved()` then derives each clip's `link_out` / `link_in` (an effect and a length) without
touching the saved data; the graph reads them through `Item::enter()` and `Item::exit()`, exactly as it reads a clip's
own intro and outro. Push is a slide on both clips, Dissolve a fade on the new clip, Zoom a fade plus a settling zoom,
and Wipe and Circle a grey mask (`media/reveal.rs`) that is animated for the first part of the new clip and white after.
Joins are resolved on the whole edit *before* `tail_from` cuts the front, so playing from inside a transition continues
it instead of restarting. A transition whose clips were dragged apart is ignored; one whose overlap shrank shortens.

## Dropping media

`drop.rs` turns a pointer position into a lane and time using the timeline's last layout (`TimelineView`, saved each frame).
Over the timeline, media lands on that lane at that time (snapping to edges when the magnet is on); anywhere else it
goes to the active track. `Project::place_batch` does the placing: stacked (each file on the next free track upward, all at
the same time) or in sequence. Library cards are dragged with egui's drag-and-drop payloads.

## Sounds

Audio is a first-class kind (`Kind::Audio`): it has no picture, so the graph skips its video layer, but it mixes, fades and
delays like any clip. `sounds/` searches Openverse (`source=freesound` for effects, `source=jamendo` for music,
`license_type=commercial`, at most 20 results per page: more needs an account). Downloads are cached in
`~/Library/Caches/GnaegiCut/sounds`. Each result carries its attribution text; it is stored on the item and the library entry
and written to `<export>.credits.txt`. Transcription uses `Streams::Speech`, which leaves music and effects out.

## Effects and easing

Motion expressions use ease-out (`1-(1-p)^3`) for entrances and ease-in for exits, so moves start fast and land softly.
Looks, glitch, flash and whip blur are plain ffmpeg filters, switched on with `enable=` windows in item time.
A glitch also jolts the overlay position; a shake adds a sum of sines to it; a spin animates the `rotate` angle.

## Library

`Project::media` lists every imported file. Items reference it by path, so deleting an item keeps the file in the
library and it can be placed again. Thumbnails are decoded once on worker threads and cached as textures.

## Transitions and rotation

A Fade is an alpha fade (`fade ... alpha=1`) run in *item time*: the clock starts at `Item::cut`, so playing from the
middle of an item looks identical to playing it from the start. A slide animates the overlay's `x`/`y` with an
expression of `t`. Rotation uses `rotate` on a transparent canvas the size of the rotated bounding box, so corners
stay transparent and the item stays centred.

## Captions

`whisper::transcribe` renders the mixed audio to 16 kHz mono (including any voice enhancement, which helps accuracy),
runs `whisper-cli` and parses the SRT. `ass::render` positions each line with `\pos` and the chosen font family.
It is part of the graph, so the preview shows captions exactly as exported.

## Testing

Pure logic (`project`, `ass`, SRT parsing, snapping) has fast unit tests. ffmpeg paths are tested against generated
clips with unique file names per test: pixel colours prove an overlay appears at the right time and place, PCM
analysis proves audio delay, enhancement, volume and fades. Tests needing the model or an audio device are `#[ignore]`.

## Extending

- **New per-item effect:** add a field on `Item`, include it in `hash_into`, add its filter in `graph::video_layer`
  or `audio_filters`, and one control in `ui/item_panel.rs`. Preview and export follow.
- **New caption style:** add a variant and `Look` in `project/caption.rs`.
