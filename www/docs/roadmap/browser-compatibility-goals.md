---
sidebar_position: 3
---

# Browser Compatibility Goals

## Philosophy

wgpu-html aims for **CSS behavior parity** with web browsers, not pixel-perfect rendering. The goal is for CSS authored for browsers to work correctly in wgpu-html. Differences in anti-aliasing, subpixel positioning, and font metrics are expected.

## CSS Standards Coverage

### Box Model
- **Target:** CSS3 Box Model (Content → Padding → Border → Margin)
- **Status:** Complete for physical (top/right/bottom/left) properties
- **Gaps:** No logical properties (block/inline-start/end), no `margin-trim`

### Flexbox
- **Target:** CSS Flexible Box Layout Level 1
- **Status:** Complete
- **Gaps:** Baseline alignment of non-text items

### Grid
- **Target:** CSS Grid Layout Level 1
- **Status:** Core features complete (track sizing, placement, alignment, gaps)
- **Gaps:** `grid-template-areas`, `repeat(auto-fill/auto-fit)`, `dense` packing, named lines, subgrid

### Positioning
- **Target:** CSS Positioned Layout Level 3
- **Status:** `static`, `relative`, `absolute`, `fixed` complete; `sticky` degraded
- **Gaps:** Full sticky behavior, anchor positioning

### Selectors
- **Target:** CSS Selectors Level 4
- **Status:** Complete — all combinators, pseudo-classes, attribute selectors, logical pseudo-classes (`:is()`, `:where()`, `:not()`, `:has()`)
- **Gaps:** None significant

### Cascade
- **Target:** CSS Cascade Level 3
- **Status:** Complete — specificity, `!important`, CSS-wide keywords, `@media`
- **Gaps:** `@supports`, `@import`, `@layer`

### Values & Units
- **Target:** CSS Values Level 3/4
- **Status:** `px`, `%`, `em`, `rem`, `vw`, `vh`, `vmin`, `vmax`, `calc()`, `min()`, `max()`, `clamp()`, `var()`
- **Gaps:** No `ex`, `ch`, `ic`, `lh` units; no `env()`; trigonometric functions parsed but not consumed

### Colors
- **Target:** CSS Color Level 4
- **Status:** sRGB hex, rgb/rgba, hsl/hsla, named colors, `transparent`
- **Gaps:** No `lab()`, `lch()`, `oklab()`, `oklch()`, `color()`; `currentColor` resolution is partial

### Typography
- **Target:** CSS Text Level 3 + CSS Fonts Level 3
- **Status:** `font-family/size/weight/style`, `text-align/transform/decoration`, `letter-spacing`, `line-height`, `white-space`
- **Gaps:** No `@font-face`, `word-break`, `hyphens`, `text-indent`, `text-overflow`, `font-variant`

### Overflow & Scrolling
- **Target:** CSS Overflow Level 3
- **Status:** `visible`, `hidden`, `scroll`, `auto` with scissor + SDF clipping
- **Gaps:** No `scroll-behavior`, `overscroll-behavior`, `scroll-snap`

## Behavior Differences from Browsers

These are intentional or accepted differences, not bugs:

| Area | Difference |
|---|---|
| Font rasterization | CPU via cosmic-text, not GPU via DirectWrite/CoreText |
| Subpixel positioning | Single-precision `f32`, not subpixel |
| Anti-aliasing | SDF-based for quads/borders, atlas-based for glyphs |
| Margin collapse | Implements margin collapsing for block siblings but not fully |
| Line height | Computed values approximate; may differ from browser at fractional sizes |
| UA stylesheet | Custom UA stylesheet, not the full HTML5 UA stylesheet |
| Rendering pipeline | Single-pass GPU rendering, not multi-pass compositing |

## What Will Never Be Supported

- JavaScript execution and DOM scripting APIs
- Web platform APIs (`fetch`, `localStorage`, `WebSocket`, etc.)
- The `document` / `window` global objects and their APIs
- Service workers and web app manifests
- Browser extensions
- SVG animation and interactive SVG
- CSS Regions, Exclusions, and Shapes
- CSS Custom Highlight API
- Web Components Shadow DOM (custom elements with factory functions are supported)
