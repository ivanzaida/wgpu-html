---
title: Scrolling
---

# Scrolling

Scrolling is implemented in `lui::scroll` (413 lines) with support for viewport-level and per-element scrolling.

## Scroll State

Scroll offsets are stored per-element:

```rust
pub scroll_offsets_y: BTreeMap<Vec<usize>, f32>,
```

The key is the element's path (a `Vec<usize>` matching the DOM and layout tree structure). The value is the vertical pixel offset. The viewport's scroll is keyed by an empty path (`vec![]`).

## Viewport Scroll

Mouse wheel events scroll the viewport by default:

```rust
tree.dispatch_scroll(&layout, (x, y), delta, &mut tree.interaction.scroll_offsets_y);
```

The dispatch layer:
1. Hit-tests the mouse position against the layout.
2. Finds the nearest scrollable ancestor (viewport or `overflow: scroll`/`auto` container).
3. Applies the delta to that element's scroll offset.
4. Clamps the offset to `[0, max_scroll_y]`.

## Per-Element Scroll

When an element has `overflow: scroll` or `overflow: auto`, its content can be scrolled independently:

```css
.scrollable-panel {
    overflow-y: auto;
    max-height: 400px;
}
```

The mouse wheel scrolls the deepest scrollable container under the pointer. If the container is already at its scroll limit, the event bubbles to the parent scrollable.

## Scrollbar Geometry

```rust
pub struct ScrollbarGeometry {
    pub track: Rect,       // Full scrollbar track
    pub thumb: Rect,       // Draggable thumb
    pub max_scroll: f32,   // Maximum scroll_y value
    pub travel: f32,       // Pixel range the thumb can move
}

pub fn scrollbar_geometry(
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
    scroll_y: f32,
) -> Option<ScrollbarGeometry>
```

The scrollbar is a 10px-wide track on the right edge, with a thumb sized proportionally to `viewport_h / document_h`. The minimum thumb height is 24px.

## Thumb Drag

```rust
pub fn scroll_y_from_thumb_top(
    thumb_top: f32,
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
) -> f32
```

When dragging the thumb, the new `scroll_y` is computed from the thumb's vertical position, clamped to the valid range.

## Scroll Clamping

```rust
pub fn clamp_scroll_y(scroll_y: f32, layout: &LayoutBox, viewport_h: f32) -> f32
```

Ensures `scroll_y` stays in `[0, max_scroll_y]`. `max_scroll_y` is the difference between the document's bottom edge and the viewport height.

## Scrollbar Painting

```rust
pub fn paint_viewport_scrollbar(
    list: &mut DisplayList,
    layout: &LayoutBox,
    viewport_w: f32,
    viewport_h: f32,
    scroll_y: f32,
)
```

Emits two filled quads: the track (semi-transparent dark) and the thumb (more opaque). The scrollbar is painted after the document content, so it appears on top.

## Display List Translation

```rust
pub fn translate_display_list_y(list: &DisplayList, dy: f32) -> DisplayList
```

Creates a copy of the display list offset vertically by `dy` pixels. Used to apply the viewport scroll offset to the rendered output. Glyph positions are translated in-place.

## Scroll Utilities

```rust
pub fn viewport_to_document(pos: (f32, f32), scroll_y: f32) -> (f32, f32);
pub fn rect_contains(rect: Rect, pos: (f32, f32)) -> bool;
pub fn max_scroll_y(layout: &LayoutBox, viewport_h: f32) -> f32;
```

## Complete Example

```rust
use lui::scroll;

// Handle wheel scroll
let delta_y = scroll_delta as f32;
scroll::scroll_element_at(
    &layout, pos, delta_y, viewport_h,
    &mut tree.interaction.scroll_offsets_y,
);

// Paint scrollbar
let scroll_y = tree.interaction.scroll_offsets_y
    .get(&vec![]).copied().unwrap_or(0.0);
scroll::paint_viewport_scrollbar(&mut list, &layout, viewport_w, viewport_h, scroll_y);

// Translate display list by scroll offset
let scrolled = scroll::translate_display_list_y(&list, -scroll_y);
```
