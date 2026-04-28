# wgpu-html — Roadmap

## Scope

A GPU renderer for **a static tree of HTML elements**, drawn through `wgpu`.

Explicit non-goals:

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
- No networking beyond image loading, no plugins.
- No accessibility tree, no print layout, no SVG rendering.

The user constructs a `Tree` programmatically from typed model structs or
by parsing an HTML string with `wgpu_html_parser::parse`, then hands it
to the renderer.

## Pipeline

```
HTML/CSS string
   │
   ▼  wgpu-html-parser           Tokenizer + tree builder + CSS parser
Tree<Node<Element>>
   │
   ▼  resolve styles (inline `style` attrs + <style> blocks + UA defaults,
      with full cascade, inheritance, and dynamic pseudo-classes)
CascadedTree
   │
   ▼  layout (block / inline / flex / grid; pure function, no scripting/reflow loop)
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
| `wgpu-html-tree`     | `Tree { root, fonts, interaction }`, `Node`, `Element`, `InteractionState` | done   |
| `wgpu-html-events`   | Typed DOM-style event structs: `HtmlEvent`, `MouseEvent`, `EventPhase`  | done   |
| `wgpu-html-parser`   | HTML tokenizer + tree builder + inline-CSS + stylesheet parser          | done   |
| `wgpu-html-style`    | Selector matching + cascade + `MatchContext` (`:hover`/`:active`)      | done   |
| `wgpu-html-text`     | Font database + cosmic-text shaping + glyph atlas                      | done   |
| `wgpu-html-layout`   | Block/flex/grid layout + IFC + image loading + hit testing              | done   |
| `wgpu-html-renderer` | wgpu device/surface + `DisplayList` consumption + pipelines             | done   |
| `wgpu-html`          | Facade: parse → cascade → layout → paint, interactivity, `PipelineTimings` | done   |
| `wgpu-html-demo`     | winit binary; font loading, event loop, scroll, `ProfileWindow`        | done   |

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

### M5 — text rendering ✅

- `wgpu-html-text` crate: font database, `cosmic-text`-based shaping, glyph atlas (shelf packer + GPU upload)
- `wgpu-html-renderer`: dedicated glyph pipeline (`glyph.wgsl`), per-glyph instanced quads, alpha-tested coverage
- `PaintCmd::Glyph` (effectively `DisplayList::glyphs`)
- `font-family` fallback list, `font-weight`, `font-style: italic`, `letter-spacing`, `text-transform`, `white-space: pre` vs collapse
- Demo: text rendering with external system font

### M6 — inline layout ✅

- Inline formatting context (IFC): line boxes, word-wrap, `text-align`
- Mixed inline runs (`<span>`, `<strong>`, etc.) inheriting style
- Demo: paragraphs with mixed bold / link spans

### M7 — backgrounds, borders, radii ✅

- SDF-based rounded-rect / border in the fragment shader (`quad.wgsl`)
- Per-side and uniform-corner border rendering; dashed/dotted patterns
- `background-clip: border-box | padding-box | content-box`
- Demo: cards with rounded backgrounds and colored borders

### M8 — images ✅ (landed)

Full image support lives in `wgpu-html-layout` and `wgpu-html-renderer`.
Covered:

- `<img>` with CSS `width` / `height` or HTML attribute sizing
- `background-image` (URL-backed; function images e.g. `linear-gradient`
  are still not painted)
- Schemes: `http(s)://` (ureq + rustls, redirect-following, retry with
  exponential backoff), `data:` URIs (base64 + percent-encoded), and
  local filesystem paths
- Formats: PNG, JPEG, GIF (animated), BMP, WebP (animated)
- Two-level process-wide cache (`raw_cache` + `sized_cache`) with TTL
  and byte-budget eviction; non-blocking via a bounded worker-thread pool
- `Tree::preload_queue` / `preload_image(url)` for startup prefetch
- `image_load_revision()` change counter for on-demand-redraw hosts
- Animated GIF/WebP: frame selection via a process-wide clock anchor
- Per-URL `Cache-Control: max-age` respected over the global TTL

Demo: `crates/wgpu-html-demo/html/img-test.html` and
`crates/wgpu-html-demo/html/gif.html`.

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

### M10 — grid ✅ (landed early)

CSS-Grid-Layout-1 lives in `wgpu-html-layout::grid`. Covered:

- `display: grid` / `inline-grid`
- `grid-template-columns` / `grid-template-rows` with `<length>`,
  `<percent>`, `auto`, `<flex>` (`fr`), and `repeat(<int>, <list>)`
  expansion
- `grid-auto-rows` / `grid-auto-columns` for implicit tracks
- Explicit placement (`grid-column-start/end`, `grid-row-start/end`,
  and the `grid-column` / `grid-row` shorthands) — line numbers
  and `span <n>`
- Auto-placement for unplaced items in source order, with
  `grid-auto-flow: row | column` (the `dense` variants accept the
  keyword for cascade fidelity)
- `gap` / `row-gap` / `column-gap`
- `justify-items` / `justify-self` / `align-items` / `align-self`
  (default `stretch`)
- `justify-content` / `align-content` for distributing the track
  block when the explicit tracks underfill the container
- 14 dedicated unit tests in `crates/wgpu-html-layout/src/tests.rs`

Deferred (see `spec/grid.md` §6):

- `grid-template-areas` and `grid-area` shorthand
- `minmax()` two-bound clamping (parsed; uses the max bound for
  now)
- `min-content` / `max-content` / `fit-content()` track sizes
- `repeat(auto-fill | auto-fit, …)` track-count resolution
- `dense` packing
- Named grid lines, negative line numbers, subgrid, masonry

Demo: `crates/wgpu-html-demo/html/grid.html` exercises a holy-
grail layout, a `repeat(4, 1fr)` photo gallery, and the row /
column auto-flow modes.

### M11 — clipping & overflow ✅ (landed early)

`overflow: hidden` clips descendants to the padding-box rect of
their containing block, including the rounded inner-padding edge
when the container has a `border-radius`. Implemented as a
CPU-side clip stack at paint time, a per-range scissor pre-pass,
and a fragment-shader SDF discard against a rounded mask.
Covered:

- `Style::overflow / overflow-x / overflow-y` resolution with
  `effective_overflow()` collapsing both axes
- `LayoutBox::overflow` carrying the resolved value into paint
- `DisplayList::clips: Vec<ClipRange>` partitioning quad / glyph
  instances into scissor-tagged runs, each carrying optional
  rounded corner radii
- `paint_box_in_clip()` clip stack with rectangle intersection
  for nested clips and inner-padding-edge radii (outer
  `border-radius` shrunk by the matching border thickness,
  matching browser behaviour)
- `QuadPipeline` and `GlyphPipeline` use a dynamic-offset
  uniform buffer to ship the active clip rect + radii per draw
  call; fragment shaders run a rounded-SDF discard on top of
  the rectangular `set_scissor_rect` pre-pass
- 3 layout tests + 6 paint tests covering propagation, range
  emission, rounded clipping, and padding-box-radii inset

Deferred (see `spec/overflow.md` §5):

- Independent per-axis clipping (`overflow-x: hidden;
  overflow-y: visible;`)
- Scroll containers with hit-test-aware scroll offsets and
  `Wheel`→`on_event` forwarding (scroll paint + drag is in M12)
- Composing more than one nested rounded clip
- `overflow: clip` distinct semantics
- Stacking-context promotion
- `clip-path`

Demo: `crates/wgpu-html-demo/html/overflow.html` — `visible` /
`hidden` / `hidden + border-radius` side by side.

### M12 — interactivity ⚠️ partial

See `spec/interactivity.md` for the full phase breakdown.

**M-INTER-1 (hover / press / click / focus chain) ✅**

- `InteractionState` on `Tree` (hover path, active path, scroll offsets,
  text selection, buttons bitmask, time origin)
- `interactivity::pointer_move`, `mouse_down`, `mouse_up`, `pointer_leave`
- Synthesised enter / leave (deepest-first leave, root-first enter)
- Click synthesis via deepest common ancestor; drag-select suppresses click
- `:hover` / `:active` cascade via `MatchContext::for_path` (focus is
  always `false` until M-INTER-2)
- `wgpu-html-events` crate: `HtmlEvent`, `MouseEvent`, `EventPhase`,
  `HtmlEventType`; both legacy (`on_click` slot) and typed (`on_event`)
  callbacks wired
- Demo wires up all mouse + scroll events; scrollbar drag implemented

**M-INTER-3 (text selection + clipboard) ✅ partial**

- `TextCursor` / `TextSelection` on `InteractionState`
- Drag-to-select; `select_all_text` / `selected_text` in `wgpu-html`
- `Ctrl+A` + `Ctrl+C` + `arboard` integration in the demo
- Selection highlight quads painted; caret overlay not yet done
- Word / line select (double-click / triple-click) not yet done

**M-INTER-4 (scroll) ⚠️ partial**

- `scroll_offsets_y: BTreeMap<Vec<usize>, f32>` on `InteractionState`
- Viewport scroll position + drag-scrollbar; `MouseWheel` scrolls
  viewport and detects deepest scrollable element
- Scrollbar quads painted in `paint.rs`
- `Wheel` events are not forwarded to element `on_event` callbacks yet

Not yet done: M-INTER-2 (`pointer-events`, `overflow` clip in hit-test,
double-click), M-INTER-5 (keyboard navigation), M-INTER-6 (re-cascade
caching).

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

## Possible follow-ups

- `:focus` / `:focus-visible` / `:disabled` / `:checked` pseudo-classes
  and focus management (Tab navigation, Enter/Space on focused elements)
- `pointer-events: none` and `overflow`-clip in hit testing (M-INTER-2)
- Keyboard navigation and caret movement (M-INTER-5)
- CSS `transform` (layout and hit-test impact)
- CSS `position: absolute / relative / fixed` with `top/right/bottom/left`
- `z-index` and stacking contexts
- `@font-face` / generic font families
- `calc()` / `min()` / `max()` / `clamp()` in length values
- `var(--foo)` CSS custom properties
- `@media` queries (re-cascade on resize)
- `clip-path` / SDF non-rectangular clips
- `wgpu-html-profiler` crate with ring-buffer history, GPU timing, and
  trace export (see `spec/profiler.md`)
- `wgpu-html-devtools` inspector crate (see `spec/devtools.md`)
- Render-loop hooks for engine-side animation (no JS; timeline-driven)
- Embedding into `egui` or another host

## Versioning

While the workspace is pre-1.0:

- `wgpu` and `winit` versions are pinned at the workspace root and
  bumped together.
- `models` is the public surface most likely to be touched as we add
  more CSS / HTML coverage; breaking changes there are expected and not
  called out specially until 1.0.
