---
id: status
title: Implementation Status
---

# Implementation Status

> **Date:** 2026-05-02
> **Scope:** GPU-accelerated HTML/CSS renderer via `wgpu`. **No JavaScript — ever.**

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
| Mouse events + bubbling | ✅ Done (typed `HtmlEvent` + legacy `MouseEvent` slots) |
| Hover / active tracking | ✅ Done |
| Text selection (drag-select, Ctrl+A, Ctrl+C) | ✅ Done (anchor/focus cursors, `arboard` clipboard) |
| Viewport scrollbar + scroll position | ✅ Done (paint + drag; wheel scroll) |
| Per-element scroll containers | ⚠️ Partial (scroll offset + scrollbar paint; no `Wheel`→`on_event` dispatch) |
| Screenshot (F12 → PNG) | ✅ Done |
| Positioned layout (absolute/relative/fixed) | ✅ Done (sticky degraded to relative) |
| z-index | ❌ Not done (parsed, not consumed) |
| Floats | ❌ Not done |
| Table layout | ❌ Not done (parsed, falls through to block) |
| Gradients | ❌ Not done (parsed as raw string, skipped in layout) |
| box-shadow | ❌ Not done (parsed as raw string, not consumed) |
| Transforms / transitions / animations | ❌ Not done (parsed as raw string, not consumed) |
| Opacity | ✅ Done |
| Filter / blend modes | ❌ Not done |
| Keyboard event dispatch | ✅ Done |
| Tab / Shift+Tab focus traversal | ✅ Done |
| Focus state | ✅ Done |
| Placeholder rendering | ✅ Done |
| Text editing + caret navigation | ✅ Done (insert, delete, arrow keys, Home/End, Shift-select, word/line click-select) |
| Checkbox / radio click toggle, select menu, form submit | ❌ Not done |
| `:focus-visible` / `:focus-within` / `:disabled` in cascade | ❌ Not done (query engine only) |
| Pseudo-elements | ❌ Not done |
| calc() / min() / max() / clamp() | ✅ Done (full AST + evaluation) |
| var() / custom properties (`--foo`) | ✅ Done (parsed, inherited, recursive substitution, cycle detection) |
| `<link>` stylesheet loading | ❌ Not done |
| JavaScript | 🚫 Permanently out of scope |

## ✅ What Is Done

### HTML Parsing

- **Tokenizer:** open/close/self-closing tags, quoted/unquoted/boolean attributes, comments, DOCTYPE, raw-text elements (`<style>`, `<script>`, `<textarea>`, `<title>`)
- **Entity decoding:** `&amp; &lt; &gt; &quot; &apos; &nbsp; &#NN; &#xNN;`
- **Tree builder:** 14-entry void list, self-closing recognition, auto-close rules for `p / li / dt / dd / thead / tbody / tfoot / tr / th / td / option / optgroup / rt / rp`, EOF auto-close, synthetic `<body>` wrap
- **~100 element variants** with per-element attribute parsing
- **Global attributes:** `id, class, style, title, lang, dir, hidden, tabindex, accesskey, contenteditable, draggable, spellcheck, translate, role`
- **DOM-style element lookup and HTML serialisation**

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

### Layout Engine

- **Block Flow:** Vertical stacking, margin/padding, `box-sizing`, explicit/percentage lengths, min/max clamping, auto margin centering, border widths/styles/colors, `border-radius`, `background-clip`
- **Flexbox:** Complete CSS Flexbox Level 1 — direction, wrap, justify-content, align-items/self/content, flex-grow/shrink/basis, gap, order, multi-line wrapping, content-based min-size
- **CSS Grid:** `grid-template-columns/rows` with `px`, `fr`, `auto`, `minmax()`, `repeat()`, line+span placement, auto-placement, track sizing, justify/align-items/self/content, gap
- **Inline Formatting Context:** Line-box layout, word wrapping, `text-align` per line box, inline-block elements, anonymous text runs
- **Hit Testing:** `LayoutBox::hit_path((x, y))` → deepest-first, topmost wins

### Text Rendering

- **Font database:** Register `.ttf/.otf/.ttc` files; family + weight + style matching
- **Text shaping:** HarfBuzz via cosmic-text; `font-family` fallback, `font-weight`, `font-style: italic`, `font-size`, `letter-spacing`, `text-transform`, `white-space`
- **Glyph atlas:** CPU rasterisation → GPU texture upload, shelf-packing allocator
- **Text decorations:** `underline`, `line-through`, `overline` as solid quads

### Interactivity

- **Pointer:** Mouse input fully wired through typed `HtmlEvent` structs; bubbling (mousedown/up/click bubble; mouseenter/mouseleave don't); hover/active tracking with `InteractionState`
- **Focus + keyboard:** Focusable predicates for `<button>`, `<a href>`, `<input>`, `<textarea>`, `<select>`, `tabindex`; Tab/Shift+Tab navigation; modifier tracking
- **Form fields:** Placeholder rendering for `<input>` and `<textarea>`; full text editing with insert/delete/backspace/arrow keys/Home/End/Shift-select/click caret/multibyte UTF-8; readonly support
- **Text selection:** `TextCursor`/`TextSelection`, drag-to-select, `Ctrl+A`/`Ctrl+C`, `user-select: none`
- **Scrolling:** Viewport and per-element scrollbar paint + drag; `MouseWheel` scrolling

### Rendering / Paint

- **Quad pipeline:** Instanced quads with SDF for rounded fill and stroked rings; solid/dashed/dotted borders
- **Glyph pipeline:** Instanced glyph quads, alpha-tested from atlas texture, per-glyph color
- **Image pipeline:** Textured quads for `<img>` and `background-image`
- **Overflow clipping:** Rectangular scissor + SDF rounded clipping; nested clip intersection
- **Screenshot:** F12 → PNG export; programmatic `capture_to()`/`capture_rect_to()`/`screenshot_node_to()`

## ❌ What Is NOT Done

### HTML Parsing Gaps
- Comments + DOCTYPE tokenized then **dropped** at tree-build time
- Unknown tags drop their **entire subtree** silently
- No HTML5 insertion-mode state machine (no `<table>` foster-parenting)
- No `<!CDATA[]>`, no foreign content (SVG/MathML inner nodes)
- Whitespace-only text between tags is dropped

### CSS Parsing Gaps
- **No at-rules:** `@media`, `@supports`, `@import`, `@keyframes`, `@font-face`, `@page` — not handled
- **No child/sibling combinators in cascade:** `>`, `+`, `~` not supported (only descendant ` `). Available in query engine only
- **No attribute selectors in stylesheet parser.** Available in query engine only
- **No structural/logical pseudo-classes in cascade.** Dynamic `:hover`/`:active`/`:focus` are supported; query engine supports many more including `:nth-child()`, `:has()`, `:not()`, `:is()`, `:where()`
- `transform`, `transition`, `animation`, `box-shadow` stored as raw `Option<String>` — never structured or applied
- Gradients parsed into `CssImage::Function(String)` but layout skips them — no gradient pipeline
- `filter` property silently dropped by parser
- No `@media` queries

### Layout Gaps
- `z-index` parsed and stored on `Style` but no LayoutBox field; paint order is tree DFS only
- **No floats** (`float: left/right`) — property is not even parsed
- **No table layout** (`display: table` and friends) — all 9 table `Display` variants fall through to block
- `em/rem` use a hard-coded 16px when no font-size is inherited

### Rendering Gaps
- No `background-origin`, `background-attachment`, or multi-layer background rendering
- **No gradients** — parsed and stored but no gradient pipeline or shader
- **No box-shadow** — parsed but never consumed
- **No transforms** — parsed as raw string but never flows to LayoutBox or GPU
- Border styles `double/groove/ridge/inset/outset` render as plain solid
- Dashed/dotted on rounded boxes only follow the curve when all four corners are uniform-circular

### Interactivity Gaps
- **No `<input type="checkbox"/"radio">` click-to-toggle** — `checked` is parsed but pointer presses don't flip it
- **No `<select>` dropdown** menu rendering or interaction
- **No form submission** — `Enter` or `<button type="submit">` doesn't synthesise `SubmitEvent`
- `:focus-visible`, `:focus-within`, `:disabled` not yet matched in cascade (query engine only)
- No `Wheel` event dispatch to elements, no cursor styling, no `preventDefault`/`stopPropagation`

### Explicitly Out of Scope (Forever)
- **No JavaScript.** No `<script>` execution, no JS engine, no scripting hooks, no `eval`, no `addEventListener`-style JS callbacks.
