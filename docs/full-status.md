# wgpu-html — Complete Project Status

> **Date:** 2026-04-27
> **Scope:** GPU-accelerated HTML/CSS renderer via `wgpu`. **No JavaScript — ever.**

---

## Architecture

```
HTML/CSS string
   │
   ▼  wgpu-html-parser           Tokenizer + tree builder + CSS parser
Tree<Node<Element>>
   │
   ▼  wgpu-html-style            UA defaults + selector match + cascade + inheritance
CascadedTree<CascadedNode>
   │
   ▼  wgpu-html-layout           Block flow + Flex + Grid + Inline (IFC) + text shaping
LayoutBox tree
   │
   ▼  wgpu-html (paint.rs)       LayoutBox → DisplayList (quads + glyphs + clip ranges)
DisplayList
   │
   ▼  wgpu-html-renderer         Quad pipeline (SDF) + Glyph pipeline + scissor/clip
Frame on wgpu surface
```

### Crate map (9 crates)

| Crate | Role |
|---|---|
| `wgpu-html-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `wgpu-html-models` | `Style` struct (~80+ fields), CSS enums, ~100 HTML element structs |
| `wgpu-html-tree` | `Tree` / `Node` / `Element`, font registration, event callbacks, interaction state |
| `wgpu-html-style` | Cascade engine: UA stylesheet, selector matching, field merge, CSS-wide keywords, inheritance |
| `wgpu-html-text` | Font database, text shaping (rustybuzz), glyph atlas (rasterisation + GPU upload) |
| `wgpu-html-layout` | Block flow, Flexbox, Grid, inline formatting context, hit testing |
| `wgpu-html-renderer` | wgpu device/surface, quad pipeline (SDF shader), glyph pipeline, scissor clipping, screenshot |
| `wgpu-html` | Façade: `parse → cascade → layout → paint`, interactivity module |
| `wgpu-html-demo` | winit window, font loading, mouse/keyboard events, continuous redraw loop |

---

## ✅ What Is Done

### 1. HTML Parsing
- **Tokenizer:** open/close/self-closing tags, quoted/unquoted/boolean attributes, comments, DOCTYPE, raw-text elements (`<style>`, `<script>`, `<textarea>`, `<title>`).
- **Entity decoding:** `&amp; &lt; &gt; &quot; &apos; &nbsp; &#NN; &#xNN;`.
- **Tree builder:** 14-entry void list, self-closing recognition, auto-close rules for `p / li / dt / dd / thead / tbody / tfoot / tr / th / td / option / optgroup / rt / rp`, EOF auto-close, synthetic `<body>` wrap for multiple top-level nodes.
- **~100 element variants** with per-element attribute parsing (`<a>`, `<img>`, `<input>`, `<form>`, etc.).
- **Global attributes:** `id, class, style, title, lang, dir, hidden, tabindex, accesskey, contenteditable, draggable, spellcheck, translate, role`.
- **`aria-*` / `data-*`** captured into `HashMap<String, String>`.

### 2. CSS Parsing
- **Box model:** `display, position, top/right/bottom/left, width, height, min-/max-width/height, box-sizing`.
- **Spacing:** `margin`, `padding` — 1/2/3/4-value shorthand + per-side longhands.
- **Backgrounds:** `background-color, background-clip, background-repeat`; `background-image / -size / -position` stored as raw strings.
- **Borders:** `border` shorthand; per-side shorthand + `-width / -style / -color` longhands; `border-radius` with `/`-separated elliptical syntax, 1–4-corner expansion, per-corner `<h> <v>` longhands.
- **Typography:** `color, font-family, font-size, font-weight, font-style, line-height, letter-spacing, text-align, text-transform, white-space, text-decoration, vertical-align`.
- **Overflow:** `overflow, overflow-x, overflow-y`.
- **Misc:** `opacity, visibility, z-index`.
- **Flexbox:** `flex-direction, flex-wrap, justify-content, align-items, align-content, align-self, gap, row-gap, column-gap, flex, flex-grow, flex-shrink, flex-basis, order`.
- **Grid:** `display: grid`, `grid-template-columns, grid-template-rows, grid-auto-columns, grid-auto-rows, grid-auto-flow, grid-column-start/end, grid-row-start/end, grid-column, grid-row, justify-items, justify-self, align-items, align-self, align-content, justify-content, gap, row-gap, column-gap`.
- **CSS-wide keywords:** `inherit`, `initial`, `unset` on any property.
- **`!important`** declarations (parsed and respected in cascade ordering).
- **Length units:** `px, %, em, rem, vw, vh, vmin, vmax, auto, 0` + raw fallback.
- **Colors:** ~20 named colors, `#rgb / #rgba / #rrggbb / #rrggbbaa`, `rgb(), rgba(), hsl(), hsla(), transparent, currentcolor`.
- **Stylesheet parser:** flat `selectors { decls }` with `/* */` comment stripping.
- **Selectors:** tag, `#id`, `.class`, universal `*`, comma-list, **descendant combinator** (` `).
- **Specificity:** packed `(id<<16) | (class<<8) | tag`.

### 3. Style Cascade
- **UA default stylesheet** — `display: none` for `<head>/<style>/<script>/…`, `body { margin: 8px }`, heading sizes/weights (`h1`–`h6`), block-level margins (`p, ul, ol, dl, blockquote, …`), inline emphasis (`b, strong, em, i, u, s, code, a, mark, small, sub, sup`).
- **Cascade order:** UA rules → author rules in ascending specificity (stable on ties) → inline `style=""` on top.
- **`!important`** respected in cascade band ordering.
- **Inheritance:** `color, font-family, font-size, font-weight, font-style, line-height, letter-spacing, text-align, text-transform, white-space, text-decoration, visibility` propagate from parent to child.
- **CSS-wide keywords:** `inherit` / `initial` / `unset` resolved per-property.
- **Selector matching:** tag, `#id`, multi-`.class`, `*`, descendant combinator.
- **Field-by-field "Some-wins" merge** across all 80+ Style fields.

### 4. Text Rendering
- **Font database** (`wgpu-html-text/font_db.rs`): register `.ttf` / `.otf` / `.ttc` font files; family + weight + style matching.
- **Font registration on Tree** (`wgpu-html-tree/fonts.rs`): `FontFace` struct, `Tree::register_font()`, family/weight/style axis.
- **Text shaping** (`wgpu-html-text/shape.rs`): `rustybuzz`-based shaping, `font-family` list fallback, `font-weight`, `font-style: italic`, `font-size`, `letter-spacing`, `text-transform` (uppercase / lowercase / capitalize), `white-space: pre` vs collapse.
- **Glyph atlas** (`wgpu-html-text/atlas.rs`): CPU rasterisation → GPU texture upload, shelf-packing allocator.
- **Glyph pipeline** (`wgpu-html-renderer/glyph_pipeline.rs`): dedicated WGSL shader (`glyph.wgsl`), per-glyph instanced quads, alpha-tested coverage.
- **Per-glyph text color** — resolved from `color` property, inherited through cascade.
- **Text decorations:** `underline`, `line-through`, `overline` — painted as solid quads at correct vertical offsets.
- **`text-align`:** `left`, `right`, `center`, `justify` (horizontal alignment of line boxes).

### 5. Layout Engine

#### Block Flow
- Vertical stacking inside parent's content box.
- `margin / padding` per side, shorthand fallback.
- `box-sizing: content-box | border-box`.
- `width`: explicit (px / % / em / rem / vw / vh / vmin / vmax) or fills container.
- `height`: explicit or sum of children.
- `min-width / max-width / min-height / max-height` — clamped.
- `auto` margin centering (horizontal).
- Border widths, colors, styles carried into `LayoutBox`.
- `border-radius`: per-corner H+V, CSS-3-spec corner-overflow clamping.
- `background-clip: border-box | padding-box | content-box` with concentric inner-radius reduction.

#### Flexbox (complete CSS Flexbox Level 1)
- `flex-direction`: row / column / row-reverse / column-reverse.
- `flex-wrap`: nowrap / wrap / wrap-reverse — **multi-line wrapping**.
- `justify-content`: start / end / center / flex-start / flex-end / left / right / space-between / space-around / space-evenly.
- `align-items`: start / end / center / flex-start / flex-end / stretch / baseline (falls to start).
- `align-content`: start / end / center / flex-start / flex-end / stretch / space-between / space-around / space-evenly.
- `align-self`: per-item override.
- `flex-grow / flex-shrink / flex-basis` — **fully functional** (two-pass grow/shrink distribution).
- `gap` (main-axis + cross-axis).
- `order` property.

#### CSS Grid
- `grid-template-columns / grid-template-rows` with `px`, `fr`, `auto`, `minmax()`, `repeat()`.
- `grid-auto-columns / grid-auto-rows`.
- `grid-auto-flow`: row / column / dense.
- Item placement: `grid-column-start/end`, `grid-row-start/end`, `span N`, line numbers.
- Auto-placement algorithm (row-major and column-major).
- Track sizing: definite lengths, `fr` distribution, intrinsic (`auto`) sizing via content measurement.
- `justify-items / justify-self / align-items / align-self` (start / center / end / stretch).
- `justify-content / align-content` (start / center / end / stretch / space-between / space-around / space-evenly).
- `gap / row-gap / column-gap`.

#### Inline Formatting Context (IFC)
- Line-box layout for mixed text + inline-block children.
- Word wrapping at container width.
- `text-align` applied per line box.
- Inline-block elements participate in line flow.
- Anonymous text runs shaped and positioned.

#### Hit Testing
- `LayoutBox::hit_path((x, y)) → Option<Vec<usize>>`.
- `find_element_from_point` / `find_elements_from_point` — deepest-first, topmost (last-painted) wins.

### 6. Rendering / Paint

#### GPU Pipelines
- **Quad pipeline** (`quad.wgsl`): instanced quads with SDF for rounded fill, stroked ring, dashed/dotted patterned ring. Per-quad: rect, color, per-corner H+V radii, per-side stroke widths, pattern descriptor.
- **Glyph pipeline** (`glyph.wgsl`): instanced glyph quads, alpha-tested from atlas texture, per-glyph color tint.

#### Paint Translation
- **Backgrounds:** solid color into `background-clip`-driven rect with elliptical corner radii. sRGB → linear conversion.
- **Borders (sharp):** per-side edge quads with per-side color/style. `solid` → one quad; `dashed`/`dotted` → segment loop; `none/hidden` → skipped; `double/groove/ridge/inset/outset` → fall through to solid.
- **Borders (rounded, uniform solid):** single SDF ring quad.
- **Borders (rounded, mixed):** per-side one-sided ring quads; patterned ring for dashed/dotted with uniform-circular corners.
- **Text:** glyph quads from shaped runs, decoration quads (underline/overline/line-through).
- **Overflow clipping:** `overflow: hidden / scroll / auto` clips children to padding-box rect via scissor rects. Rounded clipping uses SDF discard for `border-radius` + `overflow: hidden`. Clip stack with intersection for nested clips.

#### Renderer Infrastructure
- wgpu instance / adapter / device / queue, sRGB surface, vsync.
- Single render pass: clear → quads → glyphs (with scissor ranges).
- F12 screenshot → PNG export.

### 7. Interactivity & Demo
- **Mouse input:** `CursorMoved`, `MouseInput` (press/release), `CursorLeft` — all wired.
- **Event system:** `pointer_move`, `pointer_leave`, `mouse_down`, `mouse_up` with hit-test → event dispatch.
- **Event bubbling:** events bubble up the ancestor chain.
- **Hover tracking:** `InteractionState` tracks `hover_path` and `active_path`.
- **Callback slots on Node:** `on_click`, `on_mouse_down`, `on_mouse_up`, `on_mouse_enter`, `on_mouse_leave` — `Arc<dyn Fn(&MouseEvent)>`.
- **MouseEvent:** carries `pos`, `button`, `modifiers` (shift/ctrl/alt/meta), `target_path`, `current_path`.
- **Keyboard:** F12 (screenshot), Esc (exit).
- **Font loading:** platform-aware font discovery (macOS `.ttc`, Windows/Linux `.ttf`), multiple weights/styles registered.
- **Continuous redraw loop** via `request_redraw` in `about_to_wait`.

---

## ❌ What Is NOT Done

### HTML Parsing Gaps
- Comments + DOCTYPE tokenized then **dropped** at tree-build time.
- Unknown tags drop their **entire subtree** silently.
- No HTML5 insertion-mode state machine (no `<table>` foster-parenting, no `</br>` → `<br>` quirk).
- No `<![CDATA[]]>`, no foreign content (SVG / MathML inner nodes).
- Whitespace-only text between tags is dropped.

### CSS Parsing Gaps
- **No at-rules:** `@media, @supports, @import, @keyframes, @font-face, @page` — not handled.
- **No child/sibling combinators:** `>`, `+`, `~` are not supported (only descendant ` ` works).
- **No attribute selectors** (`[href]`, `[type=text]`).
- **No pseudo-classes / pseudo-elements** (`:hover, :focus, :nth-child, ::before, ::after`, …).
- `transform, transition, animation, box-shadow, background-image, background-size, background-position` stored as **raw `Option<String>`** — never structured or applied.
- No `calc()`, no `var(…)`, no custom properties (`--foo`).
- No structured types for shadows, gradients, transforms, filters, masks, clip-paths.

### Style / Cascade Gaps
- `<link rel="stylesheet">` not loaded — only inline `<style>` blocks.
- `currentColor` resolves to `None` (no foreground-color fallback for borders).
- No pseudo-class state integration (`:hover` / `:focus` / `:active` selectors don't exist).

### Layout Gaps
- **No positioned layout:** `position` and `top/right/bottom/left` parsed but never consumed.
- **No `z-index`** — paint order is tree DFS only.
- **No floats** (`float: left/right`).
- **No table layout** (`display: table` and friends).
- **No `display: inline / inline-block`** as an author-set value (IFC is auto-detected from content, not from `display`).
- `em / rem` use a hard-coded 16px when no font-size is inherited (no full font cascade for unit resolution).
- No baseline alignment in flex.
- Transforms not applied to layout.

### Rendering Gaps
- **No images:** `<img src>` parsed but never fetched or drawn. No `background-image`.
- **No gradients:** `linear-gradient(…)` stays a raw string.
- **No box-shadow.**
- **No transforms / opacity layers / filters / blend modes** (per-quad alpha only).
- Border styles `double / groove / ridge / inset / outset` render as plain solid.
- Dashed/dotted on rounded boxes only follow the curve when all four corners are uniform-circular; otherwise corners stay bare.
- A border without an explicit color is skipped (no fallback to `color` / `currentColor`).
- No multi-pass compositing (no stencil buffer usage beyond scissor).

### Interactivity Gaps
- **No keyboard input** for text fields — `<input>` and `<textarea>` are inert.
- **No scrolling** — `overflow: scroll/auto` clips but has no scroll position or scroll bars.
- **No focus management** / tab navigation.
- **No cursor styling** (`cursor` property parsed but not applied).
- No event `preventDefault` / `stopPropagation` semantics.
- The document is a compile-time constant — no URL loading, no live editing, no hot reload.

### Explicitly Out of Scope (Forever)
- **No JavaScript.** No `<script>` execution, no JS engine, no scripting hooks, no `eval`, no `addEventListener`-style JS callbacks.

---

## Summary Table

| Feature Area | Status |
|---|---|
| HTML tokenizer + tree builder | ✅ Done (no HTML5 state machine) |
| CSS declaration parsing (~40 properties) | ✅ Done |
| CSS stylesheet / selectors | ✅ Done (tag, id, class, *, descendant) |
| `!important` | ✅ Done |
| CSS-wide keywords (inherit/initial/unset) | ✅ Done |
| UA default stylesheet | ✅ Done (headings, body margin, inline emphasis, display:none) |
| Style inheritance | ✅ Done (color, font-*, text-*, line-height, visibility, etc.) |
| Block flow layout | ✅ Done |
| Flexbox (Level 1 complete) | ✅ Done |
| CSS Grid | ✅ Done |
| Inline formatting context | ✅ Done |
| Text shaping + rendering | ✅ Done (rustybuzz + glyph atlas + GPU pipeline) |
| Text decorations | ✅ Done (underline, line-through, overline) |
| Text alignment | ✅ Done (left, right, center, justify) |
| letter-spacing / text-transform | ✅ Done |
| min/max width/height clamping | ✅ Done |
| auto margin centering | ✅ Done |
| Overflow clipping (hidden/scroll/auto) | ✅ Done (rectangular + rounded SDF) |
| Borders (solid, dashed, dotted, rounded) | ✅ Done |
| background-clip | ✅ Done |
| Hit testing | ✅ Done |
| Mouse events + bubbling | ✅ Done |
| Hover / active tracking | ✅ Done |
| Screenshot (F12 → PNG) | ✅ Done |
| Positioned layout (absolute/relative/fixed) | ❌ Not done |
| z-index | ❌ Not done |
| Floats | ❌ Not done |
| Table layout | ❌ Not done |
| Images (`<img>`, `background-image`) | ❌ Not done |
| Gradients | ❌ Not done |
| box-shadow | ❌ Not done |
| Transforms / transitions / animations | ❌ Not done |
| Opacity layers / filters / blend modes | ❌ Not done |
| Scrolling (scroll position + scroll bars) | ❌ Not done |
| Keyboard input / text editing | ❌ Not done |
| Focus management | ❌ Not done |
| Pseudo-classes / pseudo-elements | ❌ Not done |
| Child/sibling combinators (`>`, `+`, `~`) | ❌ Not done |
| Attribute selectors | ❌ Not done |
| At-rules (@media, @keyframes, @font-face…) | ❌ Not done |
| calc() / var() / custom properties | ❌ Not done |
| `<link>` stylesheet loading | ❌ Not done |
| JavaScript | 🚫 Permanently out of scope |
