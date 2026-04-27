# wgpu-html — Roadmap

## Scope

A GPU renderer for **a static tree of HTML elements**, drawn through `wgpu`.

Explicit non-goals:

- Not a browser. No HTML parsing, no CSS parsing, no DOM.
- **No JavaScript, ever.** Not "deferred", not "later" — out of
  scope for the lifetime of the project. No `<script>` execution,
  no JS engine embed (V8 / SpiderMonkey / QuickJS / …), no
  `eval`-equivalent, no scripting hooks, no `addEventListener`-
  style callbacks, no `requestAnimationFrame` JS callback, no
  `eval` of inline `on*=` attributes, no JS bridge from the host.
  Interactivity is expressed entirely through CSS pseudo-classes
  and host-driven element-state mutation (see
  `spec/interactivity.md`). `<script>` content stays parsed-but-
  inert.
- No networking, no plugins.
- Interactivity / events / hit-testing — covered by
  `spec/interactivity.md` as a separate, non-JS surface.
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
  into a list of `Rule { selectors, declarations }`. A `Selector`
  carries one subject compound (`tag`, `id`, `classes`, `universal`)
  plus an optional `ancestors` chain for descendant combinators
  (`.row .item`, `div p span`, …). Specificity sums across all
  compounds in the chain, matching standard CSS. Other combinators
  (`>`, `+`, `~`) and pseudo-classes / pseudo-elements still drop
  the rule.
- New `wgpu-html-style` crate:
  - `cascade(&Tree) -> CascadedTree` walks the tree, collects every
    `<style>` block's text, parses it once, and computes a final
    `Style` per element
  - Cascade order: matched rules in ascending specificity → element's
    inline `style="…"` attribute on top
  - `matches_selector_in_tree` checks the subject compound and walks
    the element's ancestor chain to evaluate descendant requirements;
    `matches_selector` is the simple-case wrapper used when no
    ancestor context is available
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

### M9 — flexbox ✅ (landed early)

The full CSS-Flexbox-1 algorithm now lives in
`wgpu-html-layout::flex`. Covered:

- `display: flex` / `inline-flex`
- `flex-direction` (row / row-reverse / column / column-reverse)
- `flex-wrap` (nowrap / wrap / wrap-reverse), multi-line lines
- `flex-grow` / `flex-shrink` / `flex-basis` with the iterative
  freeze loop (CSS-Flex-1 §9.7), min/max clamping, and proper
  `flex` shorthand expansion in the parser
- `justify-content` (flex-start / flex-end / center / space-between /
  space-around / space-evenly, plus the start / end / left / right
  aliases)
- `align-items` and per-item `align-self`
- `align-content` for multi-line containers (default `stretch`)
- `order` (stable sort by order, then source index)
- `gap` / `row-gap` / `column-gap` (per-axis longhands win over
  the shorthand)
- `margin: auto` on flex items: absorbs free space on the main axis
  and consumes leftover line cross space on the cross axis
- `min-width` / `max-width` / `min-height` / `max-height` clamping
  on flex items (and on plain blocks too, via the shared
  `clamp_axis` helper in `lib.rs`)
- Block-level `margin-left: auto; margin-right: auto;` centering for
  fixed-width blocks outside flex containers

Deferred:

- Baseline alignment of non-text-bearing flex items (we currently
  track ascent/descent only inside the inline pass). Falls back to
  `flex-start`.
- Intrinsic `flex-basis: content` measurement (we currently treat an
  unspecified basis with no main size as 0; good enough for the
  ubiquitous `flex: 1` pattern).

Demo: `crates/wgpu-html-demo/html/flex-grow.html` exercises grow,
clamping, wrap + align-content, align-self, auto-margin, and order.

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

- Pointer interactivity (mouse events, hover / focus pseudo-classes,
  text selection) — see `spec/interactivity.md`. Stays JS-free; the
  surface is host-driven element-state mutation that re-feeds the
  cascade.
- Scroll containers
- CSS `transform`
- `@font-face` / multiple fonts
- Render-loop hooks for animation (engine-side, no JS callback) —
  *not* `requestAnimationFrame`; that name implies a JS callback
  contract we will never have.
- Embedding into `egui` or another host (we already do this elsewhere)

## Versioning

While the workspace is pre-1.0:

- `wgpu` and `winit` versions are pinned at the workspace root and
  bumped together.
- `models` is the public surface most likely to be touched as we add
  more CSS / HTML coverage; breaking changes there are expected and not
  called out specially until 1.0.
