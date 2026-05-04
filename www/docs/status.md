---
title: Implementation Status
---

# Implementation Status

> **Date:** 2026-05-04
> **Scope:** GPU-accelerated HTML/CSS renderer via `wgpu`. **No JavaScript — ever.**

## Summary Table

| Feature Area | Status |
|---|---|
| HTML tokenizer + tree builder | ✅ Done (no HTML5 state machine) |
| CSS declaration parsing (~80+ properties) | ✅ Done |
| CSS stylesheet / selectors | ✅ Done (tag, id, class, \*, descendant) |
| `!important` | ✅ Done |
| CSS-wide keywords (inherit/initial/unset) | ✅ Done |
| `@media` queries | ✅ Done (width/height/orientation, min/max, not) |
| UA default stylesheet | ✅ Done (headings, body margin, inline emphasis, display:none) |
| Style inheritance | ✅ Done (color, font-\*, text-\*, line-height, visibility, etc.) |
| `:hover` / `:active` / `:focus` cascade integration | ✅ Done |
| Block flow layout | ✅ Done |
| Flexbox (Level 1 complete) | ✅ Done |
| CSS Grid | ✅ Done |
| Inline formatting context | ✅ Done |
| Text shaping + rendering | ✅ Done (cosmic-text + glyph atlas + GPU pipeline) |
| Text decorations | ✅ Done (underline, line-through, overline) |
| Text alignment | ✅ Done (left, right, center, justify) |
| letter-spacing / text-transform | ✅ Done |
| min/max width/height clamping | ✅ Done |
| auto margin centering | ✅ Done |
| Overflow clipping (hidden/scroll/auto) | ✅ Done (rectangular + rounded SDF) |
| Borders (solid, dashed, dotted, rounded) | ✅ Done |
| background-clip | ✅ Done |
| Images (`<img>`, `background-image`) | ✅ Done (HTTP(S)/file/data-URI, GIF/WebP animation, cache) |
| Hit testing | ✅ Done |
| `pointer-events: none` hit-test skip | ✅ Done |
| `user-select: none` enforcement | ✅ Done |
| Mouse events + bubbling | ✅ Done (typed `HtmlEvent` + legacy `MouseEvent` slots) |
| Hover / active tracking | ✅ Done |
| Text selection (drag-select, Ctrl+A, Ctrl+C) | ✅ Done (anchor/focus cursors, `arboard` clipboard) |
| Viewport scrollbar + scroll position | ✅ Done (paint + drag; wheel scroll) |
| Per-element scroll containers | ⚠️ Partial (scroll offset + scrollbar paint; no `Wheel`→`on_event` dispatch) |
| Screenshot (F12 → PNG) | ✅ Done |
| Positioned layout (absolute/relative/fixed) | ✅ Done (sticky degraded to relative) |
| z-index | ⚠️ Partial (parsed, stored on LayoutBox, sorts siblings by z-index; no stacking contexts — cross-branch ordering still tree DFS) |
| Floats | ❌ Not done |
| Table layout | ❌ Not done (parsed, falls through to block) |
| display: inline/inline-block (author-set) | ❌ Not done (IFC auto-detected from content) |
| Gradients | ❌ Not done (parsed as raw string, skipped in layout) |
| box-shadow | ❌ Not done (parsed as raw string, not consumed) |
| Transforms / transitions / animations | ❌ Not done (parsed as raw string, not consumed) |
| Opacity | ✅ Done |
| Filter / blend modes | ❌ Not done |
| Keyboard event dispatch | ✅ Done |
| Tab / Shift+Tab focus traversal | ✅ Done |
| Focus state | ✅ Done |
| `querySelector` / `matches` engine | ✅ Done (full CSS Level 4: all combinators, attribute operators, pseudo-classes) |
| Placeholder rendering | ✅ Done |
| Text editing + caret navigation | ✅ Done (insert, delete, arrow keys, Home/End, Shift-select, word/line click-select) |
| Checkbox / radio click toggle | ✅ Done (checkboxes, radios, click-to-toggle) |
| `<select>` dropdown, form submission | ❌ Not done |
| `:focus-visible` / `:focus-within` / `:disabled` in cascade | ❌ Not done (query engine only) |
| Pseudo-elements | ❌ Not done |
| calc() / min() / max() / clamp() | ✅ Done (full AST + evaluation) |
| var() / custom properties (`--foo`) | ✅ Done (parsed, inherited, recursive substitution, cycle detection) |
| `<link>` stylesheet loading | ❌ Not done |
| Devtools | ⚠️ Partial (component tree browser, styles inspector, breadcrumb bar) |
| Component framework (`wgpu-html-ui`) | ✅ Done (Elm architecture, reactive Store, render caching, scoped CSS) |
| Profiler | ✅ Done (ring-buffer frame history, scopes, counters, summary_string, trace export planned) |
| JavaScript | 🚫 Permanently out of scope |

## ✅ What Is Done

### HTML Parsing

- **Tokenizer:** open/close/self-closing tags, quoted/unquoted/boolean attributes, comments, DOCTYPE, raw-text elements (`<style>`, `<script>`, `<textarea>`, `<title>`)
- **Entity decoding:** `&amp; &lt; &gt; &quot; &apos; &nbsp; &#NN; &#xNN;`
- **Tree builder:** 14-entry void list, self-closing recognition, auto-close rules for `p / li / dt / dd / thead / tbody / tfoot / tr / th / td / option / optgroup / rt / rp`, EOF auto-close, synthetic `<body>` wrap
- **~100 element variants** with per-element attribute parsing
- **Global attributes:** `id, class, style, title, lang, dir, hidden, tabindex, accesskey, contenteditable, draggable, spellcheck, translate, role`
- **`aria-*` / `data-*`** captured into `HashMap<String, String>`
- **DOM-style element lookup:** `get_element_by_id()`, `get_element_by_class_name()`, `get_elements_by_class_name()`, `query_selector()`, `query_selector_all()`
- **HTML serialisation:** `Node::to_html()`, full document and per-node

### CSS Parsing

- **Box model:** `display, position, top/right/bottom/left, width, height, min-/max-width/height, box-sizing`
- **Spacing:** `margin`, `padding` with 1–4-value shorthand + per-side longhands
- **Backgrounds:** `background-color, background-clip, background-repeat`, `background-image`, `background-size/-position`
- **Borders:** `border` shorthand, per-side `-width/-style/-color` longhands, `border-radius` with elliptical syntax
- **Typography:** `color, font-family, font-size, font-weight, font-style, line-height, letter-spacing, text-align, text-transform, white-space, text-decoration, vertical-align`
- **Overflow:** `overflow, overflow-x, overflow-y`
- **Flexbox:** full property set (direction, wrap, justify-content, align-items, align-content, align-self, gap, flex, flex-grow/shrink/basis, order)
- **Grid:** full property set (template-columns/rows, auto-columns/rows, auto-flow, column/row-start/end, column/row, justify/align-items/self/content, gap)
- **CSS-wide keywords, `!important`, length units, colors, selectors**
- **`@media` queries:** width/height/orientation with min/max prefix + `not`, evaluated during cascade
- **Math functions:** `calc()`, `min()`, `max()`, `clamp()` — fully parsed and evaluated at layout time
- **CSS variables:** `var()` + custom properties (`--foo`) — parsed, inherited, recursive substitution with cycle detection

### Style Cascade

- **UA default stylesheet** — `display: none` for `<head>/<style>/<script>/…`, `body { margin: 8px }`, heading sizes/weights, block-level margins, inline emphasis
- **Cascade order:** UA rules → author rules in ascending specificity → inline `style=""` on top
- **`!important`** respected in cascade band ordering
- **Inheritance:** `color, font-family, font-size, font-weight, font-style, line-height, letter-spacing, text-align, text-transform, white-space, text-decoration, visibility`
- **CSS-wide keywords:** `inherit` / `initial` / `unset` resolved per-property
- **Selector matching:** tag, `#id`, multi-`.class`, `*`, descendant combinator
- **Dynamic pseudo-classes:** `:hover`, `:active`, `:focus` via `MatchContext::for_path`

### Layout Engine

- **Block Flow:** Vertical stacking, margin/padding, `box-sizing`, explicit/percentage lengths, min/max clamping, auto margin centering, border widths/styles/colors, `border-radius`, `background-clip`
- **Flexbox:** Complete CSS Flexbox Level 1 — direction, wrap, justify-content, align-items/self/content, flex-grow/shrink/basis, gap, order, multi-line wrapping, content-based min-size
- **CSS Grid:** `grid-template-columns/rows` with `px`, `fr`, `auto`, `minmax()`, `repeat()`, line+span placement, auto-placement, track sizing, justify/align-items/self/content, gap
- **Inline Formatting Context:** Line-box layout, word wrapping, `text-align` per line box, inline-block elements, anonymous text runs
- **Positioned layout:** `absolute`, `relative`, `fixed` with containing blocks, insets, shrink-to-fit, right/bottom anchoring; `sticky` degraded to relative
- **Hit Testing:** `LayoutBox::hit_path((x, y))` → deepest-first, topmost wins; `pointer-events: none` skip

### Text Rendering

- **Font database:** Register `.ttf/.otf/.ttc` files; family + weight + style matching; generic-family fallback
- **Text shaping:** HarfBuzz via cosmic-text; `font-family` fallback, `font-weight`, `font-style: italic`, `font-size`, `letter-spacing`, `text-transform`, `white-space`
- **Glyph atlas:** CPU rasterisation → GPU texture upload, shelf-packing allocator
- **Text decorations:** `underline`, `line-through`, `overline` as solid quads at correct vertical offsets
- **Per-glyph color** resolved from `color` property through cascade

### Rendering / Paint

- **Quad pipeline:** Instanced quads with SDF for rounded fill and stroked rings; solid/dashed/dotted borders; patterned ring shader for dashed/dotted on rounded boxes
- **Glyph pipeline:** Instanced glyph quads, alpha-tested from atlas texture, per-glyph color
- **Image pipeline:** Textured quads for `<img>` and `background-image`; animated GIF/WebP frame selection
- **Overflow clipping:** Rectangular scissor + SDF rounded clipping; nested clip intersection; `overflow-x`/`overflow-y` axis independence
- **Opacity:** inherited multiplicatively, baked into color alpha
- **Screenshot:** F12 → PNG export; programmatic `capture_to()`/`capture_rect_to()`/`screenshot_node_to()`

### Interactivity

- **Pointer:** Mouse input fully wired through typed `HtmlEvent` structs; bubbling (mousedown/up/click bubble; mouseenter/mouseleave don't); hover/active tracking with `InteractionState`; `pointer-events: none` skip
- **Focus + keyboard:** Focusable predicates for `<button>`, `<a href>`, `<input>`, `<textarea>`, `<select>`, `tabindex`; Tab/Shift+Tab navigation; modifier tracking; `keydown`/`keyup` dispatch
- **Form fields:** Placeholder rendering for `<input>` and `<textarea>`; full text editing with insert/delete/backspace/arrow keys/Home/End/Shift-select/click caret/multibyte UTF-8; readonly support
- **Checkbox/radio:** Click toggles `checked`, fires `change` and `input` events; radio mutual exclusion
- **Text selection:** `TextCursor`/`TextSelection`, drag-to-select, `Ctrl+A`/`Ctrl+C`, `user-select: none`
- **Scrolling:** Viewport and per-element scrollbar paint + drag; `MouseWheel` scrolling; per-element scroll offsets tracked in `scroll_offsets_y`

### Profiler

- **Ring-buffer frame profiler:** Fixed-capacity (240 frames, ≈ 4s) history of per-frame spans, counters, and events
- **Scope-based instrumentation:** RAII guards with parent-stack tracking for nested spans (e.g. `cascade` → `selector` → `merge`)
- **`prof_scope!` macro:** Zero-cost when disabled — expands to `Option::map` that the compiler eliminates
- **Counters:** Per-frame scalars (`nodes`, `quads`, `glyphs`, `layout_boxes`, etc.) emitted alongside spans
- **`summary_string()`:** Human-readable indented text block with proportional bar chart and counters line
- **Alert threshold:** Silent-until-slow mode — only reports frames exceeding a configurable threshold
- **`PipelineTimings`:** `{cascade_ms, layout_ms, paint_ms}` struct always available, no profiler required
- **`PipelineCache`:** Three-level frame classification (`FullPipeline` / `PartialCascade` / `RepaintOnly`) with incremental re-cascade and paint-only pseudo-class fast path
- **Backward compat:** `clear()` / `record()` / `flush()` shims preserved

### Component Framework (`wgpu-html-ui`)

- **Elm architecture:** `Component` trait with `create` / `update` / `view` / `props_changed` lifecycle
- **`El` builder DSL:** 73 element constructors, global attributes, callbacks, child/content projection
- **`Ctx<Msg>` callback factory** with `on_click`, `callback`, `spawn` (background thread), `keyed_child`
- **`Store<T>`** shared reactive state with `subscribe`/`get`/`set`/`update`
- **Three-path render model:** clean fast-path / skeleton patch-path / full render; per-component render caching
- **Lifecycle hooks:** `mounted`, `updated`, `destroyed`

### Devtools (`wgpu-html-devtools`)

- **Component tree browser** with expand/collapse, selection, and breadcrumb bar
- **Styles inspector** showing computed properties for the selected element
- **Self-hosted:** Built with `wgpu-html-ui`; renders in its own window alongside the page
- **Attach via:** `Devtools::attach(&mut tree, open: bool)`

## ❌ What Is NOT Done

### HTML Parsing Gaps
- Comments + DOCTYPE tokenized then **dropped** at tree-build time
- Unknown tags drop their **entire subtree** silently
- No HTML5 insertion-mode state machine (no `<table>` foster-parenting)
- No `<!CDATA[]>`, no foreign content (SVG/MathML inner nodes)
- Whitespace-only text between tags is dropped

### CSS Parsing Gaps
- **No at-rules:** `@supports`, `@import`, `@keyframes`, `@font-face`, `@page` — not handled (`@media` IS handled)
- **No child/sibling combinators in cascade:** `>`, `+`, `~` not supported (only descendant ` `). Available in query engine only
- **No attribute selectors in stylesheet parser.** Available in query engine only
- **No structural/logical pseudo-classes in cascade.** Dynamic `:hover`/`:active`/`:focus` are supported; query engine supports many more including `:nth-child()`, `:has()`, `:not()`, `:is()`, `:where()`
- `transform`, `transition`, `animation`, `box-shadow` stored as raw `Option<String>` — never structured or applied
- Gradients parsed into `CssImage::Function(String)` but layout skips them — no gradient pipeline
- `filter` property silently dropped by parser

### Layout Gaps
- `z-index` parsed and stored on `LayoutBox`. Siblings sort by z-index in paint order (negative → auto → non-negative). No independent stacking contexts — cross-branch ordering is still tree DFS, so a deeply nested `z-index: 999` paints behind a shallow `z-index: 1` in a different subtree.
- **No floats** (`float: left/right`) — property is not even parsed
- **No table layout** (`display: table` and friends) — all 9 table `Display` variants fall through to block
- **No `display: inline / inline-block`** as an author-set value (IFC is auto-detected from content)
- `em/rem` use a hard-coded 16px when no font-size is inherited
- No baseline alignment in flex

### Rendering Gaps
- No `background-origin`, `background-attachment`, or multi-layer background rendering
- **No gradients** — parsed and stored but no gradient pipeline or shader
- **No box-shadow** — parsed but never consumed
- **No transforms** — parsed as raw string but never flows to LayoutBox or GPU
- Border styles `double/groove/ridge/inset/outset` render as plain solid
- Dashed/dotted on rounded boxes only follow the curve when all four corners are uniform-circular
- No multi-pass compositing (no stencil buffer usage beyond scissor)

### Interactivity Gaps
- **No `<select>` dropdown** menu rendering or interaction
- **No form submission** — `Enter` or `<button type="submit">` doesn't synthesise `SubmitEvent`
- `:focus-visible`, `:focus-within`, `:disabled` not yet matched in cascade (query engine only)
- No `Wheel` event dispatch to elements
- No `cursor` styling (property parsed but not applied to OS cursor)
- No `preventDefault` / `stopPropagation` semantics
- `InputEvent` not yet emitted on programmatic value changes
- No `<link>` stylesheet loading (field exists, no HTTP fetch)

### Explicitly Out of Scope (Forever)
- **No JavaScript.** No `<script>` execution, no JS engine, no scripting hooks, no `eval`, no `addEventListener`-style JS callbacks.
