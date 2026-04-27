# wgpu-html — Roadmap

## Scope

A GPU renderer for **a static tree of HTML elements**, drawn through `wgpu`.

Explicit non-goals:

- Not a browser. No HTML parsing, no CSS parsing, no DOM.
- No JavaScript, no scripting hooks, no networking, no plugins.
- No interactivity / events / hit-testing yet (revisit later).
- No accessibility tree, no print layout, no SVG rendering.

The user constructs a `Tree` programmatically from typed model structs and
hands it to the renderer.

## Pipeline

```
Tree (typed elements + children)
   │
   ▼  resolve styles (inline `style` attrs → computed CSS Style, with inheritance)
StyledTree
   │
   ▼  layout (block / inline / flex; pure function, no scripting/reflow loop)
LayoutTree
   │
   ▼  paint (display list: quads, glyphs, images, clips, borders)
DisplayList
   │
   ▼  renderer (wgpu)
Frame on surface
```

Each arrow is a clean module boundary: input/output are plain data, each
stage is independently testable.

## Workspace

| Crate                | Role                                                                    | Status |
|----------------------|-------------------------------------------------------------------------|--------|
| `wgpu-html-models`   | Element structs (`Div`, `P`, `Body`, …), `css::Style`, enums            | done   |
| `wgpu-html-tree`     | `Tree { root: Option<Node> }`, `Node { element, children }`, `Element`  | done   |
| `wgpu-html-parser`   | HTML tokenizer + tree builder + inline-CSS + stylesheet parser          | done   |
| `wgpu-html-style`    | Selector matching + cascade: `Tree` → `CascadedTree`                    | M4½    |
| `wgpu-html-layout`   | Block-flow layout: `CascadedTree` → `LayoutBox`                         | M4     |
| `wgpu-html-renderer` | wgpu device/surface + `DisplayList` consumption + pipelines             | M1+M2  |
| `wgpu-html`          | Facade + `paint::paint_tree` (parse → cascade → layout → paint)         | done   |
| `wgpu-html-demo`     | winit binary; builds a sample scene and runs the loop                   | M4     |

Future crate (split out only when it grows large enough to justify it):

- `wgpu-html-paint` — produces `DisplayList` from layout, owns the glyph atlas

For now the paint code lives inside the `wgpu-html` facade.

## Milestones

Each milestone ends in a runnable `cargo run -p wgpu-html-demo`.

### M1 — wgpu skeleton ✅

- Workspace + crates wired up, `wgpu-html` facade
- `Renderer::new` (instance / adapter / device / queue / surface)
- `Renderer::render` clears the surface to a solid color
- `FrameOutcome::{Presented, Reconfigure, Skipped}` instead of leaking
  `wgpu::SurfaceError`
- winit 0.30 `ApplicationHandler` event loop in the demo

### M2 — solid quad pipeline ✅

- `Rect`, `Color`, `Quad`, `DisplayList` in `wgpu-html-renderer::paint`
- `QuadPipeline` (instanced rectangles)
  - WGSL shader, viewport uniform, unit-quad VB+IB
  - Dynamic instance buffer with power-of-two growth
  - Alpha blending
- `Renderer::render(&DisplayList)`
- Demo paints a header bar + three colored columns with translucent
  highlights from a hand-built display list

### M3 — paint a tree of `<div>` ✅

- `wgpu-html::paint::paint_tree(&Tree, vw, vh) -> DisplayList`
- Reads each element's inline `style` attribute via
  `parser::parse_inline_style`, resolves `top`/`left`/`width`/`height`
  and `background-color`, emits one quad per styled box
- CSS length resolution: px / % (vs parent) / vw / vh / vmin / vmax /
  em / rem (16px placeholder) / zero. `auto` and unparsed values fall
  through to defaults (parent size for w/h, 0 for top/left)
- CSS color resolution: hex (3/4/6/8 digits), rgb/rgba, hsl/hsla, named
  (~20 common), transparent. sRGB → linear conversion in software
- "Layout" is absolute positioning: `top`/`left` interpreted relative
  to parent. No flow, no inheritance yet
- Demo parses an HTML string and renders the M2 scene

### M4 — block layout ✅

- New `wgpu-html-layout` crate exposes `LayoutBox { margin_rect,
  border_rect, content_rect, background, kind, children }` and a single
  `layout(&Tree, vw, vh) -> Option<LayoutBox>` entry point
- Block formatting context: vertical stacking inside the parent's
  content box; width auto-fills parent, height fits content
- Per-side margin and padding (with shorthand fall-through)
- `position: absolute` removed; `top`/`left` no longer used
- Borders deferred to M7 (treated as zero)
- `wgpu-html::paint` walks the laid-out box tree and emits one quad per
  background; coordinates are absolute
- Demo: a header bar + three vertically-stacked colored cards with
  padding and inner highlight strips

### M4½ — CSS stylesheets (selectors + cascade) ✅

- `wgpu-html-parser::parse_stylesheet` parses `<style>` block contents
  into a list of `Rule { selectors, declarations }` with `Selector
  { tag, id, classes, universal }` (simple selectors only — no
  combinators yet) and standard CSS specificity
- New `wgpu-html-style` crate:
  - `cascade(&Tree) -> CascadedTree` walks the tree, collects every
    `<style>` block's text, parses it once, and computes a final
    `Style` per element
  - Cascade order: matched rules in ascending specificity → element's
    inline `style="…"` attribute on top
  - `matches_selector` checks tag / id / multi-class / universal
- `wgpu-html-layout::layout` now takes `&CascadedTree`; styles are
  precomputed once per node, never re-parsed during layout
- `paint_tree` chains parse → cascade → layout → paint internally
- 13 cascade unit tests + 9 selector parser tests

### M5 — text rendering

- Pick a text stack (`cosmic-text` or `swash` + custom shaper)
- Glyph atlas (online packing via `etagere` or shelf packer)
- New textured pipeline (alpha-mask sampling for glyphs)
- `PaintCmd::Glyph { atlas_uv, screen_rect, color }`
- Demo: `<h1>` and `<p>` with one font, single-line, then with line wrap

### M6 — inline layout

- Inline formatting context, line boxes, line breaking with
  `cosmic-text`
- Mixed inline runs (`<span>`, `<strong>`, etc.) inheriting style
- Demo: paragraph with mixed bold / link spans

### M7 — backgrounds, borders, radii

- Extend `PaintCmd` and the solid pipeline with rounded corners + border
- SDF-based rounded-rect / border in the fragment shader
- Demo: cards with rounded backgrounds and colored borders

### M8 — images

- `image` crate for decoding
- Image cache + textured pipeline already exists from M5
- `<img>` with width / height
- Demo: a card with an inline image

### M9 — flexbox

- `display: flex`, main / cross axis, basis / grow / shrink
- Wrap, justify-content, align-items, gap
- Demo: a flex toolbar + content area

### M10 — clipping & overflow

- Scissor stack from `PaintCmd::PushClip` / `PopClip`
- `overflow: hidden`
- Later: stencil-based non-rectangular clips if we ever need them

## Cross-cutting concerns

- **DPI:** carry `scale_factor` from winit through layout (CSS px →
  physical px) and into the glyph atlas.
- **Color:** internal pipeline takes linear RGBA in 0..1. Surface is
  sRGB so the GPU does linear → sRGB on write. CSS color parsing (when
  added) will convert from sRGB-encoded values.
- **Coordinate system:** physical pixels, top-left origin, +Y down,
  matches CSS conventions.
- **Errors:** stages return `Result` only at I/O boundaries (font,
  image). Internal mismatches should be unreachable, not user-facing.
- **Dirty tracking:** out of scope until reflow performance becomes a
  problem. Currently every frame rebuilds the display list.
- **Threading:** single-threaded for now. wgpu calls happen on the main
  thread.

## Possible follow-ups (post-M10)

- Hit-testing → coarse pointer interactivity
- Scroll containers
- CSS `transform`
- `@font-face` / multiple fonts
- Animations (`requestAnimationFrame`-equivalent via render-loop hooks)
- Embedding into `egui` or another host (we already do this elsewhere)

## Versioning

While the workspace is pre-1.0:

- `wgpu` and `winit` versions are pinned at the workspace root and
  bumped together.
- `models` is the public surface most likely to be touched as we add
  more CSS / HTML coverage; breaking changes there are expected and not
  called out specially until 1.0.
