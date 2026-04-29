# wgpu-html — Complete Project Status

> **Date:** 2026-04-29
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

### Crate map (12 crates)

| Crate | Role |
|---|---|
| `wgpu-html-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `wgpu-html-models` | `Style` struct (~80+ fields), CSS enums, ~100 HTML element structs |
| `wgpu-html-tree` | `Tree` / `Node` / `Element`, font registration, event callbacks, `InteractionState` (hover/active/focus/selection/scroll/modifiers), path-based mouse + keyboard + focus dispatch (`dispatch` module), `is_focusable` / Tab traversal helpers (`focus` module), DOM-style query helpers (`query` module — `CompoundSelector`, `query_selector*`) |
| `wgpu-html-style` | Cascade engine: UA stylesheet, selector matching (`MatchContext` for `:hover`/`:active`/`:focus`), field merge, CSS-wide keywords, inheritance, CSS Color Module Level 4 system colors, generic-family font fallback |
| `wgpu-html-text` | Font database, text shaping (cosmic-text), glyph atlas (rasterisation + GPU upload) |
| `wgpu-html-events` | Typed DOM-style event structs: `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, `FocusEvent`, `InputEvent`, `EventPhase`, `HtmlEventType`; bubbling semantics |
| `wgpu-html-layout` | Block flow, Flexbox, Grid, inline formatting context, hit testing, image loading/cache, scroll geometry, `<input>` / `<textarea>` placeholder shaping |
| `wgpu-html-renderer` | wgpu device/surface, quad pipeline (SDF shader), glyph pipeline, image pipeline, scissor clipping, screenshot |
| `wgpu-html` | Façade: `parse → cascade → layout → paint`, interactivity wrappers (layout-aware), `PipelineTimings`, text selection helpers, public `scroll` module (scrollbar geometry + paint, document/element scroll utilities) |
| `wgpu-html-winit` | winit ↔ engine glue: type translators (`mouse_button`, `key_to_dom_key`, `keycode_to_dom_code`, `keycode_to_modifier`), forwarders (`update_modifiers`, `forward_keyboard`, `handle_keyboard`), batteries-included `WgpuHtmlWindow` harness (`AppHook` trait + `EventResponse` + `HookContext` + `FrameTimings`; built-in viewport scroll, scrollbar drag, clipboard, F12 screenshot), `system_font_variants` / `register_system_fonts` |
| `wgpu-html-egui` | Alternative `egui` / `eframe` integration backend; the demo can pick between the winit and egui renderers via `--renderer=` |
| `wgpu-html-demo` | Thin shell over `wgpu-html-winit` (or `wgpu-html-egui` via `--renderer=egui`); HTML loading, demo hooks (F9 profiling), `--profile` CLI flag |

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
- **Backgrounds:** `background-color, background-clip, background-repeat`; partial `background` shorthand (color / image / repeat / clip); `background-image` typed as URL/function, `background-size / -position` parsed and consumed from raw strings.
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
- **Text shaping** (`wgpu-html-text/shape.rs`): `cosmic-text`-based shaping, `font-family` list fallback, `font-weight`, `font-style: italic`, `font-size`, `letter-spacing`, `text-transform` (uppercase / lowercase / capitalize), `white-space: pre` vs collapse. Key types: `ShapedRun`, `ShapedLine`, `PositionedGlyph`.
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

**Pointer:**
- **Mouse input:** `CursorMoved`, `MouseInput` (press/release), `CursorLeft`, `MouseWheel` — all wired.
- **Event system:** layout-aware wrappers (`pointer_move`, `pointer_leave`, `mouse_down`, `mouse_up`) in `wgpu_html::interactivity`; path-based dispatch (`dispatch_pointer_move`, `dispatch_mouse_down/up`, `dispatch_pointer_leave`) in `wgpu_html_tree::dispatch` (no layout dep). Hit-testing happens in the wrappers.
- **Event typing:** `wgpu-html-events` crate provides `HtmlEvent`, `MouseEvent`, `KeyboardEvent`, `FocusEvent`, `InputEvent`, `EventPhase`, `HtmlEventType`; events carry DOM-compatible `target`, `current_target`, `time_stamp`, `buttons` bitmask.
- **Event bubbling:** mousedown/mouseup/click bubble target → root; mouseenter/mouseleave do not bubble (DOM semantics). `keydown`/`keyup` bubble. `focusout`/`focusin` bubble; `focus`/`blur` don't.
- **Hover / active / focus tracking:** `InteractionState` tracks `hover_path`, `active_path`, `focus_path`; `:hover` / `:active` / `:focus` cascade integration via `MatchContext::for_path`. `:focus` is exact-match (not prefix); `:focus-within` is not yet implemented.
- **Callback slots on Node:** `on_click`, `on_mouse_down`, `on_mouse_up`, `on_mouse_enter`, `on_mouse_leave` — `Arc<dyn Fn(&MouseEvent)>`; plus `on_event: Arc<dyn Fn(&HtmlEvent)>` for typed DOM events (mouse / keyboard / focus / etc.).
- **MouseEvent:** carries `pos`, `button`, `modifiers` (shift/ctrl/alt/meta), `target_path`, `current_path`.

**Focus + keyboard:**
- **Focus state on tree:** `InteractionState::focus_path: Option<Vec<usize>>`; `Tree::focus(path)` / `Tree::blur()` / `Tree::focus_next(reverse)` (and free-function equivalents). `Tree::focus(path)` walks up to the nearest focusable ancestor; the closest-ancestor logic also fires from `mouse_down` automatically.
- **Focusable predicate:** `is_focusable` / `is_keyboard_focusable` recognise `<button>` / `<a href>` / `<input>` (non-hidden) / `<textarea>` / `<select>` / `<summary>`, plus any element with `tabindex >= 0`. `tabindex < 0` means scriptable-focus only (excluded from Tab traversal).
- **Tab navigation built in:** `Tree::key_down(key, code, repeat)` advances focus when `key == "Tab"` (Shift held → reverse), wrapping at the ends.
- **Modifier state on tree:** `Modifier { Ctrl, Shift, Alt, Meta }`, `Tree::set_modifier(modifier, down)`, `Tree::modifiers()`. Dispatchers no longer take a `Modifiers` parameter — they read `tree.interaction.modifiers` when constructing DOM events.
- **DOM-style query helpers:** `wgpu_html_tree::query` — `CompoundSelector`, `Tree::query_selector(sel)`, `query_selector_all`, `query_selector_path`, `query_selector_all_paths` (and `Node::*` mirrors). Supports `tag` / `#id` / `.class` compound selectors; no combinators or pseudo-classes.

**Form fields:**
- **Placeholder rendering** for `<input>` and `<textarea>`: `compute_placeholder_run` shapes the `placeholder` attribute and attaches it as the box's text run. Color = cascaded `color` × alpha 0.5 (matches the browser default `::placeholder` styling). Single-line inputs vertically centre the run inside `content_rect` and clip overflowing glyphs at the right padding edge; textareas soft-wrap inside `content_rect.w` and stay top-aligned. Suppressed for `type="hidden"`, non-empty `value`, or non-empty textarea content. Wired into both `layout_block` and `layout_atomic_inline_subtree`.

**Text selection:**
- `TextCursor` / `TextSelection` on `InteractionState`; drag-to-select wired in `interactivity.rs`; `select_all_text` / `selected_text` in `wgpu-html`; `Ctrl+A` + `Ctrl+C` built into the `wgpu-html-winit` harness (via `arboard`).

**Scrolling:**
- `InteractionState::scroll_offsets_y: BTreeMap<Vec<usize>, f32>`; viewport and per-element scrollbar paint (10 px track, drag-to-scroll); `MouseWheel` scrolls viewport and nested scroll containers. Public `wgpu_html::scroll` module exposes `ScrollbarGeometry`, `scrollbar_geometry`, `scroll_y_from_thumb_top`, `paint_viewport_scrollbar`, `translate_display_list_y`, `clamp_scroll_y`, hit-tests, and per-element variants.

**`wgpu-html-winit` harness:**
- Batteries-included `WgpuHtmlWindow` wraps a `&mut Tree` and runs the full event loop (winit `ApplicationHandler` impl). One-call setup: `create_window(&mut tree).with_title(...).run()`.
- Builders: `with_title`, `with_size`, `with_exit_on_escape`, `with_clipboard_enabled`, `with_screenshot_key`, `with_hook`.
- Built-in: viewport scroll (mouse wheel) + scrollbar drag (viewport + per-element); clipboard (Ctrl+A select-all, Ctrl+C copy via `arboard`); screenshot (F12 → `screenshot-<unix>.png`).
- `AppHook` trait for app extension: `on_key`, `on_frame`, `on_pointer_move`. `EventResponse { Continue, Stop }` lets `on_key` skip harness defaults. `HookContext` exposes `&mut Tree`, `&mut Renderer`, `&mut TextContext`, `Option<&LayoutBox>`, `&Window`, `&ActiveEventLoop`. `FrameTimings` carries cascade/layout/paint/render ms.
- Type translators: `mouse_button`, `keycode_to_modifier`, `key_to_dom_key`, `keycode_to_dom_code`. Forwarders: `update_modifiers`, `forward_keyboard`, `handle_keyboard`.
- System-font discovery: `system_font_variants()` (Windows / Linux / macOS table) and `register_system_fonts(tree, family)` lifted out of the demo so any host can pull them in.

**Demo:**
- Now ~450 lines (was ~1460). The bespoke `App` struct + `ApplicationHandler` impl is gone. `main()` parses HTML, registers system fonts, installs example callbacks, and calls `wgpu_html_winit::create_window(&mut tree).with_hook(DemoHook::new(...)).run()`.
- `DemoHook impl AppHook`: F9 toggles a small built-in profiler (1-second rolling stats: cascade / layout / paint / render avg+max, plus hover-move count and pointer dispatch ms).
- `--renderer=winit|egui` CLI flag picks between the winit harness (default) and `wgpu-html-egui`.

**Continuous redraw loop** via `request_redraw` in `about_to_wait`. Hover-path changes trigger a throttled redraw (16 ms budget) rather than unconditional full-speed redraws.

### 8. Profiling (Inline)
- **`PipelineTimings`** struct in `wgpu-html/src/lib.rs`: `cascade_ms`, `layout_ms`, `paint_ms`, `total_ms()`.
- **`compute_layout_profiled`** and **`paint_tree_returning_layout_profiled`**: profiled variants of the main API; return `(result, PipelineTimings)`.
- **`ProfileWindow`** in `wgpu-html-demo/src/main.rs`: rolling per-second stats for every pipeline stage (`tree`, `cascade`, `layout`, `paint`, `postprocess`, `atlas_upload`, `render`) plus dedicated hover-path latency breakdown (avg/max pointer-move time, hover-triggered frame breakdown). Printed to stderr once per second.
- Note: this is an **inline profiler only**. A full `wgpu-html-profiler` crate with ring-buffer history, GPU timing, trace export, and an embedded UI panel is specified in `spec/profiler.md` but does not yet exist.

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
- **No structural / logical pseudo-classes / pseudo-elements** (`:focus`, `:nth-child`, `:not()`, `:is()`, `::before`, `::after`, …). Dynamic `:hover` and `:active` *are* supported.
- `transform, transition, animation, box-shadow` stored as **raw `Option<String>`** — never structured or applied.
- No `calc()`, no `var(…)`, no custom properties (`--foo`).
- No structured types for shadows, gradients, transforms, filters, masks, clip-paths.

### Style / Cascade Gaps
- `<link rel="stylesheet">` not loaded — only inline `<style>` blocks.
- `currentColor` resolves to `None` (no foreground-color fallback for borders).
- No `:focus` / `:focus-visible` / structural pseudo-class state integration.

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
- No `background-origin`, `background-attachment`, or multi-layer background rendering.
- **No gradients:** `linear-gradient(…)` stays a raw string.
- **No box-shadow.**
- **No transforms / opacity layers / filters / blend modes** (per-quad alpha only).
- Border styles `double / groove / ridge / inset / outset` render as plain solid.
- Dashed/dotted on rounded boxes only follow the curve when all four corners are uniform-circular; otherwise corners stay bare.
- A border without an explicit color is skipped (no fallback to `color` / `currentColor`).
- No multi-pass compositing (no stencil buffer usage beyond scissor).

### Interactivity Gaps
- **No text editing** for `<input>` / `<textarea>` — typing into a focused field has no effect on its `value`. Placeholder rendering is done; live typing / caret movement / IME / arrow-key navigation aren't.
- **No `<input type="checkbox" / "radio">` click-to-toggle** — `checked` is parsed and the cascade reads it, but pointer presses don't flip it.
- **No `<select>` dropdown** menu rendering or interaction.
- **No form submission** — `Enter` in a focused input or click on `<button type="submit">` doesn't synthesise `SubmitEvent`.
- **`:focus-visible`, `:focus-within`, `:disabled`** not yet matched in cascade.
- **No `Wheel` event dispatch to elements** — wheel scrolls the viewport and detects scroll-container scroll, but is not forwarded to element `on_event` callbacks.
- **No cursor styling** (`cursor` property parsed but not applied to the OS cursor shape).
- **No `pointer-events: none`** skipping in hit test.
- No event `preventDefault` / `stopPropagation` semantics.
- No double-click / triple-click / context-menu / aux-click synthesis.
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
| `:hover` / `:active` / `:focus` cascade integration | ✅ Done (`MatchContext::for_path` in `wgpu-html-style`; `:focus` is exact-match — no propagation to ancestors yet) |
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
| Images (`<img>`, `background-image`) | ✅ Done (HTTP(S)/file/data-URI, GIF/WebP animation, two-level cache with TTL + byte-budget eviction) |
| Hit testing | ✅ Done |
| Mouse events + bubbling | ✅ Done (typed `HtmlEvent` + legacy `MouseEvent` slots) |
| Hover / active tracking | ✅ Done |
| Text selection (drag-select, Ctrl+A, Ctrl+C) | ✅ Done (anchor/focus cursors, `arboard` clipboard) |
| Viewport scrollbar + scroll position | ✅ Done (paint + drag; wheel scroll) |
| Per-element scroll containers | ⚠️ Partial (scroll offset + scrollbar paint; no `Wheel`→`on_event` dispatch) |
| Inline pipeline profiling (`PipelineTimings`) | ⚠️ Partial (CPU stage timing + hover latency; no `wgpu-html-profiler` crate) |
| Screenshot (F12 → PNG) | ✅ Done |
| Positioned layout (absolute/relative/fixed) | ❌ Not done |
| z-index | ❌ Not done |
| Floats | ❌ Not done |
| Table layout | ❌ Not done |
| Gradients | ❌ Not done |
| box-shadow | ❌ Not done |
| Transforms / transitions / animations | ❌ Not done |
| Opacity layers / filters / blend modes | ❌ Not done |
| Keyboard event dispatch (`keydown`/`keyup` to focused element) | ✅ Done (via `Tree::key_down`/`key_up`, modifier state on tree) |
| Tab / Shift+Tab focus traversal | ✅ Done (built into `Tree::key_down`) |
| Focus state (`focus_path`, `focus`/`blur`/`focusin`/`focusout`) | ✅ Done |
| `<input>` / `<textarea>` placeholder rendering | ✅ Done (`::placeholder`-style alpha; vertical centre + horizontal clip on inputs; soft-wrap on textarea) |
| `<input>` / `<textarea>` value rendering + typing | ❌ Not done |
| Checkbox / radio click toggle, `<select>` menu, form submit | ❌ Not done |
| `:focus-visible` / `:focus-within` / `:disabled` | ❌ Not done |
| Pseudo-elements (`::before`, `::after`, …) | ❌ Not done |
| Structural pseudo-classes (`:nth-child`, `:not()`, …) | ❌ Not done |
| Child/sibling combinators (`>`, `+`, `~`) | ❌ Not done |
| Attribute selectors | ❌ Not done |
| At-rules (@media, @keyframes, @font-face…) | ❌ Not done |
| calc() / var() / custom properties | ❌ Not done |
| `<link>` stylesheet loading | ❌ Not done |
| JavaScript | 🚫 Permanently out of scope |
