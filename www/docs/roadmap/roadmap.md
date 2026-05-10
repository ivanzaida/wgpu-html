---
sidebar_position: 1
---

# Roadmap

## Current Status

The core pipeline is complete and functional: HTML parsing, CSS cascade, Flexbox + Grid layout, text shaping, GPU rendering, mouse+keyboard interactivity, form controls, text editing, and scrolling.

## Completed Milestones

| Milestone | Status | Description |
|---|---|---|
| M1 — wgpu skeleton | ✅ | Device/surface/pipelines, winit event loop, frame outcome |
| M2 — solid quad pipeline | ✅ | Instanced rectangles, WGSL shader, alpha blending |
| M3 — paint a tree | ✅ | Parse HTML, resolve styles, absolute positioning |
| M4 — block layout | ✅ | Vertical stacking, margin/padding, width/height resolution |
| M4½ — CSS cascade | ✅ | Selector matching, specificity, inheritance, `!important` |
| M5 — text rendering | ✅ | cosmic-text shaping, glyph atlas, GPU text pipeline |
| M6 — inline layout | ✅ | IFC, line boxes, word wrap, `text-align` |
| M7 — backgrounds & borders | ✅ | SDF rounded quads, dashed/dotted borders, `background-clip` |
| M8 — images | ✅ | URL/data-URI loading, animated GIF/WebP, caching |
| M9 — flexbox | ✅ | Complete CSS Flexbox Level 1 |
| M10 — grid | ✅ | CSS Grid with `fr`, `repeat()`, placement, alignment |
| M11 — clipping & overflow | ✅ | Scissor + SDF rounded clipping, scroll containers |
| M12 — interactivity | ✅ | Hover/click/focus chain, form controls, text selection, scroll |

## In Progress / Planned

| Feature | Priority | Notes |
|---|---|---|
| **Table layout** | High | Implemented: `display: table` variants work with colspan/rowspan, column distribution. Refining edge cases. |
| **z-index stacking contexts** | High | Sibling sort done; cross-branch stacking contexts needed |
| **Floats** | Medium | `float: left/right` not yet parsed |
| **Baseline alignment** | Medium | Needed for flex/grid baseline alignment |
| **`@font-face`** | Medium | Generic family fallback works; web font loading not yet |
| **`em`/`rem` font-size** | Medium | Currently hard-coded 16px without inherited font-size |
| **Transforms** | Medium | Parsed but stored raw; need layout + hit-test impact |
| **`position: sticky`** | Medium | Currently degrades to `relative` |
| **`box-shadow`** | Medium | Parsed as raw string; needs paint implementation |
| **`text-overflow`** | Low | Ellipsis not rendered |
| **`<select>` dropdown** | Low | Parser + styling exist; popup interaction missing |
| **`:focus-visible`** | Low | Keyboard vs pointer focus not tracked |
| **Smooth scrolling** | Low | `scroll-behavior` not implemented |
| **CSS transitions/animations** | Deferred | Parsed but stored raw; no runtime |
| **Multi-column layout** | Deferred | Shorthands recognized; layout not implemented |
| **Filter effects** | Deferred | Not parsed |

## Explicitly Out of Scope

- **JavaScript** — permanently excluded. No `<script>` execution, no JS engine, no `eval`.
- **Web platform APIs** — no `document`, `window`, `history`, `localStorage`, `fetch`, etc.
- **Accessibility tree** — no ARIA processing, no screen reader integration
- **Print layout** — no paged media, no `@page`
- **Full SVG rendering** — basic rasterized `<svg>` with `<path>` only; no inline SVG element tree

## Versioning

Pre-1.0:
- `wgpu` and `winit` versions pinned at workspace root
- `models` crate may have breaking changes as CSS coverage expands
- Driver trait API is evolving
