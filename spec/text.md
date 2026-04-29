# wgpu-html — Text Rendering Spec

How text leaves shape, lay out, and paint. Companion to
`roadmap.md` (M5/M6) and `status.md`.

Status: shipped. Cascaded text shapes through cosmic-text against
host-registered fonts; single-line and multi-line paragraphs flow
through an inline formatting context with rich-text spans, word-
boundary breaks across `<strong>` / `<em>` / `<a>` / `<mark>`,
per-line backgrounds and decorations, and per-glyph colour for
mixed-attribute runs. The renderer paints in two passes for
gamma-correct blending. Fonts live on the document `Tree` — no
process-global state, no `@font-face` fetcher.

---

## 1. Goals

- Render `Element::Text` leaves with shaped glyphs, honouring
  `font-family / font-size / font-weight / font-style / color /
  line-height / letter-spacing / text-align / white-space /
  text-transform / text-decoration`.
- **Host-supplied fonts.** The host loads bytes (disk,
  `include_bytes!`, network) and registers them on the `Tree`;
  cascade and layout consult the per-document registry.
- Multi-line paragraph layout with word-aware breaks across
  inline-element boundaries.
- DPI-correct: physical pixels everywhere, scale factor flows
  from winit through layout into the atlas.
- Gamma-correct blending: the GPU treats text as alpha coverage
  in display space, not as sRGB pre-multiplied colour.

## 2. Supported CSS

Resolved end-to-end (`crates/wgpu-html-style` cascade →
`crates/wgpu-html-text` shape → `crates/wgpu-html-layout` IFC →
paint):

| Property              | Coverage                                                                  |
|-----------------------|---------------------------------------------------------------------------|
| `color`               | Per-glyph foreground; resolved via `resolve_color`                        |
| `font-family`         | Comma list walked left-to-right; quoted names trimmed; generic CSS keywords (`sans-serif`, `serif`, `monospace`, `cursive`, `fantasy`, `system-ui`, `ui-*`, `-apple-system`, `BlinkMacSystemFont`) fall back to the best-scoring registered face if no listed family matched |
| `font-weight`         | `100..900`, `normal/bold/bolder/lighter` (no parent-context shift yet)    |
| `font-style`          | `normal/italic/oblique` → `FontStyleAxis`                                 |
| `font-size`           | `Px / Em / Rem / Vw / Vh / Vmin / Vmax / %` plus `calc()`/`min`/`max`     |
| `line-height`         | Same length set; defaults to `1.25 × font-size`; bare numbers TBD          |
| `letter-spacing`      | Lengths; applied as a post-shape per-glyph offset                         |
| `text-transform`      | `uppercase / lowercase / capitalize` applied pre-shape                    |
| `text-align`          | `left / right / center / start / end` (`justify` falls through to left)   |
| `text-decoration`     | `underline / line-through / overline` (whitespace list; `none` resets)    |
| `background-color`    | Inline-element wrapper backgrounds expand per-line (`<mark>`)             |
| `white-space: normal` | Whitespace runs collapse to a single ASCII space pre-shape                |
| `<input>` / `<textarea>` `placeholder` | Shaped + painted as the box's text run when the field has no value/content; colour = cascaded `color × alpha 0.5`; single-line input vertically centred + horizontally clipped; textarea soft-wrap top-aligned. See §11.8 for the full behaviour matrix. |

**Inheritance.** `wgpu-html-style::cascade` runs an
`inherit_into(child, parent)` pass that fills the standard
inheriting properties (`color`, `font-*`, `line-height`,
`letter-spacing`, `text-align`, `text-transform`, `white-space`,
`text-decoration`, `visibility`, `cursor`) for any property a
node didn't set explicitly. `inherit / initial / unset` keywords
are honoured before the implicit pass.

**UA stylesheet** (`crates/wgpu-html-style/src/ua.rs`,
prepended so author rules win on source-order ties):

- `head, style, script, meta, link, title, noscript, template,
  source, track, base, param, col, colgroup → display: none`
- `body → margin: 8px`; `p / blockquote / pre / ul / ol / dl /
  address → margin: 16px 0`; per-heading margins (21/20/19/21/22/
  26 px) + bold + descending sizes
- `ul, ol → padding-left: 40px`; `dd → margin-left: 40px`;
  `blockquote → margin: 16px 40px`; `hr → 1px solid gray`
- `b, strong → bold`; `i, em → italic`; `u, ins → underline`;
  `s, del, strike → line-through`; `code, kbd, samp, pre →
  monospace`; `a → blue + underline`; `mark → yellow + black`
- `small / sub / sup → 13px`; `sub/sup` parse `vertical-align` but
  the baseline shift isn't applied yet (font-size only)

## 3. Pipeline

```
Tree (root + fonts + interaction)
   │
   ▼  cascade               (UA + author + inline + inheritance)
CascadedTree                 (Style fully resolved per node)
   │
   ▼  layout_with_text       (IFC routes through TextContext)
LayoutBox tree               (text leaves carry ShapedRun + colour)
   │
   ▼  paint                  (one GlyphQuad per shaped glyph)
DisplayList { quads, glyphs, images }
   │
   ▼  renderer               (quad → glyph passes, gamma-correct)
Frame
```

Public entry points:

```rust
// wgpu-html-layout
pub fn layout_with_text(
    tree:       &CascadedTree,
    text_ctx:   &mut TextContext,
    viewport_w: f32,
    viewport_h: f32,
    scale:      f32,             // CSS px → physical px
) -> Option<LayoutBox>;
pub fn layout(tree, vw, vh) -> Option<LayoutBox>; // no-text wrapper

// wgpu-html (facade)
pub fn paint_tree_with_text(
    tree, text_ctx, viewport_w, viewport_h, scale,
) -> DisplayList;
pub fn paint_tree(tree, vw, vh) -> DisplayList; // no-text wrapper
pub fn compute_layout(tree, text_ctx, vw, vh, scale) -> Option<LayoutBox>;
pub fn paint_tree_returning_layout(tree, text_ctx, vw, vh, scale)
    -> (DisplayList, Option<LayoutBox>);
```

`paint` does **not** take fonts — all shaping happens during
layout, glyph rasters and atlas UVs are baked into the `LayoutBox`
tree. Paint just walks `b.text_run.glyphs` and emits quads.

## 4. Fonts on the Tree

> **External fonts are owned by the document, not by a process-
> global resource.** Each `Tree` carries its own `FontRegistry`;
> cascade + shaping consult the registry alone.

This is a hard architectural constraint:

- **No global mutable state** — two `Tree`s can carry different
  font sets without contention.
- **Same lifecycle as the rest of the document** — bytes drop
  when the tree drops.
- **Mirrors how CSS is handled** — inline `<style>` blocks belong
  to the tree; fonts follow the same rule.
- **Trivial to test** — register an embedded font, parse, render,
  assert.

API (`wgpu-html-tree`):

```rust
pub struct Tree {
    pub root:  Option<Node>,
    pub fonts: FontRegistry,
    /* asset_cache_ttl, preload_queue, interaction, … */
}

pub struct FontFace {
    pub family: String,
    pub weight: u16,                  // 100..900, default 400
    pub style:  FontStyleAxis,        // Normal / Italic / Oblique
    pub data:   std::sync::Arc<[u8]>, // shared, cheap to clone
}

impl Tree {
    pub fn register_font(&mut self, face: FontFace) -> FontHandle;
}
```

`FontHandle` is `Ord` so the text crate can pick the
"first registered" handle deterministically when scoring fails.
`FontFace::regular(family, data)` is a 400-weight / Normal-style
shorthand. Re-registering with a fresh `Arc<[u8]>` triggers a
font-db reload at the bridge layer.

The bridge (`wgpu-html-text::FontDb`) lazily caches a
`cosmic_text::FontSystem` keyed by `Arc::as_ptr` identity. Same
registry → no-op resync; replaced `Arc` → reload; removed face →
removed from `fontdb` too. The `FontSystem` is constructed
**without** system fonts (`new_with_locale_and_db("en-US",
Database::new())`) so the registry is the single source of
truth.

## 5. Font matching

`FontRegistry::find_first` walks the comma-separated `font-family`
list left-to-right; for each name, it picks the registered face
that minimises a CSS-Fonts-3-style score:

- **Style band** (× 1,000,000): exact match (0) > italic↔oblique
  interchange (1) > italic-or-oblique target falling to `Normal`
  (2). Ties on the band defer to weight distance.
- **Weight distance**: `|candidate − target|`, plus a `+10,000`
  wrong-direction penalty when a heavier target (`> 500`) gets
  a lighter candidate or vice versa. `[400, 500]` is
  bidirectional. The 10,000 floor keeps any right-direction
  match strictly better than any wrong-direction match.

Ties on the final score break toward the **later-registered**
face — re-registering overrides earlier registrations. If the
whole walk misses, layout falls back to `FontDb::first_handle`
(lowest-numbered loaded handle). An empty registry → text leaves
shape to zero size.

**Generic family fallback — done.** `FontRegistry::find_first`
recognises the CSS-Fonts-4 generic keywords (`sans-serif`,
`serif`, `monospace`, `cursive`, `fantasy`, `system-ui`,
`ui-sans-serif`, `ui-serif`, `ui-monospace`, `ui-rounded`,
`math`, `emoji`, `fangsong`, `-apple-system`, `BlinkMacSystemFont`).
If no name in the family list matches a registered face but any
entry is one of these generics, the search returns the
best-`(weight, style)`-scoring face from the entire registry.
This makes plain `font-family: sans-serif` resolve whatever face
the host registered (e.g. via
`wgpu_html_winit::register_system_fonts(tree, "DemoSans")`)
without requiring an explicit alias.

If the whole walk misses *and* no generic is in the list, layout
still falls back to `FontDb::first_handle` (lowest-numbered loaded
handle) for backward-compat. An empty registry → text leaves
shape to zero size.

## 6. Inline formatting context

A block whose direct children are all inline-level (text or
default-inline elements like `<span> / <strong> / <em> / <i> /
<b> / <u> / <s> / <a> / <code> / <kbd> / <samp> / <var> / <abbr>
/ <cite> / <dfn> / <sub> / <sup> / <time> / <small> / <mark> /
<br> / <wbr> / <bdi> / <bdo> / <ins> / <del> / <label> / <output>
/ <data> / <ruby> / <rt> / <rp>` — or anything with `display:
inline / inline-block / inline-flex`) routes through
`layout_inline_paragraph`.

Pipeline for a multi-span paragraph:

1. **`collect_paragraph_spans(node, plan, ctx)`** walks the
   inline subtree depth-first. Each `Element::Text` becomes a
   `ParagraphSpan` with cascade-resolved family / weight / style /
   size / line-height / colour. Each inline-element wrapper that
   contributed any spans **and** has a `background-color` or
   `text-decoration` is recorded as an `InlineBlockSpan` keeping
   the half-open `[leaf_start, leaf_end)` span-index range and
   the element's resolved colour.
2. **`TextContext::shape_paragraph(spans, container_w)`** builds
   a cosmic-text `Buffer`, calls `set_rich_text` with one
   `Attrs` per span (family / weight / style / metrics / colour
   / metadata = `leaf_id`), shapes it, then walks every
   `layout_run` and packs each glyph into the atlas.
3. **`ParagraphLayout`** is the result:
   - `glyphs: Vec<PositionedGlyph>` — flat, with per-glyph colour
     and y in paragraph-relative coords.
   - `lines: Vec<ParagraphLine>` — `top / baseline / height /
     line_width / glyph_range`. The `glyph_range` is the
     half-open slice into `glyphs` for that line.
   - `leaf_segments: HashMap<u32, Vec<LeafSegment>>` — for each
     `leaf_id`, the contiguous advance ranges that span occupies
     on each line (a wrapped span yields multiple segments).
4. **`layout_inline_paragraph` re-expands** the result:
   - Per-line `text-align` dx via `horizontal_align_offset`.
   - For each `InlineBlockSpan` with a background: one anonymous
     `BoxKind::Block` per `(line × span)` segment, sized to the
     segment's x range and the line's height. `<mark>`'s yellow
     stretches across every line it touches.
   - For each `InlineBlockSpan` with `text_decoration`s: per-line
     decoration quads at `baseline + thickness` (underline),
     `baseline − 0.30 × ascent` (line-through), or `line.top`
     (overline). `<a>`'s underline follows the link across line
     breaks.
   - One single `BoxKind::Text` containing every glyph, with
     each line's `text-align` dx baked in and `g.color` already
     set per-glyph.

**Single-text-leaf fast path.** When the IFC has exactly one
child and it's an `Element::Text`, layout calls
`shape_and_pack(text, font, size, line_h, ls, weight, style,
container_w, color)` directly — same wrapping behaviour, smaller
call graph.

**Block-flow text leaves** (a text node whose parent is laid out
by the non-IFC vertical stacker — rare in practice) shape with
the parent's content width as the wrap budget.

**Mixed inline + block children** fall back to the vertical
block-flow stacker. Anonymous block boxes around runs of inline
content alongside block siblings are not yet generated — see
[caveats](#9-caveats-and-known-gaps).

## 7. Glyph atlas + GPU pipeline

**Atlas** (`crates/wgpu-html-text/src/atlas.rs`):

- Single `R8Unorm` CPU buffer, default 2048×2048 (`pub const
  GLYPH_ATLAS_SIZE`); the renderer creates its GPU texture at the
  same size for 1:1 uploads.
- Shelf packer: glyphs land on horizontal shelves stacked top-to-
  bottom; each shelf locks its height to its first glyph; a
  glyph that doesn't fit horizontally or vertically opens a new
  shelf.
- Dirty-rect list: every `insert` appends a rect;
  `flush_dirty(sink)` drains them into a generic
  `(rect, &[u8])` closure.
- `upload(&Queue, &Texture)` is the wgpu-flavoured wrapper that
  calls `queue.write_texture` for every dirty rect.
- `clear()` zeros the buffer, resets the packer, and queues a
  full-atlas dirty rect so the next flush re-uploads everything.
- Overflow returns `None` from `insert` — no LRU eviction yet,
  no atlas grow / double.

**Glyph cache** (`TextContext::glyph_cache: HashMap<CacheKey,
AtlasGlyph>`): keyed by `(font_handle, glyph_id, size_px,
subpixel_x_bin)`. Misses raster through `cosmic_text::SwashCache`,
pack into the atlas, and store the resulting UVs.

**Pipeline** (`crates/wgpu-html-renderer/src/glyph_pipeline.rs`,
`shaders/glyph.wgsl`):

- Instanced textured quads. The fragment samples the R8 atlas
  and multiplies coverage by the per-instance RGBA colour.
  Premultiplied-alpha blending via
  `wgpu::BlendState::ALPHA_BLENDING`.
- Bind group: globals uniform (binding 0), atlas texture (1),
  filtering sampler (2).
- Instance: `{ pos, size, color, uv_min, uv_max }`. Same unit-quad
  geometry as the quad pipeline.

**Two-pass render** (`Renderer::render`):

1. **Quad pipeline** through the surface's sRGB view,
   `LoadOp::Clear`. Backgrounds, borders, fills are stored as
   sRGB-encoded bytes (the GPU does the linear → sRGB encode on
   write).
2. **Glyph pipeline** (and image pipeline) through a non-sRGB
   `Unorm` view of the same texture, `LoadOp::Load`. The GPU
   treats dst bytes as raw, the alpha blend runs in display
   space, and the shader pre-encodes its own colour to sRGB
   before output. Net result: gamma-correct text blending
   without empirical coverage curves.

Display list:

```rust
pub struct GlyphQuad {
    pub rect:   Rect,
    pub color:  Color,
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
}

pub struct DisplayList {
    pub quads:  Vec<Quad>,
    pub glyphs: Vec<GlyphQuad>,
    pub images: Vec<ImageQuad>,
    pub clips:  Vec<ClipRange>,
}
```

Each shaped glyph becomes its own `GlyphQuad` (no run-level
aggregation). Per-glyph colour is filled from the source span's
`Attrs.color_opt` for rich-text paragraphs, or from
`style.color` for the single-leaf fast path.

## 8. Public API

```rust
// wgpu-html-tree
pub struct FontFace { family, weight, style, data };
pub enum   FontStyleAxis { Normal, Italic, Oblique };
pub struct FontHandle(pub usize);
pub struct FontRegistry { /* faces: Vec<FontFace> */ };
impl Tree {
    pub fn register_font(&mut self, FontFace) -> FontHandle;
}
impl FontRegistry {
    pub fn register(&mut self, FontFace) -> FontHandle;
    pub fn find(&self, family, weight, style) -> Option<FontHandle>;
    pub fn find_first(&self, families, weight, style) -> Option<FontHandle>;
}

// wgpu-html-text
pub struct TextContext { /* font_db, fonts, atlas, swash, glyph_cache */ };
pub struct ShapedRun { glyphs, width, height, ascent };
pub struct PositionedGlyph { x, y, w, h, uv_min, uv_max, color };
pub struct ParagraphSpan { text, family, weight, style, size_px,
                            line_height_px, color, leaf_id };
pub struct ParagraphLayout { glyphs, lines, width, height,
                             first_line_ascent, leaf_segments };
pub struct ParagraphLine { top, baseline, height, line_width, glyph_range };
pub struct LeafSegment { line_index, x_start, x_end };
impl TextContext {
    pub fn new(atlas_size: u32) -> Self;
    pub fn sync_fonts(&mut self, &FontRegistry);
    pub fn pick_font(&self, families, weight, style) -> Option<FontHandle>;
    pub fn shape_and_pack(text, font, size, line_h, ls, weight, style,
                          max_width, color) -> Option<ShapedRun>;
    pub fn shape_paragraph(spans, max_width) -> Option<ParagraphLayout>;
}

// wgpu-html-renderer
pub const GLYPH_ATLAS_SIZE: u32 = 2048;
impl Renderer {
    pub fn glyph_atlas_texture(&self) -> &wgpu::Texture;
}

// wgpu-html-layout
pub fn layout_with_text(tree, &mut TextContext, vw, vh, scale)
    -> Option<LayoutBox>;
pub fn layout(tree, vw, vh) -> Option<LayoutBox>;

// wgpu-html (facade)
pub fn paint_tree_with_text(tree, &mut TextContext, vw, vh, scale)
    -> DisplayList;
pub fn paint_tree(tree, vw, vh) -> DisplayList;
pub fn compute_layout(tree, &mut TextContext, vw, vh, scale)
    -> Option<LayoutBox>;
pub fn paint_tree_returning_layout(tree, &mut TextContext, vw, vh, scale)
    -> (DisplayList, Option<LayoutBox>);
```

## 9. Caveats and known gaps

- **`<br>` forced line break.** Not yet emitted as a synthetic
  newline span in the rich-text pass. Easy fix once we touch the
  `collect_paragraph_spans` walker.
- **`white-space` longhands.** Only `normal` is honoured (whitespace
  collapses pre-shape). `pre / pre-wrap / pre-line / nowrap` would
  flip `collapse_whitespace` and the `Buffer::set_size(width, _)`
  argument (`None` for `nowrap`).
- **`vertical-align: super / sub`.** UA `<sub>/<sup>` parse the
  keyword and apply the smaller font-size, but the baseline raise
  / lower isn't applied. Needs a per-run y-offset on the inline
  pass.
- **Mixed inline + block parents** fall back to vertical stacking.
  Anonymous block boxes around the runs of inline content are not
  generated yet, so a `<div>` with `<p>` siblings interspersed
  with text nodes won't shape those bare text nodes as their own
  IFC.
- **Atlas eviction / grow.** Overflow returns `None` and the glyph
  vanishes. No LRU, no doubling. `Atlas::clear` exists for the
  scale-factor-changed path, but that hook isn't wired in
  (`TextContext::glyph_cache` still holds stale `AtlasEntry`s
  after a clear — needs invalidation alongside the atlas reset).
- **Letter-spacing in the rich-text path.** Single-leaf shaping
  applies it post-shape; the rich-text path skips it because
  cosmic-text 0.12's `Attrs` doesn't expose the field. Same
  per-glyph cumulative offset would work here too.
- **`text-decoration-color / -thickness / -style`.** Decorations
  always paint in the inline element's `color`, solid, with
  thickness scaled to the line ascent.
- **`font-stretch`, `font-variant`, `font-feature-settings`,
  `text-shadow`.** Not parsed, not resolved.
- **Bidi / vertical writing modes.** Not modelled.
- **Subpixel antialiasing.** Straight alpha mask only — sufficient
  for typical 16 px body text on Retina-class displays.
- ~~**Generic family fallback** — done.~~ See §5: generic CSS
  keywords now trigger a fall-back to any registered face,
  ranked by `(weight, style)`. Hosts no longer have to register
  their preferred face under the generic name explicitly.
- **`@font-face` from CSS.** Out of scope. The natural extension
  would parse `src: url(...)` into a synthetic `register_font`
  call against a host-supplied resolver — not wired up.
- **Hot font swaps.** Re-registering with a new `Arc<[u8]>` flips
  the bridge identity check and reloads the face into fontdb, but
  cosmic-text's internal shape cache still holds stale shapes
  for that face.
- **Number-only `line-height`** (e.g. `line-height: 1.5`). Not
  parsed yet — `Px / Em / Rem` only.

## 10. Tests

- `wgpu-html-tree::fonts::tests` — 11 unit tests covering
  register / lookup / case-insensitive family matching / weight
  bias / italic↔oblique swap / multi-family fallback / empty
  registry.
- `wgpu-html-text::atlas::tests` (11) and
  `wgpu-html-text::font_db::tests` (5) cover the shelf packer +
  the cosmic-text bridge identity check.
- `wgpu-html-layout::tests` (89) covers layout including text
  leaves; the no-font compatibility wrapper (`layout(...)`) keeps
  the older fixtures green.
- `wgpu-html::paint::tests` (23) covers the painter end-to-end
  for shaped text in single-leaf and rich-text paths.
- The demo (`crates/wgpu-html-demo`) is the visual end-to-end
  test: `hello-text.html`, `flex-grow.html`, `overflow.html`,
  `gif.html`, etc., rebuilt every frame against a candidate-paths
  table of system fonts.

## 11. Coverage Checklist (All Cases)

This section is the "done means done" matrix for text. Every item is
either already covered by tests, or explicitly marked as a gap.

### 11.1 Parsing + Cascade

- `font-family`: comma-separated list, quoted family names, whitespace
  trimming, unknown family fallback to next candidate.
- `font-size`: `px / em / rem / % / vw / vh / vmin / vmax`, plus
  `calc()` and nested `min()/max()/clamp()`.
- `font-weight`: keyword + numeric forms, including out-of-family
  fallback to closest registered weight.
- `font-style`: `normal / italic / oblique`, with italic↔oblique
  interchange.
- `line-height`: lengths; default `1.25 × font-size` when unset.
- `letter-spacing`: zero, positive, negative lengths.
- `text-transform`: `none / uppercase / lowercase / capitalize`.
- `text-decoration`: token list combinations and `none` reset.
- CSS-wide keywords on inherited text props: `inherit`, `initial`,
  `unset`.

### 11.2 Font Registry + Selection

- Empty registry: text leaves produce zero-sized runs without panics.
- Single family with multiple faces: weight/style scoring stability.
- Multi-family fallback: first available match in declaration order.
- Tie-break behavior: later registration wins on equal score.
- Registry resync behavior:
  - unchanged `Arc<[u8]>` => no-op
  - replaced `Arc<[u8]>` => reload
  - removed face => purge from bridge db

### 11.3 Shaping + Wrapping

- Single-leaf shaping with finite width wraps to multiple lines.
- Rich-text paragraph shaping wraps across inline-element boundaries.
- Glyph ordering and line metrics:
  - `ParagraphLine.glyph_range` must index contiguous glyph slices
  - line top/baseline/height must be monotonic and non-negative.
- Whitespace collapse for `white-space: normal`.
- Long-word behavior when no legal break point exists.

### 11.4 Inline Layout + Paint Expansion

- Per-line `text-align` offsets for `left/right/center/start/end`.
- Inline wrapper backgrounds (`<mark>`) expand across each wrapped
  line segment they touch.
- Decorations (`underline/line-through/overline`) emit per-line bars
  with expected y-position and thickness.
- Mixed-style runs preserve per-glyph foreground color.
- Clip interactions:
  - text respects ancestor clip ranges
  - rounded clips apply on overflow-hidden containers.

### 11.5 Renderer + Atlas

- Glyph atlas upload path updates only dirty rects.
- Atlas overflow behavior is deterministic (`insert => None`).
- Gamma-correct composition invariant:
  - quads render first on sRGB view
  - glyphs/images render with load on non-sRGB view.
- Clip-range/scissor partitioning:
  - each clip range maps to correct glyph/image/quad instance ranges
  - empty ranges are dropped by `DisplayList::finalize()`.

### 11.6 Regression Fixtures To Keep

- Mixed-emphasis sentence with wraps across `<strong>/<em>/<a>/<mark>`.
- Dense punctuation and multiple spaces/newlines (whitespace collapse).
- Extreme letter spacing (large positive + negative).
- Long CJK-like token / no-space Latin token.
- Nested overflow clipping around text.
- Font fallback chain where first family is missing.
- Generic-family fallback: a tree with `sans-serif` only and a
  registry under `Inter` resolves; same with `Garamond, sans-serif`.

### 11.7 Known Uncovered (Must Stay Explicit)

The following remain intentionally uncovered until implemented:

- `<br>` forced line break in rich-text paragraph path.
- `white-space: pre / pre-wrap / pre-line / nowrap`.
- `vertical-align: sub/super` baseline shifts.
- `letter-spacing` in rich-text shaping path.
- Bidi + RTL text shaping/layout.
- Number-only `line-height` (unitless).

### 11.8 Form-field placeholder rendering — done

`compute_placeholder_run` (in `wgpu_html_layout::lib`) shapes the
`placeholder` attribute on empty `<input>` and `<textarea>`
elements and attaches the result as the box's `text_run`,
painted at the cascaded `color × alpha 0.5` (the browser default
`::placeholder` styling). Wired into both `layout_block` and
`layout_atomic_inline_subtree` (form controls hit the inline-block
path most often).

Behavioural rules covered by tests in
`wgpu-html-layout::tests::*placeholder*`:

- Empty `<input>` with `placeholder="…"` → text_run + color set.
- Non-empty `value="…"` suppresses placeholder.
- Bare `<input>` (no placeholder) → no text_run.
- `<input type="hidden">` with `placeholder="…"` → no text_run.
- `placeholder=""` (empty) → no text_run.
- Empty `<textarea>` with `placeholder="…"` → text_run + color set.
- `<textarea>` with children (RAWTEXT content) → no placeholder.
- Single-line input: glyphs whose right edge crosses
  `content_rect.w` are dropped (horizontal clip); remaining run
  vertically centred inside `content_rect`.
- Textarea: soft-wrap at `content_rect.w` (white-space: pre-wrap
  from UA), top-aligned.
- Cascaded `color: red` → placeholder colour = linear-red × alpha 0.5.
- User CSS `padding: 8px 10px` overrides UA padding-block /
  padding-inline; placeholder origin tracks the new content_rect.
