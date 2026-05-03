# wgpu-html — Complete Project Status

> **Date:** 2026-05-03
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

### Crate map (14 crates)

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
| `wgpu-html-ui` | Elm-architecture component framework: `Component` trait, `Ctx` callback factory, `MsgSender` channel, `Store<T>` reactive shared state, `Children` content-projection type, `App` / `Mount` entry points, per-component render caching, keyed child identity, background task dispatch |
| `wgpu-html-devtools` | Visual devtools panel (component tree browser, styles inspector, breadcrumb bar) implemented with `wgpu-html-ui`; attaches to a host tree via `Devtools::attach` |
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
- **DOM-style element lookup:** `get_element_by_id()`, `get_element_by_class_name()`, `get_elements_by_class_name()`, `get_element_by_name()`, `get_elements_by_name()`, `get_element_by_tag_name()`, `get_elements_by_tag_name()` (all returning `&Node`), plus path-returning `find_elements_by_*()` variants.
- **HTML serialisation:** `Node::to_html()` serialises a subtree to outer HTML; `Tree::to_html()` serialises the full document; `Tree::node_to_html(path)` serialises a specific node.
- **Layout rect caching:** `Node::rect: Option<NodeRect>` — populated by layout pass for element positioning queries.

### 2. CSS Parsing
- **Box model:** `display, position, top/right/bottom/left, width, height, min-/max-width/height, box-sizing`.
- **Spacing:** `margin`, `padding` — 1/2/3/4-value shorthand + per-side longhands.
- **Backgrounds:** `background-color, background-clip, background-repeat`; partial `background` shorthand (color / image / repeat / clip); `background-image` typed as URL/function, `background-size / -position` parsed and consumed from raw strings.
- **Borders:** `border` shorthand; per-side shorthand + `-width / -style / -color` longhands; `border-radius` with `/`-separated elliptical syntax, 1–4-corner expansion, per-corner `<h> <v>` longhands.
- **Typography:** `color, font-family, font-size, font-weight, font-style, line-height, letter-spacing, text-align, text-transform, white-space, text-decoration, vertical-align`.
- **Overflow:** `overflow, overflow-x, overflow-y`.
- **Misc:** `opacity, visibility, z-index, pointer-events, user-select, cursor`.
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
- `user-select: none` suppresses text cursor hit-testing, drag-to-select, and selection highlight painting. `user-select: text | all` treated as `auto` (normal selectable).

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

### 8. Component Framework (`wgpu-html-ui`)

Elm-architecture component model layered on top of the core engine. Components produce element trees via a builder DSL; the runtime drives the update/render loop and wires everything into `wgpu-html-winit`.

#### Component trait

```
Component::create(props) → Self
Component::update(msg, props) → ShouldRender
Component::view(&self, props, ctx, env) → El
Component::props_changed(old, new) → ShouldRender   // default: Yes
Component::mounted(sender)                            // lifecycle, receives MsgSender
Component::updated(props)                             // post-update hook
Component::destroyed()                                // lifecycle
Component::scope() → &'static str                    // CSS scope prefix
Component::styles() → Stylesheet                     // scoped CSS
```

#### `El` builder DSL

- 73 element constructor functions (`el::div`, `el::button`, `el::input`, …).
- Global attributes: `.id`, `.class`, `.style`, `.hidden`, `.tabindex`, `.data(key, val)`, `.attr_title`, `.custom_property`.
- Children: `.child(el)`, `.children(iter)`, `.text(t)`.
- Callbacks: `.on_click`, `.on_mouse_down`, `.on_mouse_up`, `.on_mouse_enter`, `.on_mouse_leave`, `.on_event` (+ `_cb` variants for pre-built `Arc` callbacks).
- Element-specific mutation: `.configure(|model| { … })`.
- `El` implements `Clone` — can be stored in `Props` for named-slot patterns.
- `Children` — a cloneable `Vec<El>` newtype for variadic content projection; `from(iter)`, `iter()`, `IntoIterator`, `FromIterator`.

#### `Ctx<Msg>` — callback factory

- `ctx.on_click(Msg)` → `MouseCallback` (send fixed message on click).
- `ctx.callback(|ev| Msg)` → `MouseCallback`.
- `ctx.event_callback(|ev| Option<Msg>)` → `EventCallback`.
- `ctx.sender()` → `MsgSender<Msg>` clone for custom closures.
- `ctx.scoped("class")` → scoped class name via `Component::scope`.
- `ctx.child::<C>(props)` — embed child component (positional key `"__pos_N"`).
- `ctx.keyed_child::<C>(key, props)` — embed with explicit string key; stable across reordering.
- `ctx.spawn(|| Msg)` — spawn OS thread; result sent as a message when complete.

#### `Store<T>` — shared reactive state

- `Store::new(value)` — wrap any `Send + Sync + 'static` value.
- `store.get()` → `T` (clone), `store.set(v)`, `store.update(|v| …)`.
- `store.on_change(|v| …)` — raw subscriber callback.
- `store.subscribe(&sender, |v| Msg)` — bridge to a component's message queue.
- Subscribe inside `Component::mounted(sender)` for lifecycle-scoped wiring.
- Cheap to clone — all clones share the same `Arc<Mutex<T>>` + listener list.

#### Reconciliation / update model

- `MsgSender::send` enqueues a message and calls `wake()` (→ `request_redraw`).
- `Runtime::process` drains messages in a loop until stable; cascades child→parent callbacks within a single frame.
- **Three-path render model** per `MountedComponent`:
  - Each component stores `last_node` (resolved), `skeleton_node` (raw `view()` output with placeholders), `needs_render`, and `subtree_dirty`.
  - **Path 1 — clean fast-path** (`!needs_render && !subtree_dirty`): return `last_node`; zero work, no `view()` call.
  - **Path 2 — patch path** (`!needs_render && subtree_dirty`): parent's `view()` is **skipped**. Clone `skeleton_node` (tiny — placeholder divs only, much smaller than `last_node`) and re-substitute every child: dirty ones re-render, clean ones return their `last_node`. Saves one `view()` call per ancestor of every updated leaf.
  - **Path 3 — full render** (`needs_render`): call `view()`, reconcile child set, store raw output as `skeleton_node`, substitute children, cache as `last_node`.
- **Keyed children**: child identity is `(String, TypeId)`. Positional key (`"__pos_N"`) matches call-site order; user key from `keyed_child` survives list reordering.
- `Component::updated(props)` is called after each render caused by the component's own state change (not after initial mount).

#### Entry points

- `App::new::<C>(props)` / `App::with_state::<C>(state, props)` — full winit application.
- `App::stylesheet(css)`, `.title(...)`, `.size(w, h)`, `.setup_tree(f)`, `.with_secondary(f)`.
- `Mount<C>` — drive a component tree manually inside an existing `Tree` (no winit dep).

---

### 9. Profiling (Inline)
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
- **No at-rules:** `@supports, @import, @keyframes, @font-face, @page` — not handled. `@media` IS handled (width/height/orientation with min/max prefix + `not`; evaluated during cascade via `cascade_with_media`).
- **No child/sibling combinators:** `>`, `+`, `~` are not supported (only descendant ` ` works).  *(Note: the query engine in `wgpu-html-tree/src/query.rs` supports child / next-sibling / subsequent-sibling combinators and attribute selectors for `querySelector`/`matches`/`closest`; the stylesheet parser and cascade selector matching have not been updated to match.)*
- **No attribute selectors in stylesheet parser** (`[href]`, `[type=text]`). (Attribute selectors work in the `query_selector` API.)
- **No structural / logical pseudo-classes in stylesheet parser.** Dynamic `:hover` and `:active` *are* supported in cascade matching; the query engine additionally supports `:focus`, `:focus-within`, `:checked`, `:disabled`, `:enabled`, `:required`, `:optional`, `:read-only`, `:read-write`, `:placeholder-shown`, `:first-child`, `:last-child`, `:only-child`, `:first-of-type`, `:last-of-type`, `:nth-child()`, `:nth-last-child()`, `:nth-of-type()`, `:not()`, `:is()`, `:where()`, `:has()`, `:root`, `:scope`, `:lang()`, `:dir()` for `query_selector*`.
- `transform, transition, animation, box-shadow` stored as **raw `Option<String>`** — never structured or applied.
- Gradients (`linear-gradient`, `radial-gradient`, …) parsed into `CssImage::Function(String)` but layout skips them — no gradient pipeline exists. `filter` property is silently dropped by the parser.
- No structured types for shadows, transforms — stored as raw strings only.

### Style / Cascade Gaps
- `<link rel="stylesheet">` not loaded — only inline `<style>` blocks. (`linked_stylesheets` field exists on `Tree` and is consumed by the cascade, but no HTTP fetch to populate it.)
- `currentColor` resolves to `None` (no foreground-color fallback for borders).
- `calc()` / `min()` / `max()` / `clamp()` fully parsed (AST with 18 CSS math functions) and evaluated in `length.rs` at layout time.
- `var()` + custom properties (`--foo`) fully implemented: parsed, inherited, recursive variable substitution with cycle detection, late re-parse through `apply_css_property()`.
- Programmatic custom properties via `Node::set_custom_property()` flow through cascade and participate in `var()` resolution.
- No `:focus-visible` / `:focus-within` / `:disabled` in *cascade* matching (available in query engine only).

### Layout Gaps
- **Positioned layout** (`absolute` / `relative` / `fixed`) — fully implemented. `layout_out_of_flow_block()` resolves containing blocks, insets, shrink-to-fit, right/bottom anchoring. `apply_relative_position()` handles relative and sticky (sticky is degraded to relative — no scroll-pinning). 6 tests cover absolute/fixed/relative scenarios.
- **No `z-index`** — parsed and stored on `Style`, but no `LayoutBox` field; paint order is tree DFS only.
- **No floats** (`float: left/right`) — `float` property is not even parsed.
- **No table layout** (`display: table` and friends) — all 9 table `Display` variants are parsed but fall through to block layout.
- **No `display: inline / inline-block`** as an author-set value (IFC is auto-detected from content, not from `display`).
- `em / rem` use a hard-coded 16px when no font-size is inherited (no full font cascade for unit resolution).
- No baseline alignment in flex.
- Transforms not applied to layout.
- `flex-shrink` with `min-width: auto` now respects content-based minimum size per CSS-Flex-1 §4.5 — flex items with `overflow: visible` cannot shrink below their content width.

### Rendering Gaps
- No `background-origin`, `background-attachment`, or multi-layer background rendering.
- **No gradients:** `linear-gradient(…)` parsed and stored but layout skips it — no gradient pipeline or shader.
- **No box-shadow** — parsed but never consumed.
- **No transforms** — parsed as raw string but never flows to LayoutBox or GPU.
- **Opacity is fully working** (computed during layout, inherited multiplicatively, baked into color alpha, image pipeline respects it).
- **No `filter`** — not parsed at all.
- Border styles `double / groove / ridge / inset / outset` render as plain solid.
- Dashed/dotted on rounded boxes only follow the curve when all four corners are uniform-circular; otherwise corners stay bare.
- A border without an explicit color is skipped (no fallback to `color` / `currentColor`).
- No multi-pass compositing (no stencil buffer usage beyond scissor).

### Interactivity Gaps
- **Text editing for `<input>` / `<textarea>`** — fully implemented. `text_edit.rs` (12 functions, 425 lines) handles insert, delete, backspace, arrow-key navigation, Home/End, Shift-select, Ctrl+A, line breaks in textareas, multibyte/UTF-8. Wired in `dispatch.rs` for keyboard and `interactivity.rs` for click caret placement (with single/double/triple-click word/line selection). Readonly is respected.
- **No `<input type="checkbox" / "radio">` click-to-toggle** — `checked` is parsed and the cascade reads it, but pointer presses don't flip it.
- **No `<select>` dropdown** menu rendering or interaction.
- **No form submission** — `Enter` in a focused input or click on `<button type="submit">` doesn't synthesise `SubmitEvent`.
- **`:focus-visible`, `:focus-within`, `:disabled`** not yet matched in cascade (available in query engine only).
- **No `Wheel` event dispatch to elements** — wheel scrolls the viewport and detects scroll-container scroll, but is not forwarded to element `on_event` callbacks.
- **No cursor styling** (`cursor` property parsed but not applied to the OS cursor shape).
- ~~**No `pointer-events: none`** skipping in hit test.~~ **Done** — `pointer-events: none` elements are transparent to hit-testing; children with `auto` remain hittable.
- No event `preventDefault` / `stopPropagation` semantics.
- No double-click / triple-click / context-menu / aux-click synthesis.
- `InputEvent` is not yet emitted (`text_input` mutates values but does not dispatch a DOM event).
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
| `@media` queries | ✅ Done (width/height/orientation, min/max, not) |
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
| `pointer-events: none` hit-test skip | ✅ Done |
| `user-select: none` enforcement | ✅ Done |
| Text selection (drag-select, Ctrl+A, Ctrl+C) | ✅ Done (anchor/focus cursors, `arboard` clipboard) |
| Viewport scrollbar + scroll position | ✅ Done (paint + drag; wheel scroll) |
| Per-element scroll containers | ⚠️ Partial (scroll offset + scrollbar paint; no `Wheel`→`on_event` dispatch) |
| Inline pipeline profiling (`PipelineTimings`) | ⚠️ Partial (CPU stage timing + hover latency; no `wgpu-html-profiler` crate) |
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
| `querySelector` / `matches` engine | ✅ Done (full CSS Level 4: all combinators, attribute operators, pseudo-classes including `:has()`, `:is()`, `:where()`, `:not()`, structural pseudos, state pseudos) |
| Placeholder rendering | ✅ Done |
| Text editing + caret navigation | ✅ Done (insert, delete, arrow keys, Home/End, Shift-select, word/line click-select) |
| Checkbox / radio click toggle, select menu, form submit | ❌ Not done |
| `:focus-visible` / `:focus-within` / `:disabled` in cascade | ❌ Not done (query engine only) |
| Pseudo-elements | ❌ Not done |
| calc() / min() / max() / clamp() | ✅ Done (full AST + evaluation in length.rs) |
| var() / custom properties (`--foo`) | ✅ Done (parsed, inherited, recursive substitution, cycle detection) |
| `<link>` stylesheet loading | ❌ Not done |
| Devtools crate (`wgpu-html-devtools`) | ⚠️ Partial (exists with component tree browser, styles inspector, breadcrumb bar; panels not yet self-hosted) |
| JavaScript | 🚫 Permanently out of scope |
| **Component framework (`wgpu-html-ui`)** | |
| `Component` trait (create / update / view / props_changed) | ✅ Done |
| Local component state | ✅ Done (struct fields) |
| Props (immutable, Clone, passed from parent) | ✅ Done |
| `El` builder DSL (73 elements, all global attrs, callbacks) | ✅ Done |
| `El: Clone` + `Children` content-projection type | ✅ Done |
| `Ctx::child` (positional keying) | ✅ Done |
| `Ctx::keyed_child` (explicit string key, list-stable) | ✅ Done |
| `Ctx::spawn` (background thread → message) | ✅ Done |
| `Store<T>` shared reactive state | ✅ Done |
| `Store::subscribe` / `Component::mounted(sender)` | ✅ Done |
| Per-component render cache + `subtree_dirty` fast-path | ✅ Done — three-path model: clean fast-path / skeleton patch-path / full render |
| `Component::updated` post-render lifecycle hook | ✅ Done |
| `Component::mounted` / `Component::destroyed` lifecycle | ✅ Done |
| Scoped CSS (`Component::scope` + `Component::styles`) | ✅ Done |
| `App` / `Mount` entry points, `Env` shared context | ✅ Done |
| Virtual-DOM diffing of `Node` tree (structural diff) | ❌ Not done — full subtree replace on component re-render |
| Subscription handle / auto-unsubscribe on destroy | ❌ Not done — subscriptions accumulate until `Store` is dropped |
| Async `Future`-based task dispatch | ❌ Not done — thread-based `spawn` only (no async runtime) |
| Named template slots / portal rendering | ❌ Not done |
