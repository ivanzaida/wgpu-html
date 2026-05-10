---
sidebar_position: 8
---

# Scrolling

## Viewport Scrolling

The entire document can be scrolled vertically and horizontally via the viewport scroll offset. The scroll offset is threaded through layout and paint, translating all content.

Scrollbar geometry is computed from the document's total height vs viewport height. A scrollbar track and thumb are painted as quad primitives.

## Per-Element Scrolling

Elements with `overflow: scroll` or `overflow: auto` become scroll containers:

```css
.scroll-box {
    overflow: auto;
    height: 200px;
    scrollbar-color: #555 #222;
    scrollbar-width: thin;
}
```

Scroll offsets are tracked per layout path in `BTreeMap<Vec<usize>, ScrollOffset>`.

## Scrollbar Styling

| Property | Values | Notes |
|---|---|---|
| `scrollbar-color` | `auto`, `<thumb-color> <track-color>` | Custom track/thumb colors |
| `scrollbar-width` | `auto`, `thin`, `none` | Width override |

## Scroll Interaction

- **Mouse wheel** — scrolls viewport or deepest scrollable element under cursor
- **Scrollbar drag** — drag the thumb to scroll proportionally
- **Touch pad** — two-finger scrolling supported
- **Overflow: scroll** — always shows scrollbar
- **Overflow: auto** — shows scrollbar only when content exceeds container

## Scroll Utilities

Public API in `wgpu-html/src/scroll.rs`:

```rust
// Viewport scroll
pub fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32;
pub fn clamp_scroll_x(scroll_x: f32, layout: &LayoutBox, viewport_w: f32) -> f32;
pub fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32;

// Element scroll
pub fn scroll_element_at(tree, layout, pos, delta_x, delta_y) -> bool;
pub fn deepest_scrollable_path_at(layout, pos) -> Option<Vec<usize>>;
pub fn scroll_element_thumb_to_y(tree, layout, path, thumb_top);

// Scrollbar geometry
pub fn scrollbar_geometry(layout, vw, vh, scroll_y) -> Option<ScrollbarGeometry>;
pub fn element_scrollbar_geometry(b: &LayoutBox, scroll_y: f32) -> Option<ScrollbarGeometry>;
```

## Current Limitations

- No `scroll-behavior: smooth`
- No `overscroll-behavior`
- No `scroll-snap-*` properties
- Wheel events not forwarded to element `on_event` callbacks
- No scroll anchoring
