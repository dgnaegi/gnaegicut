# Design: Swiss International Style

Objective, grid-first, typographic. The interface recedes so the video is the only image.

## Tokens (`src/theme.rs`)

| Token | Value | Use |
|---|---|---|
| `WHITE` / `BLACK` | `#FFFFFF` / `#000000` | canvas, text, structure |
| `MUTED` | `#F2F2F2` | secondary surfaces, slider rails |
| `ACCENT` | `#FF3000` | the only signal colour: primary action, selection, playhead, section numbers |
| `BORDER` | 2 px | every container and control |
| Radius | 0 | strictly rectangular |

Type is Inter: Black for the wordmark and the empty-state headline, Bold uppercase for controls and labels, Regular
for values. No shadows, no gradients.

## Components (`src/widgets.rs`)

- `button` has three kinds. `Plain` inverts to black on hover; `Active` (selected) turns red on hover;
  `Cta` is red and turns black. Change is instant, never a fade.
- `segmented` joins buttons into one bordered strip (aspect ratio, fill/fit, caption style).
- `section("01", "Clip")` is the numbered label with a rule beneath.
- `patterns` provides the 24 px grid (stage, timeline) and 16 px dot matrix (inspector) at a few percent opacity.

## Rules for new UI

1. Take colours and sizes from `theme`; never write a literal colour in a panel.
2. Build from `widgets`; if a pattern repeats twice, it becomes a widget.
3. Red marks what matters now. Never decorative.
4. Left-align everything. Uppercase labels, sentence-case values.
5. Layout is asymmetric: large stage, narrow inspector, full-width timeline.

## Logo (`assets/logo.svg`, `src/logo.rs`)

A geometric **G** (three-quarter ring plus crossbar) on a black square, cut by a red 45-degree blade. The blade
carries a black gap so it reads as a real cut through the letter. Built on a 48-unit grid from circles and lines only.
`logo.rs` redraws it with egui primitives so it stays crisp at any size; `assets/icon.png` is the rendered window icon.
