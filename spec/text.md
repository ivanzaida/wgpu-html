# wgpu-html — Text Rendering Spec

The plan for moving from "text nodes contribute zero height" to a real
inline formatting context with shaped glyphs on the GPU.

Status as of T3: single-line shaped text rendered end-to-end against a
host-registered font, with cascade inheritance pulled forward. T4–T7
still match the original plan.

Companion to `roadmap.md` (engine milestones — this fleshes out M5/M6
and pulls some inheritance work forward) and `status.md` (current
gaps).

---

## 1. Goals

- Render text from `Element::Text(_)` leaves with shaped glyphs,
  honouring `font-family / font-size / font-weight / font-style /
  color / line-height / letter-spacing / text-align / white-space`.
- **Host-supplied external fonts.** The host loads font bytes from
  wherever it likes (disk, `include_bytes!`, network) and registers
  them on the document.
- Single-line and multi-line layout with word-aware line breaking.
- Mixed inline runs (`<span>`, `<strong>`, `<em>`, …) flowing through
  the same line boxes with their own per-run cascaded style.
- DPI-correct: physical pixels everywhere, scale factor flows from
  winit through layout into the atlas.

## 2. Non-goals (M5/M6 scope)

- No web fonts via URL fetch — the host is responsible for bytes.
- No `@font-face` parsing in the first pass. CSS only refers to
  fonts by family / weight / style; the binding is driven by the
  Tree's font registry.
- No bidi, no shaping for complex scripts beyond what
  HarfBuzz-via-rustybuzz gives us out of the box.
- No vertical writing modes.
- No `font-stretch`, no `font-variant`, no font features
  (`font-feature-settings`, `font-variant-ligatures`).
- No `text-shadow`, no emoji color fonts, no `letter-spacing` in
  the middle of a kern (we treat it as a flat post-shape advance
  fixup).
- No subpixel antialiasing — straight alpha mask first, SDF later
  if needed.

## 3. Hard constraint: fonts live in the Tree

> **External fonts are registered as part of the document tree, not
> as a process-global resource.** Each `Tree` carries its own
> `FontRegistry`; cascade and layout consult that registry alone.

Rationale:

- **No global mutable state.** Two `Tree`s in the same process can
  carry different font sets without contention.
- **Same lifecycle as the rest of the document.** Bytes are dropped
  when the tree is dropped; nothing dangles.
- **Mirrors how the engine already treats CSS.** Inline `<style>`
  blocks are owned by the tree, not by a global stylesheet store —
  fonts follow the same rule.
- **Host stays in charge of provenance.** No fetcher, no
  `@font-face` magic. The host explicitly hands over `Vec<u8>`s
  with metadata.
- **Trivial to test.** A test just builds a tree, registers an
  embedded font, parses + renders, and asserts.

Landed API in `wgpu-html-tree` (`src/fonts.rs`, `src/lib.rs`):

```rust
pub struct Tree {
    pub root:  Option<Node>,
    pub fonts: FontRegistry,
}

#[derive(Default, Debug, Clone)]
pub struct FontRegistry { /* faces: Vec<FontFace> */ }

#[derive(Debug, Clone)]
pub struct FontFace {
    pub family: String,                  // CSS-side family name
    pub weight: u16,                     // 100..900, 400 default
    pub style:  FontStyleAxis,           // Normal / Italic / Oblique
    pub data:   std::sync::Arc<[u8]>,    // shared & cheap to clone
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontStyleAxis { Normal, Italic, Oblique }

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct FontHandle(pub usize);

impl Tree {
    pub fn register_font(&mut self, face: FontFace) -> FontHandle;
}

impl FontRegistry {
    pub fn register(&mut self, face: FontFace) -> FontHandle;
    pub fn find(&self, family: &str, weight: u16, style: FontStyleAxis)
        -> Option<FontHandle>;
    pub fn find_first(&self, families: &[&str], weight: u16,
        style: FontStyleAxis) -> Option<FontHandle>;
    // get / iter / len / is_empty
}
```

`FontFace::regular(family, data)` is a 400-weight / Normal-style
shorthand. `FontHandle` is `Ord` so the text crate can pick the
"first registered" handle deterministically.

Typical host wiring (today's demo):

```rust
let mut tree = wgpu_html_parser::parse(html);
let bytes = std::sync::Arc::<[u8]>::from(font_bytes_vec.into_boxed_slice());
tree.register_font(FontFace {
    family: "DemoSans".into(),
    weight: 400,
    style:  FontStyleAxis::Normal,
    data:   bytes,
});
```

`FontRegistry` is plain data — no fancy traits, no async, no
`dyn`. `wgpu-html-text` consumes `&FontRegistry` and converts it
into its shaper-side database on demand.

## 4. Pipeline

```
Tree (with fonts)
   │
   ▼  cascade  (now also runs an inheritance pass)
CascadedTree                (Style.color + font_* per node, inherited)
   │
   ▼  layout_with_text       (text leaves shape via &mut TextContext)
LayoutBox tree               (text_run: Option<ShapedRun> on text leaves)
   │
   ▼  paint                  (glyph quads with atlas UVs)
DisplayList { quads, glyphs }
   │
   ▼  renderer               (quad pipeline + glyph pipeline, one pass)
Frame
```

Landed signatures:

```rust
// wgpu-html-layout
pub fn layout_with_text(
    tree:       &CascadedTree,
    text_ctx:   &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale:      f32,             // CSS px → physical px
) -> Option<LayoutBox>;

// Back-compat wrapper for callers that don't render text. Builds a
// throwaway empty TextContext at scale 1.0; existing layout tests
// keep using this.
pub fn layout(
    tree: &CascadedTree, viewport_w: f32, viewport_h: f32,
) -> Option<LayoutBox>;

// wgpu-html (facade)
pub fn paint_tree_with_text(
    tree:       &Tree,
    text_ctx:   &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale:      f32,
) -> DisplayList;
pub fn paint_tree(tree: &Tree, vw: f32, vh: f32) -> DisplayList; // back-compat
```

Note the deviation from the original plan:

- **`paint` does not take fonts.** All shaping happens during layout;
  glyph UVs and rasters are already packed into the atlas by the time
  `paint_box` runs. Paint just walks `b.text_run.glyphs` and emits
  one `GlyphQuad` per glyph at `content_rect + (glyph.x, glyph.y)`.
- **The signature is `&mut TextContext`, not `&FontRegistry, &mut Atlas`.**
  `TextContext` bundles the cosmic-text bridge, the CPU atlas, and a
  `SwashCache` so the API surface stays compact.

Renderer changes (`wgpu-html-renderer`):

- Two pipelines now: the existing `QuadPipeline` plus a new
  `GlyphPipeline` (instanced textured quads). Both run in the same
  render pass, quads first then glyphs, so text sits over backgrounds
  and borders without sorting.
- One bind group per pipeline: globals uniform, plus the glyph
  pipeline adds an `R8Unorm` texture and a linear sampler.
- New `pub const GLYPH_ATLAS_SIZE: u32 = 2048` — the host's CPU
  atlas in `wgpu-html-text` must be created at the same size for
  uploads to land 1:1.
- `Renderer::glyph_atlas_texture() -> &wgpu::Texture` lets the host
  call `Atlas::upload(&queue, &texture)` to flush dirty rasters
  before each draw.

## 5. Library choice

Use **cosmic-text 0.12** as the shaper / line-breaker:

- Ships a `FontSystem` + `fontdb::Database` we can populate from
  `FontRegistry` via `load_font_source(Source::Binary(Arc<dyn AsRef<[u8]>>))`.
- Wraps rustybuzz for shaping and unicode-bidi / unicode-linebreak
  for break opportunities.
- Used in production (egui, iced, …); active upstream.

Why not raw rustybuzz / swash:

- We'd reimplement line-breaking (UAX 14), white-space collapsing,
  and word boundaries.
- The win is "more control"; the cost is several months of
  re-inventing shaping ergonomics. Not worth it pre-1.0.

The bridge layer (`wgpu-html-text::FontDb`, `src/font_db.rs`) caches
a `cosmic_text::FontSystem` keyed by the underlying `Arc<[u8]>`
identity (`Arc::as_ptr` cast to `*const u8`). Re-syncing against the
same registry is a true no-op; a face whose `Arc` was swapped in
place gets re-loaded; a face that disappeared from the registry is
removed from `fontdb` too. The `FontSystem` is constructed with
`new_with_locale_and_db("en-US", Database::new())` — **no system
fonts** — so the registry is the single source of truth.

Glyph cache: `cosmic_text::SwashCache` lives inside `TextContext`;
the GPU atlas is owned separately by the renderer (T2 design).

## 6. Font matching

Implemented today (`FontRegistry::find`, `find_first`):

1. Walk the comma-separated `font-family` list left-to-right
   (`find_first`).
2. Within each family name (case-insensitive ASCII compare), pick
   the face whose `(weight, style)` minimises a score:
   - **Style band** (× 1,000,000): exact (0) > italic↔oblique
     interchange (1) > italic-or-oblique target falling to Normal
     (2). Ties on the band defer to the weight distance.
   - **Weight distance**: absolute `|candidate − target|`, plus a
     +10,000 wrong-direction penalty when the target prefers heavier
     (`> 500`) but the candidate is lighter, or prefers lighter
     (`< 400`) but the candidate is heavier. `[400, 500]` is
     bidirectional. The 10,000 floor keeps any right-direction match
     strictly better than any wrong-direction match (max raw weight
     gap is 800).
3. Ties on score break toward the **later-registered** face, so a
   host can override an earlier registration by re-registering.
4. If the whole `find_first` walk misses, the layout text path falls
   back to `FontDb::first_handle` (lowest-numbered loaded handle).

Generic family names (`serif`, `sans-serif`, `monospace`) get no
special treatment yet — a host that wants `sans-serif: Inter` must
register `Inter` under that family explicitly. Mappings are a
`Tree::set_generic_family(...)` follow-up.

If `FontRegistry` is empty, text leaves shape to zero size, same as
the pre-T3 behaviour.

## 7. Glyph atlas & textured pipeline

Atlas (`wgpu-html-text::Atlas`, `src/atlas.rs`):

- Single `R8Unorm` CPU buffer, default 2048×2048, fixed size in T2/T3
  (no doubling yet — overflow returns `None`; T7 brings LRU).
- Shelf packer: glyphs go onto horizontal shelves stacked top-to-
  bottom; each shelf locks its height to its first glyph; a glyph
  that doesn't fit horizontally or vertically opens a new shelf.
- Dirty-rect list: every `insert` appends a rect; `flush_dirty(sink)`
  drains them into a generic closure `(rect, &[u8])`.
- `upload(&Queue, &Texture)` is the wgpu-flavoured wrapper around
  `flush_dirty` using `queue.write_texture` with the modern
  `TexelCopyTextureInfo` / `TexelCopyBufferLayout` names.
- `clear()` zeros the buffer, resets the packer, and queues a full-
  atlas dirty rect so the next flush re-uploads everything (T7's
  scale-factor-changed path).
- `AtlasEntry::uv_min/uv_max(atlas_w, atlas_h)` for normalised UVs.

Pipeline (`wgpu-html-renderer::glyph_pipeline`, `shaders/glyph.wgsl`):

- WGSL: instanced textured quads. The fragment samples the R8 atlas
  and multiplies the coverage by the per-instance color (RGB + α).
  Premultiplied-alpha blending via `wgpu::BlendState::ALPHA_BLENDING`.
- Bind group: globals uniform (binding 0), atlas texture (1),
  filtering sampler (2). Bind group layout dropped after pipeline +
  bind group construction; the sampler is held on the pipeline
  struct purely to keep the underlying GPU object alive.
- Instance: `{ pos: vec2, size: vec2, color: vec4, uv_min: vec2,
  uv_max: vec2 }`. Same unit-quad geometry as the quad pipeline.
- Drawn after the quad pass in the same render pass.

Display list (`wgpu-html-renderer/src/paint.rs`):

```rust
pub struct GlyphQuad {
    pub rect: Rect,
    pub color: Color,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

pub struct DisplayList {
    pub quads:  Vec<Quad>,
    pub glyphs: Vec<GlyphQuad>,
}
```

The original `DisplayItem::GlyphRun` aggregate didn't materialise —
each glyph is its own one-quad entry. Simpler; T7 might revisit if
there's a measurable batching win.

`paint_box` walks `b.text_run.glyphs` and pushes one `GlyphQuad`
per shaped glyph at `(content_rect.x + glyph.x, content_rect.y +
glyph.y)` with `b.text_color` (default opaque black).

## 8. Inline formatting context (minimal)

**Status: not yet implemented (T4).**

The IFC has to land in the same milestone as text layout — text
without it is just one-line boxes that no one wants. T3 sidesteps it
by treating each `Element::Text` as a single-line "block-ish" box
sized to the run's width × line-height.

Per block-level box that contains inline content (T4 plan):

1. **Build a flat "inline level box list"** from the children:
   `Vec<InlineItem>` where each item is either a `TextRun` (one
   `Element::Text`'s shaped glyphs) or an `InlineNested` (a
   recursive IFC for a `<span>` and similar).
2. **Break into lines.** Walk items left-to-right, accumulating
   width. At each break opportunity (from cosmic-text), check if
   the running width fits in the container's content width. If
   not, close the line and start a new one.
3. **Vertical metrics.** Each line's height is the max of its
   items' line-heights (CSS line-height applied per inline-level
   element); baseline is the max ascent.
4. **Justification (M5: only `text-align: left | right | center`).**
   `justify` left to a follow-up.
5. Emit one `LayoutBox::children` entry per line, kind `LineBox`,
   carrying its glyph runs and per-run cascaded style.

`white-space: nowrap | pre | pre-wrap` follow the standard rules
(for `pre`, no soft breaks; preserve newlines as forced breaks).

Anonymous block boxes (text adjacent to block siblings) are
wrapped in synthetic anonymous `<span>`-equivalents inside an
anonymous block — handled in a small normalisation pass before
layout, so the IFC builder only sees inline-only parents.

## 9. CSS coverage today (post-T3 + inheritance pull-forward)

**Honoured during shaping right now:**

- `color` — read from the text leaf's cascaded `style.color` (filled
  in by inheritance), resolved through `wgpu_html_layout::resolve_color`.
  Defaults to opaque black when no ancestor sets a color.
- The "first registered font" picks family/weight/style today.
  T4 will read `font-family / font-weight / font-style` from the
  inherited cascade.

**Parsed and inherited but not yet consumed by the text path:**

`font-family`, `font-size`, `font-weight`, `font-style`,
`line-height`, `letter-spacing`, `text-align`, `text-transform`,
`white-space`, `text-decoration` — they reach text leaves via the
inheritance pass below; the text layout still hardcodes 16px /
20px line-height / no spacing in T3.

**Cascade inheritance landed early.** `wgpu-html-style::cascade`
gained a per-node `inherit_into(child, parent)` that fills in unset
values for the standard inheriting set:

```
color, font_family, font_size, font_weight, font_style,
line_height, letter_spacing, text_align, text_transform,
white_space, text_decoration, visibility, cursor
```

Inheritance runs after rule-merge + inline-style merge, so an
explicit child value still wins. UA defaults are still deferred:
hosts must set `body { font-family, color, font-size, ... }`
themselves (§9 of `status.md`).

This list deviates slightly from the plan: `direction` and
`text_decoration_color` weren't included because they aren't modelled
in `wgpu-html-models` yet. Add them when the model gains the fields.

**Deferred to T4:**

- `line-height` consumed for line-box height (only `<number>` and
  `<length>`; `normal` resolves to `font.metrics.height * scale`).
- `text-align: left | right | center | start | end` (start/end
  collapse to left/right with `dir: ltr` until bidi lands).
- `white-space: normal | nowrap | pre`.

**Deferred to T5/T6:**

- Mixed inline runs through `<span>`, `<strong>`, `<b>`, `<em>`,
  `<i>`, `<u>`, `<a>`, `<code>`, `<small>`, `<mark>`, `<sub>`,
  `<sup>`, `<kbd>`, `<samp>`, `<var>`.
- `text-decoration: underline | line-through` painted as quads
  beneath / through the run.
- `vertical-align: baseline | super | sub` (only the three).
- `letter-spacing: <length>` post-shape advance fixup.

## 10. Public API surface

```
wgpu-html-tree
  + FontFace, FontRegistry, FontHandle, FontStyleAxis            (T1, done)
  + Tree::register_font, Tree::fonts                             (T1, done)

wgpu-html-text  (new crate)
  + Atlas (CPU shelf packer, dirty rects, upload)                (T2, done)
  + FontDb (cosmic-text bridge over a FontRegistry)              (T2, done)
  + TextContext { font_db, atlas, swash, glyph_cache }           (T3, done)
  + TextContext::shape_and_pack(text, font, size, line_height)   (T3, done)
  + ShapedRun { glyphs, width, height, ascent }                  (T3, done)
  + PositionedGlyph { x, y, w, h, uv_min, uv_max }               (T3, done)

wgpu-html-style
  + cascade inheritance pass for the standard set                (T3, done)

wgpu-html-layout
  ! layout_with_text(tree, &mut TextContext, vw, vh, scale)      (T3, done)
  ! layout(...) is now a back-compat wrapper                     (T3, done)
  + LayoutBox.text_run + text_color                              (T3, done)
  - Inline formatting context, BoxKind::LineBox                  (T4, planned)

wgpu-html-renderer
  + GlyphPipeline (textured)                                     (T3, done)
  + R8 atlas texture + linear sampler + bind group               (T3, done)
  + GLYPH_ATLAS_SIZE constant; glyph_atlas_texture() accessor    (T3, done)
  + DisplayList.glyphs + push_glyph + GlyphQuad                  (T3, done)

wgpu-html
  + paint_tree_with_text(tree, ctx, vw, vh, scale)               (T3, done)
  + paint::paint_box emits glyph quads from text_run             (T3, done)
```

`wgpu-html-text` ends up as the heaviest new dep (cosmic-text +
its tree). Everything else stays light.

## 11. Phases

Each phase ends in something runnable in `wgpu-html-demo`.

### T1 — Font registry on the Tree (no rendering) ✅ DONE

- Added `FontFace, FontRegistry, FontHandle, FontStyleAxis,
  Tree::register_font` to `wgpu-html-tree`.
- `register_font` returns a `FontHandle` indexing into the
  registry; duplicates allowed (later registration wins on tie via
  the strict `>` in `find`).
- 11 unit tests in `wgpu-html-tree` covering register / lookup /
  case-insensitive family / weight bias / style swap / multi-family
  fallback / empty registry.
- No engine changes, no library deps yet — just `std::sync::Arc`.

### T2 — `wgpu-html-text` crate skeleton + atlas ✅ DONE

- New crate `wgpu-html-text` with `wgpu` + `wgpu-html-tree` +
  `cosmic-text 0.12 (default-features=false, std + swash +
  shape-run-cache)` deps.
- `Atlas` (CPU side, `src/atlas.rs`): shelf packer + dirty rect list.
  `flush_dirty(sink)` for testable drain; `upload(&Queue, &Texture)`
  for wgpu callers.
- `FontDb` (`src/font_db.rs`): wraps `cosmic_text::FontSystem` built
  with an empty `fontdb::Database` (no system fonts). Keyed by
  `Arc::as_ptr` so re-syncing against the same registry is a no-op,
  swapped `Arc`s re-load, dropped registry entries are removed from
  fontdb too.
- 11 atlas tests + 5 font_db tests.
- No GPU pipeline yet; renderer untouched.

### T3 — Single-line shaped text ✅ DONE

- `wgpu-html-text::TextContext` bundles `FontDb + Atlas +
  cosmic_text::SwashCache + glyph_cache: HashMap<CacheKey,
  AtlasGlyph>`. `shape_and_pack(text, font, size_px, line_height)
  -> Option<ShapedRun>` shapes via cosmic-text `Buffer` (unbounded
  width, single layout run), rasters each glyph through SwashCache,
  packs into the atlas, and emits `PositionedGlyph` quads with
  pre-computed UVs.
- `wgpu-html-renderer` gained the textured `GlyphPipeline` next to
  the existing `QuadPipeline`; both run in one render pass, quads
  first then glyphs. New `R8Unorm` atlas texture + linear sampler
  + 3-binding bind group. `DisplayList { quads, glyphs }`,
  `push_glyph(rect, color, uv_min, uv_max)`. Public
  `GLYPH_ATLAS_SIZE = 2048` plus `Renderer::glyph_atlas_texture()`.
- `wgpu-html-layout::layout_with_text(tree, &mut TextContext, vw,
  vh, scale)` is the new entry point; `layout(...)` keeps the old
  three-arg shape and creates a throwaway empty `TextContext`
  internally so the existing 52 layout tests didn't have to change.
  `LayoutBox` gained `text_run: Option<ShapedRun>` and
  `text_color: Option<Color>`. The text branch in `layout_block`
  shapes via `shape_text_run` (T3 default: first registered handle,
  16px, 20px line-height) and reads the resolved `color` from the
  cascaded text node's style.
- `wgpu-html::paint_tree_with_text(tree, &mut TextContext, vw, vh,
  scale)` is the new high-level entry; it `text_ctx.sync_fonts(&
  tree.fonts)` first so freshly-registered faces are loaded before
  shaping. `paint_box` walks `text_run.glyphs` and emits one
  `GlyphQuad` per shaped glyph.
- **Cascade inheritance pulled forward** (originally T4) because
  text-related properties are useless without it. See §9.
- Demo (`crates/wgpu-html-demo`): `hello-text.html` (`<p>` with a
  cream background and gold border around `Hello, world.`).
  `App` holds a `TextContext::new(GLYPH_ATLAS_SIZE)`. Each frame:
  parse → register the demo font (a `OnceLock<Arc<[u8]>>` with
  bytes loaded from the first available system-font path so the
  bridge's `Arc::as_ptr` cache stays valid across frames) → call
  `paint_tree_with_text` → `text_ctx.atlas.upload(&renderer.queue,
  renderer.glyph_atlas_texture())` → render.

Deviations from the original T3 plan worth noting:

- Layout API: `&mut TextContext` instead of separate `&FontRegistry`
  and `scale` plus a hidden atlas. Folds three params into one.
- Paint signature: doesn't take fonts. All shaping and atlas
  packing happens during layout; paint just emits the prepared
  quads.
- `LayoutBox` kept `BoxKind::Text` and added `Option`-typed
  `text_run` / `text_color` instead of growing a `BoxKind::Inline`
  variant. Less rippling through existing call sites.
- Demo doesn't ship a font asset; it walks a candidate list of
  common system-font paths. Hosts that ship their own asset can
  swap in `include_bytes!`.

### T4 — Line breaking + multi-property consumption ▶ NEXT

- IFC builder lives in `wgpu-html-layout::inline`. Walks a block
  parent's children, batches text into lines using cosmic-text's
  break opportunities, fits each line to `inner_width`.
- `white-space: normal | nowrap | pre`.
- `text-align: left | right | center`.
- `font-size`, `font-family`, `font-weight`, `font-style` from
  the cascade (already inherited; just need to be read in
  `shape_text_run`).
- Demo: a paragraph that wraps cleanly across the viewport on
  resize.

Inheritance was already done in T3. UA defaults (e.g. `<h1> {
font-size: 2em }`) still out of scope; hosts set them explicitly.

### T5 — Mixed inline runs ⏳

- `<span>`, `<strong>`, `<em>`, `<a>`, `<code>` participate in the
  same IFC with their own cascaded style.
- Per-run font matching (a `<strong>` inside Inter 400 picks Inter
  700 if registered; falls back to fake-bold synthesis as a
  follow-up).
- Demo: a paragraph with bold + italic + monospace + link runs.

### T6 — Decorations + vertical alignment + letter-spacing ⏳

- `text-decoration: underline | line-through` painted under /
  through the run.
- `vertical-align: super | sub` (1/3em raise + size scale of 0.83).
- `letter-spacing: <length>` post-shape advance fixup.

### T7 — Atlas eviction + DPI changes ⏳

- Hooking `winit`'s `scale_factor_changed` into a re-rasterise of
  any cached glyphs at the new size (`Atlas::clear` already queues
  the full-atlas dirty rect — the missing piece is invalidating
  `TextContext::glyph_cache`).
- LRU-style eviction in the atlas instead of "blow the cache".
- Atlas grow / double when full.

## 12. Open questions

- **Generic families** (`sans-serif`, `serif`, `monospace`,
  `system-ui`, `cursive`, `fantasy`). Push to T5+: a host-side
  mapping like `Tree::set_generic_family("sans-serif", "Inter")`.
- **`@font-face` from CSS.** Still out of M5/M6 scope, but the
  natural extension would parse it into a synthetic
  `tree.register_font` call with `data` resolved through a
  host-supplied `FontResolver` callback (since we don't fetch).
- **Subpixel antialiasing.** Skipped initially; revisit if 16px
  body text looks rough on standard-DPI displays.
- **Emoji / color fonts.** Out of scope; the path would be a
  separate `R8` → `Rgba8` atlas + COLR/CPAL or CBDT support in
  cosmic-text.
- **Bidi.** RTL is a multi-quarter project; ignored entirely until
  a host needs it.
- **Hot font swaps.** Re-registering a face with a different
  `Arc<[u8]>` flips the bridge's identity check and re-loads the
  face into fontdb. Confirmed by the `font_db::tests::
  sync_replaces_when_arc_identity_changes` test in T2; the
  knock-on cascade-cache invalidation (cosmic-text shape cache
  still has stale shapes) is left for a follow-up.
- **`include_bytes!` vs system-font lookup in the demo.** Today
  the demo uses a small candidate-paths table; that's fine for
  development but not a great default for shipping. T4+ might
  switch to a checked-in OFL-licensed font.

---

## Summary

T1 landed the structural commitment first: fonts belong to the
`Tree`. T2–T3 stood the renderer up to draw shaped glyphs from a
single registered face, and pulled cascade inheritance forward
because none of the text-related properties make sense without it.
T4 will add real line breaking and start consuming the rest of the
inherited typography properties; T5–T6 will polish multi-run text
and decorations; T7 will cope with DPI and atlas pressure. The
whole thing still hangs off the constraint in §3 — fonts owned by
the `Tree`, no globals, no fetcher, no `@font-face` magic.
