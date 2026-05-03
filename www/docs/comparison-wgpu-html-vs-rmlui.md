---
title: wgpu-html vs RmlUI
---

# wgpu-html vs RmlUI — Capability Comparison

> **Date:** 2026-05-03
> **Purpose:** Comparison of two GPU-accelerated HTML/CSS UI rendering engines aimed at game/application UI. wgpu-html is a Rust alternative to the C++ RmlUI library.

## Project Overview

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Language** | Rust | C++ (C++17) |
| **License** | MIT | MIT |
| **Architecture** | Parser → Style Cascade → Layout → Paint → Renderer (5-stage pipeline) | Document → Style → Layout → Render (traditional UI engine) |
| **GPU Backend** | wgpu (Vulkan/Metal/DX12 via `wgpu::Backends::PRIMARY`) | OpenGL 2/3, Vulkan, SDL GPU, SDLrenderer, DirectX 12 (all optional backends) |
| **Rendering Model** | Display-list with draw commands (Quads + Glyphs + Images + Clip ranges) | Geometry generation (vertices, indices, textures) via `RenderInterface` |
| **Font Engine** | cosmic-text + CPU rasterization to glyph atlas | FreeType (replaceable via custom `FontEngineInterface`) |
| **External Dependencies** | wgpu, winit, cosmic-text, resvg, image, arboard | FreeType only (core). Optional: Lua, LunaSVG, rlottie, HarfBuzz |
| **Target Use Case** | Application UI, game HUD/menus, desktop tools | Game UI, embedded applications, installers, game menus |
| **First Release** | 2025–2026 (active development) | 2008 (as libRocket), forked as RmlUi in 2019 |
| **Maturity** | Alpha/beta — core pipeline works, some gaps | Production-grade — used in commercial games |
| **Platforms** | Windows, Linux, macOS (winit + wgpu) | Windows, Linux, macOS, Android, iOS, Switch, Emscripten |
| **JavaScript** | 🚫 Permanently out of scope | Not native; Lua plugin available |

## Markup Language

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Format** | Standard HTML5 (`.html`) | RML (XHTML1-like `.rml`) |
| **Element Count** | ~100 typed HTML element structs | ~20 custom elements + `<script>`/`<lottie>`/`<svg>` via plugins |
| **HTML5 Semantic Tags** | All major elements: `<div>`, `<span>`, `<p>`, `<h1>`–`<h6>`, `<img>`, `<a>`, `<button>`, `<input>` (22 types), `<form>`, `<textarea>`, `<select>`, `<table>`, `<ul>`/`<ol>`/`<li>`, etc. | Functional tags: `<body>`, `<div>`, `<span>`, `<p>`, `<h1>`–`<h6>`, `<br>`, `<img>`, `<a>`, `<input>`, `<textarea>`, `<select>`, `<option>`, `<label>`, `<form>` + custom |
| **HTML Serialization** | ✅ `Node::to_html()`, `Tree::to_html()` | ✅ InnerRML get/set |
| **Entity Decoding** | ✅ `&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, `&nbsp;`, `&#NN;`, `&#xNN;` | ✅ XML entities |
| **Global Attributes** | `id`, `class`, `style`, `title`, `lang`, `hidden`, `tabindex`, `contenteditable`, `draggable`, `dir`, `accesskey`, `spellcheck`, `translate`, `role`, `aria-*`, `data-*` | `id`, `class`, `style`, `data-*` |

## CSS / RCSS Support

### Selectors

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Tag, `#id`, `.class`, universal `*`** | ✅ | ✅ |
| **Descendant ` `** | ✅ | ✅ |
| **Comma-list `A, B`** | ✅ | ✅ |
| **Child `>`** | ❌ cascade only (✅ query engine) | ✅ |
| **Next-sibling `+`** | ❌ cascade only (✅ query engine) | ✅ |
| **Subsequent-sibling `~`** | ❌ cascade only (✅ query engine) | ✅ |
| **Attribute selectors** | ❌ cascade only (✅ query engine) | ✅ |
| **`:hover` / `:active` / `:focus`** | ✅ cascade; `:focus` exact-match only | ✅ (+ propagates to parents) |
| **`:focus-visible`** | ❌ cascade only (✅ query engine) | ✅ |
| **`:checked`, `:disabled`/`:enabled`** | ❌ cascade only (✅ query engine) | ✅ (partial) |
| **`:first-child`/`:last-child`/`:only-child`** | ❌ cascade only (✅ query engine) | ✅ |
| **`:nth-child()`/`:nth-last-child()`** | ❌ cascade only (✅ query engine) | ✅ |
| **`:nth-of-type()`/`:nth-last-of-type()`** | ❌ cascade only (✅ query engine) | ✅ |
| **`:not()`** | ❌ cascade only (✅ query engine) | ✅ |
| **`:is()` / `:where()` / `:has()`** | ❌ cascade only (✅ query engine) | ❌ |
| **`:empty`** | ❌ | ✅ |
| **`:placeholder-shown`** | ❌ cascade only (✅ query engine) | ✅ |
| **Pseudo-elements** | ❌ | ❌ |
| **`!important`** | ✅ (4-band cascade) | ✅ (2-band cascade) |
| **Specificity** | `(id<<16) \| (class<<8) \| tag` | Standard CSS specificity (a,b,c) |

### Cascade System

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **UA stylesheet** | ✅ (~50 rules) | ❌ (recommended stylesheet available separately) |
| **Author stylesheet** | ✅ (`<style>` blocks + `<link>` parsed) | ✅ (`<link type="text/rcss">` + inline) |
| **Inline style** | ✅ (`style="..."` attribute) | ✅ |
| **Inheritance** | ✅ (color, font-*, text-*, visibility, cursor, pointer-events, user-select) | ✅ |
| **`inherit` / `initial` / `unset`** | ✅ | ✅ |
| **`@media` queries** | ✅ (width/height/orientation with min/max + `not` prefix) | ✅ (width, height, orientation, resolution, prefers-color-scheme, prefers-reduced-motion) |
| **Dynamic pseudo recalc** | ✅ Incremental re-cascade of dirty paths only | ✅ |
| **`<link rel="stylesheet">`** | ⚠️ Parsed but no HTTP fetch | ✅ File-based loading |

### CSS Properties Summary

| Property Area | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`display`** | block, inline, inline-block, flex, grid, none | inline, block, inline-block, flow-root, flex, inline-flex, table, inline-table, table-row-group, table-row, table-column-group, table-column, table-cell, none |
| **`position`** | static, relative, absolute, fixed | static, relative, absolute, fixed |
| **Box Model** | width/height, min-/max-, margin, padding, box-sizing, border | width/height, min-/max-, margin, padding, box-sizing, border |
| **`float`** | ❌ Not parsed | ✅ left, right, none |
| **`clear`** | ❌ | ✅ |
| **`border-radius`** | ✅ Elliptical, per-corner H+V, CSS-3 corner clamping | ✅ Single-radius per corner only |
| **`background-clip`** | ✅ border-box/padding-box/content-box | ❌ Uses decorators instead |
| **`background-image`** | ✅ URL + tiling + rounded-clip | ❌ Uses decorators instead |
| **`border-style`** | solid/dashed/dotted/none/hidden; double/groove/ridge/inset/outset→solid fallback | ❌ Not a property (only width+color) |
| **`box-shadow`** | ❌ Parsed as raw string, never consumed | ✅ |
| **Gradients** | ❌ Parsed as raw string, skipped | ✅ Linear, radial, conic gradients via decorators |
| **`opacity`** | ✅ (inherited multiplicatively) | ✅ |
| **`visibility`** | ✅ | ✅ |
| **`z-index`** | ❌ Parsed, not consumed (tree DFS paint order) | ✅ |
| **`overflow` / `overflow-x` / `overflow-y`** | ✅ hidden/scroll/auto/visible | ✅ hidden/scroll/auto/visible |
| **`pointer-events`** | ✅ auto/none | ✅ auto/none |
| **`user-select`** | ✅ auto/none/text/all | ❌ Not applicable |
| **`cursor`** | ✅ Parsed, not applied to OS cursor | ✅ (application-defined cursor names) |
| **`transform`** | ❌ Parsed as raw string only | ✅ Full 2D/3D transforms with interpolation |
| **`transition`** | ❌ Parsed as raw string only | ✅ Full transitions with tweening |
| **`animation` / `@keyframes`** | ❌ Not parsed | ✅ Full keyframe animations |
| **`filter`** | ❌ Not parsed | ✅ All CSS filter functions |
| **`backdrop-filter`** | ❌ | ✅ |
| **`mask-image`** | ❌ | ✅ |
| **`text-overflow`** | ❌ | ✅ clip/ellipsis/custom string |
| **`white-space`** | ✅ normal/pre | ✅ normal/pre/nowrap/pre-wrap/pre-line |
| **`word-break`** | ❌ | ✅ normal/break-all/break-word |
| **Custom Properties `--foo`** | ✅ Full `var()` with inheritance + cycle detection | ❌ |
| **`calc()` / `min()` / `max()` / `clamp()`** | ✅ Full 18-node AST + evaluation | ⚠️ Limited `calc()` support |

## Layout Engines

| Layout Model | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Block Flow** | ✅ Full | ✅ Full |
| **Inline Formatting Context** | ✅ Line-box layout, word wrapping, text-align, inline-block | ✅ Line-box layout, word wrapping, text-align |
| **Flexbox** | ✅ Complete CSS Flexbox Level 1 | ✅ flex-direction, flex-wrap, justify-content, align-items, align-content, align-self, flex-grow/shrink/basis, gap |
| **CSS Grid** | ✅ Full grid with fr units, minmax(), repeat(), auto-placement | ❌ Not supported |
| **Float Layout** | ❌ Not parsed | ✅ left/right floats with inline content wrap-around |
| **Clear** | ❌ | ✅ |
| **Table Layout** | ❌ Display values parsed, fall through to block | ✅ Full table model |
| **Positioned Layout** | ✅ absolute/relative/fixed (sticky→relative) | ✅ static/relative/absolute/fixed |
| **Hit Testing** | ✅ `LayoutBox::hit_path()` (deepest-first, topmost wins) | ✅ Element-based hit testing |
| **`z-index` Stacking** | ❌ Tree DFS paint order only | ✅ Full stacking contexts |

## Text & Typography

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Font Engine** | cosmic-text + CPU rasterization → glyph atlas | FreeType (replaceable) |
| **Text Shaping** | cosmic-text (HarfBuzz-based) | FreeType kerning; HarfBuzz optional |
| **`font-family`** | ✅ Comma-separated fallback list, generic keywords (sans-serif, serif, monospace, cursive, fantasy, system-ui, ui-*), quoted names | ✅ Single family only |
| **`font-weight`** | ✅ 100–900, normal/bold/bolder/lighter | ✅ normal/bold/1–1000 |
| **`font-style`** | ✅ normal/italic/oblique | ✅ normal/italic |
| **`font-size`** | ✅ All length units + calc() | ✅ Length/percentage |
| **`line-height`** | ✅ Length units + number (default 1.25) | ✅ Number/length (default 1.2) |
| **`letter-spacing`** | ✅ Post-shape per-glyph offset | ✅ |
| **`text-transform`** | ✅ uppercase/lowercase/capitalize (pre-shape) | ✅ |
| **`text-align`** | ✅ left/right/center/start/end (no justify) | ✅ left/right/center |
| **`text-decoration`** | ✅ underline/line-through/overline | ✅ underline/overline/line-through |
| **System Font Discovery** | ✅ Windows/Linux/macOS via `system_font_variants()` | ❌ (manual registration required) |
| **Font Effects** | ❌ | ✅ glow, outline, shadow, blur |

## Rendering & Visuals

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **GPU Pipelines** | Quad (SDF) + Glyph (alpha test) + Image (textured) | Backend-dependent |
| **Backgrounds** | ✅ Solid color + image tiling with rounded clip | ✅ Solid color + decorators |
| **Borders** | ✅ Solid/dashed/dotted, rounded corners via SDF; mixed per-side colors | ✅ Color + width only |
| **Rounded Corners** | ✅ Elliptical, per-corner H+V, CSS-3 corner clamping | ✅ Single-radius per corner |
| **Gradients** | ❌ | ✅ Linear, radial, conic via decorators |
| **Box Shadow** | ❌ | ✅ |
| **Filters** | ❌ | ✅ All CSS filter functions |
| **Transforms** | ❌ | ✅ Full 2D/3D with interpolation |
| **Animations / Transitions** | ❌ | ✅ |
| **Opacity** | ✅ (inherited multiplicatively) | ✅ |
| **Overflow Clipping** | ✅ Rectangular scissor + SDF rounded clipping, per-axis, nested intersection | ✅ Rectangular clipping |
| **Decorators** | ❌ | ✅ Powerful decorator engine |
| **Sprite Sheets** | ❌ | ✅ |
| **Screenshot** | ✅ F12 → PNG; `capture_to()` / `capture_rect_to()` / `screenshot_node_to()` | ❌ Not built-in |

## Interactivity & Events

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Pointer/Mouse** | ✅ CursorMoved, MouseInput, CursorLeft, MouseWheel | ✅ MouseMove, MouseDown/Up, MouseWheel |
| **Event Bubbling** | ✅ Bubbles mousedown→up→click target→root; mouseenter/mouseleave don't bubble; keydown/keyup bubble; focusout/focusin bubble, focus/blur don't | ✅ Event propagation (capture + bubble phases) |
| **Event Types** | ✅ MouseEvent, KeyboardEvent, FocusEvent, InputEvent, WheelEvent, PointerEvent, CompositionEvent, ClipboardEvent, DragEvent, TouchEvent, AnimationEvent, TransitionEvent, SubmitEvent, FormDataEvent, ToggleEvent, ProgressEvent | ✅ Standard event types |
| **`preventDefault` / `stopPropagation`** | ❌ | ✅ |
| **Hover / Active Tracking** | ✅ via `InteractionState` + `:hover`/`:active` cascade | ✅ via pseudo-class matching (propagates to parents) |
| **Focus Tracking** | ✅ Exact-match `:focus`; Tab/Shift+Tab navigation; focusable predicate | ✅ `:focus` + `:focus-visible` + `tab-index` spatial navigation |
| **Spatial Navigation** | ❌ (only Tab order) | ✅ `nav-up/down/left/right` for controller/gamepad |
| **Keyboard** | ✅ Modifiers tracked (Ctrl/Shift/Alt/Meta); key_to_dom_key translation | ✅ KeyDown/KeyUp with modifiers |
| **Text Selection** | ✅ Drag-to-select, Ctrl+A select-all, Ctrl+C copy, word/line select, `user-select: none` | ❌ Not built-in |
| **Text Editing** | ✅ Full text editing for `<input>`/`<textarea>` (insert, delete, backspace, arrow keys, Home/End, Shift-select, multibyte/UTF-8, blinking caret, click-to-position, password masking, placeholder) | ✅ `<input>` / `<textarea>` element packages |
| **Clipboard** | ✅ Ctrl+C via arboard | ✅ Via platform backends |
| **Checkbox / Radio Toggle** | ❌ (parsed but no click toggle) | ✅ |
| **Select Dropdown** | ❌ (parsed but no menu) | ✅ |
| **Form Submission** | ❌ | ⚠️ Partial |
| **Drag & Drop** | ❌ | ✅ |
| **Touch Events** | ❌ | ✅ Via SDL platform backend |
| **Double-click / Context-menu** | ❌ | ✅ |
| **Cursor Styling** | ❌ (property parsed, not applied) | ✅ |
| **Event Callbacks** | ✅ `on_click`, `on_mouse_down/up/enter/leave`, `on_event` (typed DOM events) | ✅ `AddEventListener()` + inline event attributes |

## Form Controls

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`<input type="text">`** | ✅ Full editing + placeholder + caret | ✅ |
| **`<input type="password">`** | ✅ Bullet masking | ✅ |
| **`<input type="email/search/tel/url">`** | ✅ Text editing (no validation) | ✅ |
| **`<input type="checkbox">`** | ❌ No click-to-toggle | ✅ |
| **`<input type="radio">`** | ❌ No click-to-toggle | ✅ |
| **`<input type="range">`** | ❌ | ✅ |
| **`<textarea>`** | ✅ Full editing + placeholder + line breaks | ✅ |
| **`<select>` / `<option>`** | ❌ No dropdown menu | ✅ Full dropdown |
| **`<label>`** | ✅ (click focuses associated input) | ✅ |
| **`<button>`** | ✅ (clickable, focusable) | ✅ |
| **Tab Navigation** | ✅ Tab/Shift+Tab | ✅ `tab-index: auto` + spatial nav |

## Images & Media

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **`<img>`** | ✅ PNG, JPEG, GIF (animated), BMP, WebP (animated); HTTP(S), data: URI, local files | ✅ Via renderer (format depends on backend) |
| **Image Cache** | ✅ Two-level (raw+sized), process-wide, TTL (5 min default), byte-budget eviction (256 MiB default), `Cache-Control: max-age` | ❌ No built-in image cache |
| **Async Loading** | ✅ Bounded worker pool, never blocks layout | ❌ Synchronous via render interface |
| **Animated Images** | ✅ GIF/WebP frame selection at paint time | ⚠️ Partial |
| **Image Redirects** | ✅ Up to 5, cross-scheme, exponential backoff | ❌ |
| **Preload Queue** | ✅ `tree.preload_asset(url)` | ❌ |
| **`<svg>`** | ✅ Rasterized via `resvg` at declared size with cache | ✅ Via LunaSVG plugin |
| **`<lottie>`** | ❌ | ✅ Via rlottie plugin |

## Special Features

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Data Binding (MVC)** | ❌ (component framework provides state management) | ✅ Full model-view-controller |
| **Templates** | ❌ | ✅ Template engine |
| **Decorator Engine** | ❌ | ✅ Powerful custom styling engine |
| **Localization** | ❌ | ✅ Text translation in documents |
| **Component Framework** | ✅ Full Elm-architecture: Component trait, El builder DSL, Ctx, Store\<T\>, scoped CSS, keyed children, 3-path render caching, background task dispatch, App/Mount entry points | ❌ (element packages provide reusable elements) |
| **Reactive State (`Store<T>`)** | ✅ Arc\<Mutex\<T\>\> with subscribe() for component bridging | ❌ |
| **Scoped CSS** | ✅ `Component::scope()` + `Component::styles()` | ❌ |
| **Render Caching** | ✅ 3-path: clean fast-path, skeleton patch-path, full render | ❌ |
| **Devtools** | ✅ Visual panel: component tree browser, styles inspector, breadcrumb bar | ✅ Runtime visual debugging suite |
| **Query Selector API** | ✅ Full CSS Level 4: all combinators, all attribute operators, extensive pseudo-classes | ✅ `QuerySelector()` / `QuerySelectorAll()` |

## Integration & Platform

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **Window Integration** | `wgpu-html-winit` with `WgpuHtmlWindow` harness (one-call setup) | Multiple backends: Win32, X11, GLFW, SFML, SDL |
| **Renderer Integration** | wgpu (built-in, no abstract interface needed) | `RenderInterface` abstract class (user implements) |
| **Font Engine Integration** | cosmic-text (built-in) | `FontEngineInterface` (replaceable) |
| **System Interface** | winit-provided (window, input, DPI) | `SystemInterface` abstract class |
| **File Interface** | Built-in (HTTP + file + data URI) | Optional (user implements) |
| **Loop Integration** | User calls `paint_tree_returning_layout()` each frame | User calls `context->Update()` + `context->Render()` each frame |
| **egui Backend** | ✅ `wgpu-html-egui` for embedding in egui applications | ❌ |
| **DPI / High DPI** | ✅ Scale factor from winit through to glyph atlas | ✅ dp-ratio, monitor DPI awareness |
| **Clipboard** | ✅ arboard (read/write) | ✅ Via platform backends |
| **Pipeline Caching** | ✅ FullPipeline / PartialCascade / RepaintOnly with generation tracking | ❌ |
| **Profiling** | ✅ PipelineTimings (cascade/layout/paint/render ms), per-frame ring buffer, ProfileWindow | ❌ Not built-in |

## Performance & Optimization

| | **wgpu-html** | **RmlUI** |
|---|---|---|
| **GPU-Accelerated** | ✅ wgpu (Vulkan/Metal/DX12) | ✅ Via backends (OpenGL, Vulkan, DX12, etc.) |
| **Pipeline Caching** | ✅ Classifies frame to skip cascade/layout/paint when state unchanged | ❌ |
| **Incremental Cascade** | ✅ Only dirty paths re-processed | ❌ |
| **Text Shaping** | HarfBuzz-based via cosmic-text | FreeType basic; optional HarfBuzz |
| **Glyph Atlas** | ✅ Shelf-packing CPU→GPU texture | ✅ GPU texture via FreeType |
| **Continuous Redraw** | ✅ `request_redraw` loop; hover changes throttled to 16ms | ❌ User-controlled update loop |
| **Binary Size** | Heavier (wgpu + winit + cosmic-text + dependencies) | Light (FreeType only core dependency; ~1 MB) |
| **Compile Time** | Slower (Rust + many crates) | Fast (C++ CMake) |

## Quick Summary — Strengths & Weaknesses

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
- **Limited CSS selector support in cascade** — only descendant combinator, no attribute selectors or structural pseudo-classes
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
- **No built-in UA stylesheet** — must include `rml.rcss` manually

## Verdict — Viability as Rust Alternative

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
