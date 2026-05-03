# HTML5 / CSS Implementation Plan

> **Date:** 2026-05-02
> **Goal:** Close remaining spec gaps for HTML5 parsing, CSS selectors/layout/paint, and form controls.
> **Scope:** GPU-accelerated HTML/CSS via `wgpu`. No JavaScript.

---

## P0 — Rendering Gaps (visual difference from browsers)

These directly affect visual output — wrong or missing paint means the page looks broken.

### P0-1: Per-glyph clipping fix
Lines changed: ~15 in `crates/wgpu-html/src/paint.rs`

| Item | Status |
|---|---|
| Clamp glyph width when glyph extends past text box `box_right` | ❌ |
| Adjust UV coordinates for clipped portion | ❌ |
| Test: `tree_row_painted_glyphs_dont_overlap` (display-list level) | ❌ |

**Why:** Glyphs that start inside a shrunken flex item's `content_rect` but extend past it bleed into the next item. The current check `glyph_x >= box_right` only catches glyphs whose *start* is past the edge.

**Depends on:** nothing — paint.rs change only.

---

### P0-2: `z-index` / stacking contexts
Lines changed: ~200 spread across layout + paint

| Item | Status |
|---|---|
| Add `z_index: Option<i32>` to `LayoutBox` | ❌ |
| Populate from `style.z_index` during layout | ❌ |
| Identify stacking contexts (positioned + z-index ≠ auto, opacity < 1, transforms) | ❌ |
| Sort paint children by z-order before emitting quads/glyphs | ❌ |
| Preserve document order for hit-testing (AGENTS.md invariant) | ❌ |

**Why:** Overlapping positioned or flex/grid items with different `z-index` paint in wrong order. Everything is currently tree-DFS order.

**Depends on:** nothing — layout + paint change.

---

### P0-3: `border-color: currentColor` and default border color
Lines changed: ~10 in `crates/wgpu-html-layout/src/lib.rs`

| Item | Status |
|---|---|
| When `border-color` is not set, fall back to `color` (computed `currentColor`) | ❌ |
| Make `border-color` resolve to `color` value when no explicit border color is present | ❌ |

**Why:** `border: 1px solid` without an explicit color renders invisible instead of using the text color.

**Depends on:** `currentColor` resolution (already partially done — style has `color` field).

---

## P1 — Layout Gaps (missing layout modes)

### P1-1: Float layout (`float: left/right`, `clear`)
Lines changed: ~500 new file `crates/wgpu-html-layout/src/float.rs`

| Item | Status |
|---|---|
| Add `Float` enum to `css_enums.rs` | ❌ |
| Add `float` and `clear` fields to `Style` | ❌ |
| Parse `float` and `clear` in CSS parser | ❌ |
| Implement float placement (shrink-to-fit, line-box shortening) | ❌ |
| Implement `clear` (none/left/right/both) | ❌ |
| Float-aware hit testing | ❌ |

**Why:** Many legacy layouts use floats. Common in HTML emails and older sites.

**Depends on:** nothing — new layout module.

---

### P1-2: Table layout (`display: table`, `table-row`, `table-cell`)
Lines changed: ~800 new file `crates/wgpu-html-layout/src/table.rs`

| Item | Status |
|---|---|
| Anonymous table box generation (auto-wrap orphans in `table`/`table-row`/`table-cell`) | ❌ |
| Table width algorithm (fixed vs auto layout) | ❌ |
| Column width distribution | ❌ |
| Row height computation | ❌ |
| `colspan` / `rowspan` | ❌ |
| `vertical-align` on table cells | ❌ |
| Table border collapsing (`border-collapse`) | ❌ |

**Why:** `display: table` is parsed but falls through to block layout. Tables are common for data display.

**Depends on:** nothing — new layout module. But large scope (CSS Table spec is complex).

---

### P1-3: Sticky positioning
Lines changed: ~50 in `crates/wgpu-html-layout/src/lib.rs`

| Item | Status |
|---|---|
| Detect nearest scroll ancestor during layout | ❌ |
| Clamp sticky element to scroll-container edges | ❌ |
| Handle `top`/`bottom`/`left`/`right` sticky offsets | ❌ |

**Why:** `position: sticky` is parsed but degraded to `relative`. Common in modern layouts (sticky headers).

**Depends on:** scroll container tracking (already partially done).

---

## P2 — CSS Selector & Cascade Gaps

### P2-1: Stylesheet selectors — combinators + pseudo-classes
Lines changed: ~200 in `crates/wgpu-html-style/src/`

| Item | Status |
|---|---|
| Child combinator `>` in stylesheet parser + matcher | ❌ |
| Next-sibling combinator `+` in stylesheet parser + matcher | ❌ |
| Subsequent-sibling combinator `~` in stylesheet parser + matcher | ❌ |
| Attribute selectors `[attr]`, `[attr=val]`, `[attr~=val]`, etc. in stylesheet parser + matcher | ❌ |
| `:first-child`, `:last-child`, `:nth-child()` structural pseudo-classes in cascade | ❌ |
| `:not()` negation pseudo-class in cascade | ❌ |

**Why:** The query engine (`wgpu-html-tree/src/query.rs`) already implements ALL of these for `querySelector`/`matches`/`closest`. The stylesheet parser and cascade matcher need to be updated to use the same machinery. This is largely a wiring task, not a rewrite.

**Depends on:** none — the parsing/matching logic already exists in the query engine.

---

### P2-2: `:focus-visible`, `:focus-within`, `:disabled` in cascade
Lines changed: ~50 in `crates/wgpu-html-style/src/`

| Item | Status |
|---|---|
| `:focus-within` — match element if any descendant is focused | ❌ |
| `:focus-visible` — match element if focus was from keyboard navigation | ❌ |
| `:disabled` — match form elements with `disabled` attribute | ❌ |

**Why:** These pseudo-classes work in the query engine but not in cascade selector matching. Needed for `:focus-within` containment highlighting and disabled button styling.

**Depends on:** P2-1 (same wiring pattern).

---

### P2-3: At-rules — `@media` support
Lines changed: ~100 in `crates/wgpu-html-style/src/`, ~100 in `crates/wgpu-html-parser/src/`

| Item | Status |
|---|---|
| Parse `@media (condition) { rules }` blocks | ✅ |
| Evaluate media queries (`width`, `height`, `prefers-color-scheme`, etc.) | ✅ (width/height/orientation + not) |
| Gate rules on media query match during cascade | ✅ |

**Status:** ✅ Done. `@media screen and (min-width: 600px) { … }` fully parsed, evaluated, and applied during cascade. Supports min-width, max-width, min-height, max-height, orientation (portrait/landscape), and `not` prefix.

---

## P3 — Form Control Gaps

### P3-1: Checkbox / radio click-to-toggle
Lines changed: ~40 in `crates/wgpu-html-tree/src/dispatch.rs`

| Item | Status |
|---|---|
| On primary-button click on `<input type="checkbox">`, flip `checked` | ❌ |
| On Space key on focused checkbox, flip `checked` | ❌ |
| On click on `<input type="radio">`, set `checked`, uncheck other radios in same `name` group | ❌ |
| Increment `generation` and fire `InputEvent` | ❌ |

**Why:** Checkboxes and radios are read-only despite having `checked` parsed and `:checked` working.

**Depends on:** nothing — dispatch.rs change only.

---

### P3-2: `<select>` dropdown
Lines changed: ~500 across tree + layout + paint

| Item | Status |
|---|---|
| Render dropdown option list on click | ❌ |
| Click-to-select an `<option>` | ❌ |
| Update `OptionElement.selected` on selection | ❌ |
| Keyboard navigation (ArrowUp/Down, Enter) | ❌ |
| `multiple` attribute support | ❌ |

**Why:** `<select>` is focusable but has zero interaction behavior.

**Depends on:** P0-2 (`z-index` for overlay dropdown).

---

### P3-3: Form submission
Lines changed: ~80 in `crates/wgpu-html-tree/src/dispatch.rs`

| Item | Status |
|---|---|
| `Enter` in focused `<input>`/`<textarea>` synthesizes `SubmitEvent` | ❌ |
| Click on `<button type="submit">` synthesizes `SubmitEvent` | ❌ |
| Collect form data from named controls | ❌ |

**Why:** Forms are non-functional without submission.

**Depends on:** nothing — dispatch.rs change only.

---

## P4 — Remaining Rendering Gaps

### P4-1: Gradients (`linear-gradient`, `radial-gradient`, `conic-gradient`)
Lines changed: ~400 in `crates/wgpu-html-renderer/src/` (new gradient pipeline or quad shader extension)

| Item | Status |
|---|---|
| Parse gradient color stops and directions into typed structs | ❌ |
| Layout: precompute gradient tile rects (like `background-image` tiles) | ❌ |
| Render: GPU gradient via SDF or procedural shader | ❌ |

**Why:** Gradients are extremely common in modern CSS. Currently parsed as opaque `CssImage::Function(String)` and skipped.

**Depends on:** nothing — new renderer feature.

---

### P4-2: `box-shadow`
Lines changed: ~200 in `crates/wgpu-html-renderer/src/`

| Item | Status |
|---|---|
| Parse `box-shadow` into typed struct (offsets, blur, spread, color, inset) | ❌ |
| Render shadow quads (blur via multi-pass or SDF approximation) | ❌ |
| `inset` shadows | ❌ |

**Why:** Common decorative CSS property. Currently parsed as raw string, not consumed.

**Depends on:** nothing — new renderer feature.

---

### P4-3: `transform` (2D)
Lines changed: ~150 in `crates/wgpu-html-layout/src/`, ~50 in `crates/wgpu-html-renderer/src/`

| Item | Status |
|---|---|
| Parse `transform` into typed matrix/function list | ❌ |
| Apply transform in layout (affects hit-testing and containing blocks) | ❌ |
| Apply transform in paint (translate quads) or GPU (uniform matrix) | ❌ |
| `transform-origin` | ❌ |

**Why:** Common for animations and UI effects. Currently parsed as raw string, never consumed.

**Depends on:** GPU uniform for transform matrix (renderer change).

---

### P4-4: `filter` property
Lines changed: ~50 parser, ~100 renderer

| Item | Status |
|---|---|
| Parse `filter` functions (`blur()`, `brightness()`, etc.) | ❌ |
| Apply via GPU post-processing or multi-pass | ❌ |

**Why:** Currently silently dropped by parser.

**Depends on:** nothing — parser + renderer change.

---

## P5 — HTML Parsing Gaps

### P5-1: Unknown tags — keep subtree
Lines changed: ~20 in `crates/wgpu-html-parser/src/tree_builder.rs`

| Item | Status |
|---|---|
| Instead of dropping unknown tag subtrees, keep them as generic elements | ❌ |

**Why:** Web components and custom elements rely on unknown tags being preserved. Currently the entire subtree is silently dropped.

**Depends on:** nothing.

---

### P5-2: Whitespace-only text preservation
Lines changed: ~20 in `crates/wgpu-html-parser/src/tree_builder.rs`

| Item | Status |
|---|---|
| Keep whitespace text nodes between inline elements (not between blocks) | ❌ |

**Why:** Whitespace between `<span>` elements affects inline layout spacing. Currently all whitespace-only text is dropped.

**Depends on:** nothing.

---

### P5-3: `<link rel="stylesheet">` loading
Lines changed: ~100 in `crates/wgpu-html-tree/src/`, ~50 in `crates/wgpu-html-parser/src/`

| Item | Status |
|---|---|
| Parse `<link rel="stylesheet" href="...">` elements | ❌ |
| Fetch CSS files (HTTP, file, data URI) in tree-building phase | ❌ |
| Register fetched CSS as linked stylesheets via `Tree::linked_stylesheets` | ❌ |

**Why:** `linked_stylesheets` field exists on `Tree` and is consumed by the cascade, but there's no fetch to populate it. External CSS is the primary way stylesheets are loaded.

**Depends on:** HTTP client or file I/O.

---

## P6 — Interactivity Gaps

### P6-1: Cursor styling
Lines changed: ~30 in `crates/wgpu-html-winit/src/`

| Item | Status |
|---|---|
| Read `cursor` from hovered element's `LayoutBox` | ❌ |
| Set OS cursor via winit `Window::set_cursor_icon()` | ❌ |
| Custom cursor images (`cursor: url(...)`) | ❌ |

**Why:** `cursor` property is parsed and stored in `LayoutBox` but never applied to OS cursor.

**Depends on:** none — winit glue change.

---

### P6-2: `preventDefault` / `stopPropagation`
Lines changed: ~100 in `crates/wgpu-html-tree/src/dispatch.rs`

| Item | Status |
|---|---|
| Add `default_prevented` flag to `HtmlEvent` | ❌ |
| Check flag in harness before executing default actions | ❌ |
| Add `stop_propagation` flag, stop bubble on set | ❌ |

**Why:** Event hooks can't prevent default browser-like behavior (e.g., scrolling, keyboard navigation).

**Depends on:** none.

---

### P6-3: Double-click / triple-click / context-menu synthesis
Lines changed: ~50 in `crates/wgpu-html-winit/src/`

| Item | Status |
|---|---|
| Detect double-click from time+position proximity | ❌ |
| Synthesize `dblclick` event | ❌ |
| Synthesize `contextmenu` event on right-click | ❌ |

**Why:** Common desktop interaction patterns.

**Depends on:** none.

---

## P7 — Infrastructure

### P7-1: `em`/`rem` against actual inherited font-size
Lines changed: ~30 in `crates/wgpu-html-layout/src/length.rs`

| Item | Status |
|---|---|
| Pass computed `font-size` through layout context for `em`/`rem` resolution | ❌ |
| Currently uses hard-coded 16px fallback when no font-size is inherited | ❌ |

**Why:** `em` values on elements that set their own `font-size` are wrong.

**Depends on:** propagating font-size through layout context (minor refactor).

---

### P7-2: Flex baseline alignment
Lines changed: ~100 in `crates/wgpu-html-layout/src/flex.rs`

| Item | Status |
|---|---|
| Track text baseline (ascent) through block layout | ❌ |
| Align flex items by their first text baseline | ❌ |

**Why:** `align-items: baseline` falls back to `flex-start` currently.

**Depends on:** propagating ascent from inline text leaves to block layout (moderate refactor).

---

## Summary (by effort)

| Priority | Feature | Est. lines | Dependencies |
|---|---|---|---|
| **P0** | Per-glyph clipping | 15 | none |
| **P0** | `z-index` / stacking contexts | 200 | none |
| **P0** | `border-color: currentColor` | 10 | none |
| **P1** | Float layout | 500 | none |
| **P1** | Table layout | 800 | none |
| **P1** | Sticky positioning | 50 | scroll tracking |
| **P2** | Selector combinators + pseudo-classes in cascade | 200 | query engine (exists) |
| **P2** | `:focus-within`, `:disabled` in cascade | 50 | P2-1 |
| **P2** | `@media` at-rule | ✅ Done | — |
| **P3** | Checkbox/radio toggle | 40 | none |
| **P3** | `<select>` dropdown | 500 | P0-2 |
| **P3** | Form submission | 80 | none |
| **P4** | Gradients | 400 | none |
| **P4** | `box-shadow` | 200 | none |
| **P4** | `transform` (2D) | 200 | none |
| **P4** | `filter` | 150 | none |
| **P5** | Unknown tag preservation | 20 | none |
| **P5** | Whitespace text preservation | 20 | none |
| **P5** | `<link>` stylesheet loading | 150 | HTTP/file I/O |
| **P6** | Cursor styling | 30 | none |
| **P6** | `preventDefault`/`stopPropagation` | 100 | none |
| **P6** | Double-click synthesis | 50 | none |
| **P7** | `em`/`rem` font-size resolution | 30 | none |
| **P7** | Flex baseline alignment | 100 | ascent tracking |
| **Total** | | **~3,700** | |
