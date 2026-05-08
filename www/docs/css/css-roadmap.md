---
title: CSS Roadmap
---

# CSS Roadmap — What's Missing

> **Date:** 2026-05-07
> Companion to the [Implementation Status](/docs/status) page. This doc lists every CSS feature gap, organized by implementation priority.

---

## 🔴 Blockers — no support at all

These must be built from scratch to claim CSS feature parity.

### ~~Selectors in cascade~~ ✅ DONE

The cascade delegates to `query.rs`'s full `ComplexSelector::matches_in_tree()` at `lib.rs:1300`. All CSS Level 4 selectors work:

| Feature | Parser | Query engine | Cascade |
|---------|--------|-------------|---------|
| `>`, `+`, `~` combinators | ✅ parsed | ✅ full | ✅ **works** |
| `:is()`, `:where()`, `:not()` | ✅ parsed | ✅ full | ✅ **works** |
| `:has()` | ✅ parsed | ✅ full (relative selectors) | ✅ **works** |
| `:nth-child()`, `:nth-of-type()` | ✅ parsed | ✅ full (+ `of S` syntax) | ✅ **works** |
| `:disabled`, `:enabled`, `:checked` | ✅ parsed | ✅ full | ✅ **works** |
| `:required`, `:optional`, `:read-only`, `:read-write` | ✅ parsed | ✅ full | ✅ **works** |
| `:placeholder-shown` | ✅ parsed | ✅ full | ✅ **works** |
| `:focus-within` | ✅ parsed | ✅ full | ✅ **works** |
| `:lang()`, `:dir()` | ✅ parsed | ✅ full | ✅ **works** |
| `:first-of-type`, `:last-of-type`, `:only-child`, `:empty` | ✅ parsed | ✅ full | ✅ **works** |
| Multi-value attrs (`~=`, `\|=`, `^=`, `$=`, `*=`) | ✅ parsed | ✅ full | ✅ **works** |
| Case-insensitive attribute flag (`i`) | ✅ parsed | ✅ full | ✅ **works** |

> **Note:** `:focus-visible` requires keyboard-vs-pointer focus origin tracking which is not yet implemented.

### At-rules

The stylesheet parser skips every `@`-prefixed block except `@media`, `@charset`, and `@import`.

| At-rule | Status |
|---------|--------|
| `@charset` | ✅ parsed and skipped (Rust strings are always UTF-8) |
| `@media` | ✅ fully implemented (width/height/orientation, min/max, `not`) |
| `@import` | ✅ fully supported — inlines CSS from `linked_stylesheets`, media queries, cycle detection, auto-load via `resolve_css_imports()` |
| `@font-face` | ❌ not parsed — no font-face loading pipeline |
| `@keyframes` | ❌ not parsed — no animation engine |
| `@supports` | ❌ not parsed — no feature-query evaluation |
| `@layer` | ❌ not parsed — no cascade-layer awareness |
| `@scope` | ❌ not parsed — no scoped styling |
| `@container` | ❌ not parsed — no container-query evaluation |
| `@page` | ❌ not parsed — no paged media support |
| `@font-feature-values` | ❌ not parsed — no OpenType feature value aliasing |
| `@counter-style` | ❌ not parsed — no custom counter styles |
| `@property` | ❌ not parsed — no registered custom property types |
| `@namespace` | ❌ not parsed — no XML namespace support |
| `@color-profile` | ❌ not parsed — no ICC color profile support |
| `@font-palette-values` | ❌ not parsed — no font palette customization |
| `@starting-style` | ❌ not parsed — no entry animations |
| `@position-try` | ❌ not parsed — no anchor positioning fallbacks |
| `@view-transition` | ❌ not parsed — no view transition API |

### Pseudo-elements

`::before` and `::after` are fully supported with `content` string values. The cascade computes pseudo-element styles on `CascadedNode.before`/`.after`, and layout injects synthetic children into both block and inline formatting contexts.

| Pseudo-element | Parser | Cascade | Renderer | Notes |
|---------------|--------|---------|----------|-------|
| `::before` | ✅ | ✅ matched + styled | ✅ rendered (block + inline) | |
| `::after` | ✅ | ✅ matched + styled | ✅ rendered (block + inline) | |
| `::first-line` | ✅ | ✅ matched + styled | ✅ color override on first-line glyphs | |
| `::first-letter` | ✅ | ✅ matched + styled | ✅ color override on first glyph | |
| `::marker` | ✅ | ✅ auto-generated | ✅ bullets + numbers (disc/circle/square, decimal, alpha, roman) | |
| `::placeholder` | ✅ | ✅ matched + styled | ✅ color override on placeholder text | |
| `::selection` | ✅ | ✅ matched + styled | ✅ color + background override on selected text | |
| `::backdrop` | ❌ | ❌ | ❌ | Requires top-layer / `<dialog>` fullscreen rendering |
| `::cue` | ❌ | ❌ | ❌ | WebVTT subtitle styling; requires `<video>` subtitle support |
| `::details-content` | ❌ | ❌ | ❌ | Requires `<details>`/`<summary>` open/close toggle |
| `::file-selector-button` | ❌ | ❌ | ❌ | Requires `<input type="file">` native file-picker UI |

### Layout gaps

| Feature | Status |
|---------|--------|
| `float: left/right` | ❌ not even parsed — no property entry exists |
| `display: table` and friends | ⚠️ all 10 table `Display` variants parsed but fall through to block layout |
| Multi-column (`column-count`, `column-width`) | ⚠️ parsed as deferred longhands, never consumed by layout |
| `flex-basis: content` | ❌ not implemented |
| Baseline alignment in flex (`align-items: baseline`) | ⚠️ parsed, falls back to `flex-start` (`flex.rs:1236`) |
| Sticky positioning | ⚠️ parsed, degraded to `relative` |

### Animation & transitions

| Feature | Status |
|---------|--------|
| `@keyframes` | ❌ not parsed |
| Animation engine | ❌ none — `animation-*` properties parsed into 13 deferred longhands, never consumed |
| Transition engine | ❌ none — `transition-*` parsed into 5 deferred longhands, never consumed |
| `animation-timeline` (scroll-driven) | ⚠️ parsed as deferred longhand, no runtime |
| `scroll-timeline` | ⚠️ parsed as shorthand → deferred longhands |
| `prefers-reduced-motion` media query | ❌ not implemented |

### Media & container queries

| Feature | Status |
|---------|--------|
| `prefers-color-scheme` | ❌ not implemented (system colors always light-mode) |
| `prefers-contrast` | ❌ not implemented |
| `prefers-reduced-motion` | ❌ not implemented |
| `@container` / `container-type` / `container-name` | ⚠️ parsed as deferred longhands, no runtime |

---

## 🟡 Partial — parsed but not consumed

These properties are parsed into the `Style` struct or stored as raw strings, but have no effect on layout or rendering.

### Parsed as raw `Option<String>`

| Property | Where stored | Gap |
|----------|-------------|-----|
| `box-shadow` | `Style.box_shadow` (`style.rs:211`) | No shadow rendering pipeline |
| `transform` | `Style.transform` (`style.rs:200`) | No transform application in layout or GPU |
| `transform-origin` | `Style.transform_origin` (`style.rs:202`) | No transform pipeline |
| `transition` | Deferred longhands (5 fields) | No transition engine |
| `animation` | Deferred longhands (13 fields) | No animation engine |
| `filter` | **Not even parsed** — silently dropped | No filter pipeline |

### Parsed but rendering pipeline missing

| Property | Parser status | Rendering status |
|----------|-------------|-----------------|
| ~~`linear-gradient()`~~ | ~~`CssImage::Function(String)`~~ | ✅ **Done** — CPU rasterization into image pipeline |
| ~~`radial-gradient()`~~ | ~~`CssImage::Function(String)`~~ | ✅ **Done** — CPU rasterization into image pipeline |
| ~~`conic-gradient()`~~ | ~~`CssImage::Function(String)`~~ | ✅ **Done** — CPU rasterization into image pipeline |
| `outline` | Shorthand → 3 deferred longhands | Never rendered |
| `text-shadow` | Deferred longhand | Never consumed by paint |
| `border-image` | Shorthand → 5 deferred longhands | No border-image rendering path |
| `background-origin` | Deferred longhand | No effect — always padding-box origin |
| `background-attachment` | Deferred longhand | `fixed` would require viewport-relative positioning |

### Color value gaps

| Value | Parser | Resolution |
|-------|--------|-----------|
| ~~`currentColor`~~ | ~~`CssColor::CurrentColor`~~ | ✅ **Done** — resolves to inherited `color`; borders without explicit color use foreground |
| `color-mix()` | `CssColor::Function(String)` | Returns `None` |
| `lab()` / `lch()` / `oklab()` / `oklch()` | `CssColor::Function(String)` | Returns `None` |
| `color()` function | `CssColor::Function(String)` | Returns `None` |
| `hwb()` / `light-dark()` | `CssColor::Function(String)` | Returns `None` |
| System colors | Resolved to fixed light-mode sRGB | No dark-mode awareness |

### Cascade gaps

| Feature | Status |
|---------|--------|
| `revert` keyword | Parsed as token but never applied — only used to skip animation-name parsing |
| `revert-layer` keyword | Same as `revert` |
| Multiple backgrounds | Not supported — `background-image` is single `Option<CssImage>`, not `Vec` |
| `z-index` stacking contexts | Sorts siblings by value (3 tests pass), but no independent cross-branch stacking |

### Other partial features

| Feature | Status |
|---------|--------|
| `gap` in non-flex/grid contexts | Parsed but not consumed by block layout |
| `display: inline / inline-block` (author-set) | IFC auto-detected from content, not driven by `display` value |
| Dashed/dotted on non-uniform rounded corners | Corners stay bare — only uniform-circular radii follow the curve |
| Border `double`/`groove`/`ridge`/`inset`/`outset` styles | Render as plain `solid` |
| `<link rel="stylesheet">` | Field exists on `Tree`, parsed from HTML, but no HTTP fetch |

---

## 🟠 Missing properties — not parsed at all

These CSS properties have no entry in the parser (`apply_css_property`), `Style` struct, or deferred longhands table.

| Property | Notes |
|----------|-------|
| `text-overflow` | No ellipsis/fade rendering |
| `word-break` | No break-all/keep-all logic |
| `vertical-align` | UA sets it on inline elements but parser ignores it |
| `object-fit` | Images use hardcoded contain/cover logic |
| `object-position` | No positioning of replaced content |
| `mix-blend-mode` | No compositing support |
| `isolation` | No stacking context support |
| `will-change` | No GPU hint system |
| `contain` | No containment support |
| `aspect-ratio` | No intrinsic sizing ratio |
| `scroll-behavior` | No `smooth` scrolling |
| `scroll-snap-type` / `scroll-snap-align` | No snap-point system |
| `scroll-margin-*` / `scroll-padding-*` | Parsed as deferred, never consumed |
| `cursor` (OS cursor shape) | Parsed into `Style.cursor`, not applied to OS cursor |
| `accent-color` | Not parsed |
| `caret-color` | Not parsed |
| `print-color-adjust` | Not parsed |
| `hyphens` | Not parsed |
| `tab-size` | Not parsed |

---

## 🟢 What works

CSS-Cascade-3 ordering with specificity, `!important` band separation, `inherit`/`initial`/`unset` keywords, `@media` queries (width/height/orientation), full CSS Level 4 selectors in cascade (all combinators, 30+ pseudo-classes, attribute operators, `:is`/`:where`/`:not`/`:has`), `calc()`/`min()`/`max()`/`clamp()` with full AST, `var()`/custom properties with inheritance and cycle detection, complete Flexbox Level 1, complete CSS Grid, positioned layout (absolute/relative/fixed), 80+ parsed CSS properties with typed resolution, CSS gradients (`linear-gradient`, `radial-gradient`, `conic-gradient` + repeating variants).

---

## Suggested implementation order

The shortest path to "full CSS producer-grade engine":

| Priority | Task | Impact |
|----------|------|--------|
| ~~1~~ | ~~Port query engine selectors into cascade~~ | ✅ **Done** — full CSS4 selectors in cascade |
| ~~2~~ | ~~Implement `currentColor` resolution~~ | ✅ **Done** — borders, backgrounds, pseudo-elements |
| ~~3~~ | ~~Build gradient rasterizer~~ | ✅ **Done** — linear/radial/conic + repeating |
| ~~7~~ | ~~Pseudo-element rendering~~ | ✅ **Done** — `::before`/`::after`/`::marker`/`::first-line`/`::first-letter`/`::placeholder`/`::selection` |
| 4 | Implement `@keyframes` + animation engine | Motion design becomes possible |
| 5 | Float layout | Required for text-wrap-around-images layouts |
| 6 | Table layout algorithm | Required for data tables |
| 8 | `box-shadow` / `text-shadow` rendering | Visual depth |
| 9 | Multi-column layout | Text-heavy page layouts |
| 10 | Transforms (`transform`, `transform-origin`) | Animations, layout adjustments |
| 11++ | Remaining property gaps, `@layer`, `@supports`, `@container`, media queries, modern colors | Edge cases and modern CSS features |
