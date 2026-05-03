# wgpu-html vs RmlUI — Capability Comparison

> **Date:** 2026-05-03
> **Purpose:** Comparison of two GPU-accelerated HTML/CSS UI rendering engines aimed at game/application UI. wgpu-html is a Rust alternative to the C++ RmlUI library.

---

## 1. Project Overview

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Language** | Rust | C++ (C++17) |
| **License** | MIT | MIT |
| **GitHub Stars** | N/A (new) | ~4,000 |
| **Architecture** | Parser → Style Cascade → Layout → Paint → Renderer (5-stage pipeline) | Document → Style → Layout → Render (traditional UI engine) |
| **GPU Backend** | wgpu (Vulkan/Metal/DX12 via `wgpu::Backends::PRIMARY`) | OpenGL 2/3, Vulkan, SDL GPU, SDLrenderer, DirectX 12 (all optional backends) |
| **Rendering Model** | Display-list with draw commands (Quads + Glyphs + Images + Clip ranges) | Geometry generation (vertices, indices, textures) via `RenderInterface` |
| **Font Engine** | cosmic-text + CPU rasterization to glyph atlas | FreeType (replaceable via custom `FontEngineInterface`) |
| **External Dependencies** | wgpu, winit, cosmic-text, resvg, image, arboard | FreeType only (core). Optional: Lua, LunaSVG, rlottie, HarfBuzz |
| **Target Use Case** | Application UI, game HUD/menus, desktop tools | Game UI, embedded applications, installers, game menus |
| **First Release** | 2025-2026 (active development) | 2008 (as libRocket), forked as RmlUi in 2019 |
| **Maturity** | Alpha/beta — core pipeline works, some gaps | Production-grade — used in commercial games (The Thing: Remastered, Killing Time: Resurrected, ROSE Online, etc.) |
| **Platforms** | Windows, Linux, macOS (winit + wgpu) | Windows, Linux, macOS, Android, iOS, Switch, Emscripten |
| **JavaScript** | 🚫 Permanently out of scope | Not native; Lua plugin available |

---

## 2. Markup Language

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Format** | Standard HTML5 (`.html`) | RML (XHTML1-like `.rml`) |
| **DOCTYPE** | Parsed then dropped | Not used |
| **Element Count** | ~100 typed HTML element structs | ~20 custom elements + `<script>`/`<lottie>`/`<svg>` via plugins |
| **HTML5 Semantic Tags** | All major elements: `<div>`, `<span>`, `<p>`, `<h1>`–`<h6>`, `<img>`, `<a>`, `<button>`, `<input>` (22 types), `<form>`, `<textarea>`, `<select>`, `<table>`, `<ul>`/`<ol>`/`<li>`, etc. | Functional tags: `<body>`, `<div>`, `<span>`, `<p>`, `<h1>`–`<h6>`, `<br>`, `<img>`, `<a>`, `<input>`, `<textarea>`, `<select>`, `<option>`, `<label>`, `<form>` + custom: `<handle>`, `<tabset>`, `<tab>`, `<panel>`, `<progress>` |
| **HTML Serialization** | ✅ `Node::to_html()`, `Tree::to_html()` | ✅ InnerRML get/set |
| **Custom Elements** | Can be modeled via `Element` variants | Plugin-based (C++ element classes) |
| **HTML5 Auto-close** | ✅ 14 void elements, auto-close for p/li/td/etc. | Not applicable (XHTML1 self-closing) |
| **Entity Decoding** | ✅ `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`, `&#NN;`, `&#xNN;` | ✅ XML entities |
| **HTML5 Spec Compliance** | Partial (tokenizer + tree builder, no insertion-mode state machine) | XHTML1-based, not HTML5 |
| **Global Attributes** | `id`, `class`, `style`, `title`, `lang`, `hidden`, `tabindex`, `contenteditable`, `draggable`, `dir`, `accesskey`, `spellcheck`, `translate`, `role`, `aria-*`, `data-*` | `id`, `class`, `style`, `data-*` (for bindings) |

---

## 3. CSS / RCSS Support

### 3.1 Selectors

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Tag/Elements** | ✅ | ✅ |
| **`#id`** | ✅ | ✅ |
| **`.class` (multi-class)** | ✅ | ✅ |
| **Universal `*`** | ✅ | ✅ |
| **Descendant ` `** | ✅ | ✅ |
| **Child `>`** | ❌ (cascade only; query engine has it) | ✅ |
| **Next-sibling `+`** | ❌ (cascade only; query engine has it) | ✅ |
| **Subsequent-sibling `~`** | ❌ (cascade only; query engine has it) | ✅ |
| **Comma-list `A, B`** | ✅ | ✅ |
| **Attribute `[attr]`** | ❌ (cascade only; query engine has it) | ✅ |
| **`[attr=val]` / `~=` / `\|=` / `^=` / `$=` / `*=`** | ❌ (cascade only; query engine has it) | ✅ |
| **`:hover` / `:active` / `:focus`** | ✅ (cascade; `:focus` exact-match only) | ✅ (+ propagates to parents, unlike CSS) |
| **`:focus-visible`** | ❌ (query engine only) | ✅ |
| **`:checked`** | ❌ (query engine only) | ✅ |
| **`:disabled` / `:enabled`** | ❌ (query engine only) | ✅ (partial) |
| **`:first-child` / `:last-child` / `:only-child`** | ❌ (query engine only) | ✅ |
| **`:nth-child()` / `:nth-last-child()`** | ❌ (query engine only) | ✅ |
| **`:nth-of-type()` / `:nth-last-of-type()`** | ❌ (query engine only) | ✅ |
| **`:not()`** | ❌ (query engine only) | ✅ |
| **`:is()` / `:where()` / `:has()`** | ❌ (query engine only) | ❌ |
| **`:empty`** | ❌ | ✅ |
| **`:placeholder-shown`** | ❌ (query engine only) | ✅ |
| **`:root` / `:scope`** | ❌ (query engine only) | No/`:scope` for DOM API |
| **Pseudo-elements (`::before`, `::after`)** | ❌ | ❌ |
| **`!important`** | ✅ (4-band cascade) | ✅ (2-band cascade) |
| **Specificity** | `(id<<16) \| (class<<8) \| tag` | Standard CSS specificity (a,b,c) |

### 3.2 Cascade System

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **UA stylesheet** | ✅ (~50 rules: headings, margins, display:none, inline emphasis) | ❌ No built-in UA stylesheet (recommended stylesheet available separately) |
| **Author stylesheet** | ✅ (`<style>` blocks + `<link>` parsed) | ✅ (`<link type="text/rcss">` + inline) |
| **Inline style** | ✅ (`style="..."` attribute) | ✅ |
| **Inheritance** | ✅ (color, font-*, text-*, visibility, cursor, pointer-events, user-select) | ✅ |
| **`inherit` / `initial` / `unset`** | ✅ | ✅ |
| **`@media` queries** | ✅ (width/height/orientation with min/max + `not` prefix) | ✅ (width, height, orientation, resolution, prefers-color-scheme, prefers-reduced-motion) |
| **Dynamic pseudo recalc** | ✅ Incremental re-cascade of dirty paths only | ✅ |
| **`<link rel="stylesheet">`** | ⚠️ Parsed by cascade, `linked_stylesheets` map available but no HTTP fetch | ✅ File-based loading |

### 3.3 CSS Properties Summary

| Property Area | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`display`** | block, inline, inline-block, flex, grid, none | inline, block, inline-block, flow-root, flex, inline-flex, table, inline-table, table-row-group, table-row, table-column-group, table-column, table-cell, none |
| **`position`** | static, relative, absolute, fixed | static, relative, absolute, fixed |
| **Box Model** | width/height, min-/max-, margin, padding, box-sizing, border widths/colors/styles | width/height, min-/max-, margin, padding, box-sizing, border widths/colors |
| **`float`** | ❌ Not parsed | ✅ left, right, none |
| **`clear`** | ❌ | ✅ left, right, both, none |
| **`border-radius`** | ✅ Elliptical, per-corner H+V, CSS-3 corner clamping | ✅ Single-radius per corner only (no elliptical) |
| **`background-color`** | ✅ | ✅ |
| **`background-clip`** | ✅ border-box/padding-box/content-box | ❌ Uses decorators instead |
| **`background-image`** | ✅ URL + tiling + rounded-clip | ❌ Uses decorators instead |
| **`background-size` / `-position`** | ✅ Parsed, consumed from raw strings | ❌ |
| **`border-style`** | solid/dashed/dotted/none/hidden; double/groove/ridge/inset/outset → solid fallback | ❌ Not a property (only width+color) |
| **`box-shadow`** | ❌ Parsed as raw string, never consumed | ✅ |
| **Gradients** | ❌ Parsed as raw string, skipped | ✅ Linear, radial, conic gradients via decorators |
| **`opacity`** | ✅ (inherited multiplicatively) | ✅ |
| **`visibility`** | ✅ | ✅ |
| **`z-index`** | ❌ Parsed, not consumed (tree DFS paint order) | ✅ (applies to all elements) |
| **`overflow` / `overflow-x` / `overflow-y`** | ✅ hidden/scroll/auto/visible | ✅ hidden/scroll/auto/visible |
| **`pointer-events`** | ✅ auto/none | ✅ auto/none |
| **`user-select`** | ✅ auto/none/text/all (none suppresses selection) | ❌ Not applicable |
| **`cursor`** | ✅ Parsed, not applied to OS cursor | ✅ (application-defined cursor names) |
| **`caret-color`** | ❌ | ✅ |
| **`transform`** | ❌ Parsed as raw string only | ✅ Full 2D/3D transforms with interpolation |
| **`transform-origin`** | ❌ | ✅ |
| **`perspective` / `perspective-origin`** | ❌ | ✅ |
| **`transition`** | ❌ Parsed as raw string only | ✅ Full transitions with tweening functions |
| **`animation` / `@keyframes`** | ❌ Not parsed | ✅ Full keyframe animations with tweening |
| **`filter`** | ❌ Not parsed | ✅ All CSS filter functions |
| **`backdrop-filter`** | ❌ | ✅ |
| **`mask-image`** | ❌ | ✅ |
| **`clip`** | ✅ SDF rounded clip via overflow | ✅ auto/none/always (controls ancestor clipping) |
| **`word-break`** | ❌ | ✅ normal/break-all/break-word |
| **`text-overflow`** | ❌ | ✅ clip/ellipsis/custom string |
| **`vertical-align`** | ✅ (parsed, limited layout) | ✅ baseline/sub/super/text-top/text-bottom/middle/top/center/bottom |
| **`white-space`** | ✅ normal/pre | ✅ normal/pre/nowrap/pre-wrap/pre-line |
| **`font-kerning`** | ❌ | ✅ auto/normal/none |
| **`overscroll-behavior`** | ❌ | ✅ auto/contain |
| **Custom Properties `--foo`** | ✅ Full `var()` with inheritance + cycle detection | ❌ Not in CSS spec subset |
| **`calc()` / `min()` / `max()` / `clamp()`** | ✅ Full 18-node AST + evaluation | ⚠️ Limited `calc()` support |

---

## 4. Layout Engines

| Layout Model | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Block Flow** | ✅ Vertical stacking, margin/padding, auto-centering, min/max clamping | ✅ Full block formatting context |
| **Inline Formatting Context** | ✅ Line-box layout, word wrapping, text-align, inline-block children | ✅ Line-box layout, word wrapping, text-align |
| **Flexbox** | ✅ Complete CSS Flexbox Level 1 (direction, wrap, justify-content, align-items, align-content, align-self, flex-grow/shrink/basis, gap, order, multi-line, content-based min-size) | ✅ flex-direction, flex-wrap, justify-content, align-items, align-content, align-self, flex-grow/shrink/basis, gap |
| **CSS Grid** | ✅ grid-template-{columns,rows}, auto-{columns,rows}, auto-flow, line+span placement, auto-placement, fr units, minmax(), repeat(), justify/align-{items,self,content}, gap | ❌ Not supported |
| **Float Layout** | ❌ Not parsed | ✅ left/right floats with inline content wrap-around |
| **Clear** | ❌ | ✅ left/right/both |
| **Table Layout** | ❌ Display values parsed, fall through to block | ✅ Full table model: table, inline-table, table-row-group, table-row, table-column-group, table-column, table-cell |
| **Positioned Layout** | ✅ absolute/relative/fixed (sticky→relative) | ✅ static/relative/absolute/fixed |
| **Hit Testing** | ✅ `LayoutBox::hit_path()` + `find_element_from_point` (deepest-first, topmost wins) | ✅ Element-based hit testing |
| **`z-index` Stacking** | ❌ Tree DFS paint order only | ✅ Full stacking contexts |

---

## 5. Text & Typography

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Font Engine** | cosmic-text + CPU rasterization → glyph atlas | FreeType (replaceable via `FontEngineInterface`) |
| **Text Shaping** | cosmic-text (HarfBuzz-based) | FreeType kerning by default; HarfBuzz optional |
| **`font-family`** | ✅ Comma-separated fallback list, quoted names, generic keywords (sans-serif, serif, monospace, cursive, fantasy, system-ui, ui-*, -apple-system, BlinkMacSystemFont) | ✅ Single family only (`font-family: "LatoLatin"`) |
| **`font-weight`** | ✅ 100–900, normal/bold/bolder/lighter | ✅ normal/bold/<number 1–1000> |
| **`font-style`** | ✅ normal/italic/oblique | ✅ normal/italic (no oblique) |
| **`font-size`** | ✅ All length units + calc() | ✅ Length/percentage |
| **`line-height`** | ✅ Length units + number (default 1.25) | ✅ Number/length (default 1.2) |
| **`letter-spacing`** | ✅ Post-shape per-glyph offset | ✅ |
| **`text-transform`** | ✅ uppercase/lowercase/capitalize (pre-shape) | ✅ uppercase/lowercase/capitalize |
| **`text-align`** | ✅ left/right/center/start/end (no justify in impl) | ✅ left/right/center (no justify) |
| **`text-decoration`** | ✅ underline/line-through/overline (solid quads) | ✅ underline/overline/line-through |
| **`text-overflow`** | ❌ | ✅ clip/ellipsis/custom string |
| **`word-break`** | ❌ | ✅ normal/break-all/break-word |
| **`white-space`** | ✅ normal/pre | ✅ normal/pre/nowrap/pre-wrap/pre-line |
| **System Font Discovery** | ✅ Windows/Linux/macOS via `system_font_variants()` | ❌ (manual registration required) |
| **Font File Registration** | ✅ `.ttf` / `.otf` / `.ttc` per-document | ✅ `.ttf` / `.otf` per-context |
| **Font Effects** | ❌ | ✅ glow, outline, shadow, blur effects |
| **Emoji** | ⚠️ Limited (depends on font) | ✅ Via fallback font |

---

## 6. Rendering & Visuals

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **GPU Pipelines** | Quad (SDF shader) + Glyph (alpha test) + Image (textured) | Backend-dependent (OpenGL 3 reference has transforms, clip masks, filters, shaders) |
| **Backgrounds** | ✅ Solid color + image tiling with rounded clip | ✅ Solid color + decorators (gradients, images, tiled, ninepatch) |
| **Borders** | ✅ Solid/dashed/dotted, rounded corners via SDF; mixed per-side colors; double/groove/ridge/inset/outset→solid fallback | ✅ Color + width only (no style, decorator-based for effects) |
| **Rounded Corners** | ✅ Elliptical, per-corner H+V, CSS-3 corner clamping | ✅ Single-radius per corner |
| **Gradients** | ❌ | ✅ Linear, radial, conic via decorators |
| **Box Shadow** | ❌ | ✅ |
| **Filters** | ❌ | ✅ All CSS filter functions (blur, drop-shadow, etc.) |
| **Transforms** | ❌ | ✅ Full 2D/3D with interpolation |
| **Animations / Transitions** | ❌ | ✅ Full keyframe animations + property transitions |
| **Opacity** | ✅ (inherited multiplicatively, baked into color alpha) | ✅ |
| **Overflow Clipping** | ✅ Rectangular scissor + SDF rounded clipping, per-axis, stack with intersection | ✅ Rectangular clipping |
| **Decorators** | ❌ | ✅ Powerful decorator engine (image, tiled, ninepatch, gradients, shader, text) |
| **Sprite Sheets** | ❌ | ✅ High DPI sprite support |
| **Mask Images** | ❌ | ✅ |
| **Clip Masks** | ✅ Via overflow clipping + SDF | ✅ Required for transformed elements + border-radius |
| **sRGB / Linear Color** | ✅ sRGB→linear conversion | ✅ |
| **Screenshot** | ✅ F12 → PNG; `capture_to()` / `capture_rect_to()` / `screenshot_node_to()` | ❌ Not built-in |

---

## 7. Interactivity & Events

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Pointer/Mouse** | ✅ CursorMoved, MouseInput, CursorLeft, MouseWheel | ✅ MouseMove, MouseDown/Up, MouseWheel |
| **Event Bubbling** | ✅ Bubbles mousedown→up→click target→root; mouseenter/mouseleave don't bubble; keydown/keyup bubble; focusout/focusin bubble, focus/blur don't | ✅ Event propagation (capture + bubble phases) |
| **Event Types** | ✅ MouseEvent, KeyboardEvent, FocusEvent, InputEvent, WheelEvent, PointerEvent, CompositionEvent, ClipboardEvent, DragEvent, TouchEvent, AnimationEvent, TransitionEvent, SubmitEvent, FormDataEvent, ToggleEvent, ProgressEvent | ✅ Standard event types |
| **`preventDefault` / `stopPropagation`** | ❌ | ✅ |
| **Hover / Active Tracking** | ✅ Via InteractionState + `:hover`/`:active` cascade | ✅ Via pseudo-class matching (propagates to parents) |
| **Focus Tracking** | ✅ Exact-match `:focus`; Tab/Shift+Tab navigation; focusable predicate | ✅ `:focus` + `:focus-visible` + `tab-index` spatial navigation |
| **Spatial Navigation** | ❌ (only Tab order) | ✅ `nav-up/down/left/right` for controller/gamepad |
| **Keyboard** | ✅ Modifiers tracked (Ctrl/Shift/Alt/Meta); key_to_dom_key translation | ✅ KeyDown/KeyUp with modifiers |
| **Text Selection** | ✅ Drag-to-select, Ctrl+A select-all, Ctrl+C copy (arboard), word/line select (double/triple click), `user-select: none` | ❌ Not built-in |
| **Text Editing** | ✅ Full text editing for `<input>`/`<textarea>` (insert, delete, backspace, arrow keys, Home/End, Shift-select, multibyte/UTF-8, blinking caret, click-to-position, password masking, placeholder) | ✅ `<input>` / `<textarea>` element packages (browser-like behavior) |
| **Clipboard** | ✅ Ctrl+C via arboard | ✅ Via platform backends |
| **Checkbox / Radio Toggle** | ❌ (parsed but no click toggle) | ✅ |
| **Select Dropdown** | ❌ (parsed but no menu) | ✅ Full `<select>` with dropdown |
| **Form Submission** | ❌ | ⚠️ Partial (elements exist, no built-in submission) |
| **Drag & Drop** | ❌ | ✅ `<drag>` property for drag generation |
| **Touch Events** | ❌ | ✅ Via SDL platform backend |
| **Double-click / Context-menu** | ❌ | ✅ |
| **IME / Composition** | ❌ | ❌ |
| **Cursor Styling** | ❌ (property parsed, not applied) | ✅ Application-defined cursor names |
| **Event Callbacks** | ✅ `on_click`, `on_mouse_down/up/enter/leave`, `on_event` (typed DOM events) | ✅ `AddEventListener()` + inline event attributes |

---

## 8. Form Controls

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`<input type="text">`** | ✅ Full editing + placeholder + caret | ✅ |
| **`<input type="password">`** | ✅ Bullet masking | ✅ |
| **`<input type="email/search/tel/url">`** | ✅ Text editing (no validation) | ✅ |
| **`<input type="number/date/time/etc.">`** | ✅ Text editing (no validation/special UI) | ⚠️ Partial |
| **`<input type="checkbox">`** | ❌ No click-to-toggle | ✅ |
| **`<input type="radio">`** | ❌ No click-to-toggle | ✅ |
| **`<input type="range">`** | ❌ | ✅ |
| **`<input type="submit/reset/button">`** | ✅ (no form submission) | ✅ |
| **`<textarea>`** | ✅ Full editing + placeholder + line breaks | ✅ |
| **`<select>` / `<option>`** | ❌ No dropdown menu | ✅ Full dropdown |
| **`<label>`** | ✅ (click focuses associated input) | ✅ |
| **`<form>`** | ✅ (no submission) | ✅ |
| **`<button>`** | ✅ (clickable, focusable) | ✅ |
| **`<progress>`** | ❌ | ✅ |
| **Tab Navigation** | ✅ Tab/Shift+Tab | ✅ `tab-index: auto` + spatial nav |
| **Placeholder** | ✅ Shaped text, 50% alpha, centered in inputs, wrapped in textareas | ✅ |

---

## 9. Images & Media

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`<img>`** | ✅ PNG, JPEG, GIF (animated), BMP, WebP (animated); HTTP(S), data: URI, local files | ✅ Via renderer (format depends on backend; TGA built-in, SDL_image adds PNG/JPEG/etc.) |
| **`background-image`** | ✅ URL-based with tiling + rounded clip | ✅ Via decorators |
| **Image Cache** | ✅ Two-level (raw+sized), process-wide, TTL (5 min default), byte-budget eviction (256 MiB default), `Cache-Control: max-age` | ❌ No built-in image cache |
| **Async Loading** | ✅ Bounded worker pool, never blocks layout | ❌ Synchronous via render interface |
| **Animated Images** | ✅ GIF/WebP frame selection at paint time | ⚠️ Partial |
| **Image Redirects** | ✅ Up to 5, cross-scheme, exponential backoff | ❌ |
| **Preload Queue** | ✅ `tree.preload_asset(url)` via `image_cache.preload()` | ❌ |
| **`<svg>`** | ✅ Rasterized via `resvg` at declared size with cache | ✅ Via LunaSVG plugin |
| **`<lottie>`** | ❌ | ✅ Via rlottie plugin |
| **`<video>` / `<audio>`** | ❌ | ❌ |

---

## 10. Special Features

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Data Binding (MVC)** | ❌ (component framework provides state management) | ✅ Full model-view-controller: data variables, expressions, for/if/attr/value views, controllers, dirty tracking |
| **Templates** | ❌ | ✅ Template engine for consistent window design |
| **Decorator Engine** | ❌ | ✅ Powerful custom styling engine (image, tiled, ninepatch, gradients, custom shaders, text decorators) |
| **Sprite Sheets** | ❌ | ✅ High DPI sprite support |
| **Localization** | ❌ | ✅ Text translation in documents |
| **Component Framework** | ✅ Full Elm-architecture: Component trait, El builder DSL, Ctx, Store<T>, scoped CSS, keyed children, 3-path render caching, background task dispatch, App/Mount entry points | ❌ (element packages provide reusable elements) |
| **Elm Architecture** | ✅ `Component::create/update/view/props_changed/mounted/destroyed` | ❌ |
| **Reactive State (`Store<T>`)** | ✅ Arc<Mutex<T>> with subscribe() for component bridging | ❌ (data bindings provide different approach) |
| **Scoped CSS** | ✅ `Component::scope()` + `Component::styles()` | ❌ |
| **Render Caching** | ✅ 3-path: clean fast-path, skeleton patch-path, full render | ❌ |
| **Devtools** | ✅ Visual panel: component tree browser, styles inspector, breadcrumb bar | ✅ Runtime visual debugging suite |
| **Query Selector API** | ✅ Full CSS Level 4: all combinators, all attribute operators, extensive pseudo-classes | ✅ `QuerySelector()` / `QuerySelectorAll()` |
| **`querySelectorAll`** | ✅ Full CSS Level 4 selectors (child/sibling combinators, attribute selectors, :nth-child, :has(), :is(), :where(), :not(), :focus-within, etc.) | ✅ CSS2 + CSS3 selectors |
| **`matches()` / `closest()`** | ✅ | ✅ |

---

## 11. Scripting

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **JavaScript** | 🚫 Permanently out of scope | ❌ |
| **Lua** | ❌ | ✅ Full Lua plugin (element manipulation, events, data bindings, documents, contexts) |
| **Script Callbacks** | ❌ (callbacks must be Rust closures) | ✅ Via Lua scripts |

---

## 12. Component / UI Framework

| | **wgpu-html** (wgpu-html-ui) | **RmlUI** |
|---|---|---|
| **Architecture** | Elm-architecture (Model → Update → View) | Object-oriented element packages + data bindings |
| **Component Trait** | `Component<Msg, Props>` with `create/update/view/props_changed/mounted/destroyed/updated` lifecycle | C++ `Element` subclass with `OnChildAdd/OnRender/OnUpdate` |
| **Message Passing** | `MsgSender<Msg>` channel, enqueue + wake → `update()` | Data binding variables (automatic sync) |
| **View DSL** | `El` builder with 73 element constructors, `.id()`, `.class()`, `.style()`, `.child()`, `.children()`, `.text()`, `.on_click()`, `.on_event()`, etc. | RML markup + data attributes for binding |
| **Child Embedding** | ✅ Positional (`ctx.child::<C>(props)`) + keyed (`ctx.keyed_child::<C>(key, props)`) | ❌ (elements are children in RML tree directly) |
| **Content Projection** | ✅ `Children` type (cloneable Vec<El>) + named-slot patterns | ❌ |
| **Scoped CSS** | ✅ Per-component CSS via `scope()` + `styles()` | ❌ (global RCSS only) |
| **Render Optimization** | ✅ 3-path model: clean fast-path (zero work), skeleton patch-path (skips parent view()), full render | ❌ (full layout+render each frame) |
| **Async Tasks** | ✅ `ctx.spawn(|| Msg)` — OS thread → message queue | ❌ |
| **State Management** | `Store<T>` (Arc<Mutex<T>> with subscribe()) | Data model variables (dirty tracking + auto-sync) |

---

## 13. Integration & Platform

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Window Integration** | `wgpu-html-winit` with `WgpuHtmlWindow` harness (one-call setup) | Multiple backends: Win32, X11, GLFW, SFML, SDL |
| **Renderer Integration** | wgpu (built-in, no abstract interface needed) | `RenderInterface` abstract class (user implements) |
| **Font Engine Integration** | cosmic-text (built-in, no abstract interface needed) | `FontEngineInterface` (replaceable, FreeType default) |
| **System Interface** | winit-provided (window, input, DPI) | `SystemInterface` abstract class (user implements) |
| **File Interface** | Built-in (HTTP + file + data URI) | Optional (user implements for custom loading) |
| **Loop Integration** | User calls `paint_tree_returning_layout()` each frame | User calls `context->Update()` + `context->Render()` each frame |
| **egui Backend** | ✅ `wgpu-html-egui` for embedding in egui applications | ❌ |
| **DPI / High DPI** | ✅ Scale factor from winit through to glyph atlas | ✅ dp-ratio, monitor DPI awareness (Windows 10+, SDL3) |
| **Touch Support** | ❌ | ✅ Via SDL platform backend |
| **Clipboard** | ✅ arboard (read/write) | ✅ Via platform backends |
| **Pipeline Caching** | ✅ FullPipeline / PartialCascade / RepaintOnly with generation tracking | ❌ |
| **Profiling** | ✅ PipelineTimings (cascade/layout/paint/render ms), per-frame ring buffer, ProfileWindow, CPU stage timing | ❌ Not built-in |

---

## 14. Performance & Optimization

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **GPU-Accelerated** | ✅ wgpu (Vulkan/Metal/DX12) | ✅ Via backends (OpenGL, Vulkan, DX12, etc.) |
| **Pipeline Caching** | ✅ Classifies frame to skip cascade/layout/paint when state unchanged | ❌ |
| **Incremental Cascade** | ✅ Only dirty paths re-processed; `paint_only_pseudo_rules` flag for O(n) color-only patch | ❌ |
| **Text Shaping** | HarfBuzz-based via cosmic-text | FreeType basic; optional HarfBuzz |
| **Glyph Atlas** | ✅ Shelf-packing CPU→GPU texture | ✅ GPU texture via FreeType |
| **Continuous Redraw** | ✅ `request_redraw` loop; hover changes throttled to 16ms | ❌ User-controlled update loop |
| **Parallelism** | ✅ Async image loading (worker pool); cascade/layout/paint serial on main thread | Serial on main thread |
| **Binary Size** | Heavier (wgpu + winit + cosmic-text + dependencies) | Light (FreeType only core dependency; ~1 MB) |
| **Compile Time** | Slower (Rust + many crates) | Fast (C++ CMake) |

---

## 15. Maturity & Ecosystem

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Production Use** | ❌ Too new | ✅ Commercial games: The Thing: Remastered, Killing Time: Resurrected, ROSE Online, Unvanquished; tools: Alchemist (Cfx.re/Rockstar), Gearmulator, alt:V, TruckersMP, WOTInspector |
| **Test Coverage** | ✅ Inline Rust unit tests (HTML/CSS in test code) | ✅ Test suite in Tests/ directory |
| **Documentation** | ✅ AGENTS.md, docs/full-status.md, spec/*.md | ✅ Full documentation site, C++ manual, Lua manual |
| **Community** | Very small (new project) | Active (Zulip chat, GitHub discussions, ~4k stars) |
| **Breaking Changes** | Frequent (active development) | Stable API, semantic versioning |
| **Plugin Ecosystem** | ❌ | ✅ Lua, LunaSVG, rlottie plugins |
| **Sample Demos** | Single demo (flex-browser-like.html) | Many samples: invaders, demo, benchmark, animation, effects, transform, drag, treeview, etc. |

---

## 16. Quick Summary — Strengths & Weaknesses

### wgpu-html Strengths
- **Rust-native** — memory safety, no FFI, Cargo integration
- **CSS Grid** — RmlUI has no grid layout
- **CSS Custom Properties** (`var()`) plus `calc()`/`min()`/`max()`/`clamp()`
- **Full CSS Flexbox Level 1** (matches RmlUI)
- **Rich component framework** — Elm-architecture with reactive state, scoped CSS, render caching
- **Built-in image pipeline** — async loading, animated GIF/WebP, cache with TTL/eviction, preload
- **Screenshot API** — single element or viewport, programmatic PNG export
- **Text selection + editing** — not in RmlUI
- **Embeddable in egui**
- **Pipeline caching + incremental cascade** — skips expensive stages when state unchanged

### wgpu-html Weaknesses
- **No floats** — cannot wrap text around images or sidebars
- **No table layout** — tables degrade to block flow
- **No animations/transitions/transforms** — elements are static
- **No gradients or box-shadows** — flat visual design only
- **No decorator system** — harder to skin elements beyond CSS
- **Limited CSS selector support in cascade** — only descendant combinator, no attribute selectors or structural pseudo-classes in stylesheet parser (note: the `querySelector` API has full CSS Level 4 selector support)
- **No Lua/scripting** — by design, but limits customization
- **No data binding** — component framework is different paradigm
- **Immature** — not production-tested

### RmlUI Strengths
- **Production-proven** — 4000+ stars, shipped in commercial games
- **Full animations, transitions, transforms** — rich visual possibilities
- **Decorator engine** — powerful custom skinning with shaders, gradients, ninepatch, sprite sheets
- **Floats + Clear + Table layout** — full CSS2 print-like layouts
- **All CSS selectors in cascade** — combinators, attributes, structural pseudo-classes
- **Lua scripting** — runtime UI logic without recompilation
- **Data binding** — model-view-controller for automatic data↔UI sync
- **Lightweight** — FreeType only core dependency
- **Cross-platform** — Windows, Linux, macOS, Android, iOS, Switch
- **Template + Localization** — built-in
- **Spatial navigation** — controller/gamepad input
- **Backend flexibility** — OpenGL 2/3, Vulkan, DX12, SDL, Metal

### RmlUI Weaknesses
- **C++ only** — no Rust integration without FFI
- **No CSS Grid**
- **No custom properties / calc() / var()**
- **No text selection API**
- **No component framework** (element packages are simpler)
- **Image support depends on backend** (most only support uncompressed TGA by default)
- **No screenshot / off-screen capture API**
- **No pipeline caching** — full layout+render every frame
- **No built-in UA stylesheet** — must include `rml.rcss` manually for standard element behavior

---

## 17. Verdict — Viability as Rust Alternative

wgpu-html is **already a viable Rust alternative** for many use cases, particularly:
- UI that needs **CSS Grid** or **custom properties**
- **Static or mostly-static layouts** with rich Rust-driven interactivity
- Applications that want **Rust-native**, single-binary deployment
- **Text-heavy interfaces** with selection/clipboard
- Embedding inside **egui** applications

wgpu-html needs work to match RmlUI on:
- **Animations & transforms** — critical for game UI polish
- **Float layout** — essential for document-like layouts
- **Table layout** — important for data display
- **Gradients & box-shadows** — modern visual design
- **Decorator system** — flexibility beyond CSS properties
- **Production hardening** — real-world testing and optimization

The two projects have **different design philosophies**: RmlUI emphasizes visual richness (animations, transforms, decorators, filters) while wgpu-html emphasizes web-standards fidelity (grid, custom properties, calc(), full selector engine for DOM queries, text editing/selection).

