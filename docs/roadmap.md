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
| `wgpu-html-renderer` | wgpu device/surface + `DisplayList` consumption + pipelines             | M1+M2  |
| `wgpu-html`          | Facade re-exporting `models`, `renderer`, `tree`                        | done   |
| `wgpu-html-demo`     | winit binary; builds a sample scene and runs the loop                   | M2     |

Future crates (split out only when they grow large enough to justify it):

- `wgpu-html-style` — selector/cascade/inheritance, computes `Style` per node
- `wgpu-html-layout` — box tree, block / inline / flex
- `wgpu-html-paint` — produces `DisplayList` from layout, owns the glyph atlas

For now `paint` types live inside `wgpu-html-renderer` and the rest are
TBD.

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

### M3 — paint a tree of `<div>` ⏭ next

- Walk a `Tree` and produce a `DisplayList` of solid quads
- Read `style` attribute → `models::css::Style`
  (initially: just `width`, `height`, `background-color`, `color`)
- Stack frames with explicit positions (no real layout yet); think of it
  as “absolute positioning only”
- Demo builds the same scene as M2 but as `Body{Div, Div, Div}`

### M4 — block layout

- Box tree from styled tree (anonymous boxes for inline fixups)
- Block formatting context: width-from-parent, height-from-content,
  margin / border / padding
- No floats, no flex yet
- Demo: a header + a column of cards with auto-sized backgrounds

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
