---
id: positioned
title: Positioned Layout
---

# Positioned Layout

Out-of-flow positioning handles `position: absolute`, `fixed`, and `relative`. The engine implements the CSS Positioned Layout Module Level 3.

## Position Values

| Value | Behavior |
|---|---|
| `static` | Normal flow (default) |
| `relative` | Normal flow position, then offset by `top`/`left`/`right`/`bottom` |
| `absolute` | Removed from flow; positioned relative to nearest positioned ancestor |
| `fixed` | Removed from flow; positioned relative to the viewport |
| `sticky` | Degrades to `relative` (known limitation) |

## layout_out_of_flow_block()

```rust
fn layout_out_of_flow_block(
    node: &CascadedNode,
    static_x: f32,
    static_y: f32,
    _container_w: f32,
    _container_h: f32,
    containing_block: Rect,
    ctx: &mut Ctx,
) -> LayoutBox
```

### Containing Block

- **absolute**: The padding box of the nearest ancestor with `position != static`. Falls back to the initial containing block.
- **fixed**: The viewport (`Rect::new(0, 0, viewport_w, viewport_h)`).

### Insets

CSS inset properties (`top`, `right`, `bottom`, `left`) resolve against the containing block dimensions:

```css
.overlay {
    position: absolute;
    top: 20px;
    right: 0;
    bottom: 0;
    left: 0;
}
```

Percentage values resolve against the containing block's *width* for left/right and *height* for top/bottom.

### Shrink-to-Fit

When an absolutely-positioned element has no explicit `width` but left + right are both set, the width is computed as:

```
available_w = cb_w - left - right - margin_horizontal - border_horizontal - padding_horizontal
```

When neither pair is set (only one inset, or none), the width shrink-wraps to content:

```rust
fn shrink_to_fit_content_width(node: &CascadedNode, available_w: f32, ctx: &mut Ctx) -> f32
```

### Right/Bottom Anchoring

When `left` is not set but `right` is set, the box anchors to the right edge:

```rust
if left.is_none() && let Some(right) = right {
    let target_x = cb.x + cb.w - right - box_.margin_rect.w;
    translate_box_x_in_place(&mut box_, target_x - box_.margin_rect.x);
}
```

The same logic applies symmetrically for `bottom`.

## apply_relative_position()

```rust
fn apply_relative_position(box_: &mut LayoutBox, style: &Style, container_w: f32, container_h: f32, ctx: &mut Ctx)
```

For `relative` elements, the box is first laid out in normal flow, then translated by the resolved inset values. When both `left` and `right` are set, `left` wins. When both `top` and `bottom` are set, `top` wins.

## Sticky

`position: sticky` currently degrades to `relative`. The scroll-offset-based threshold logic is not yet implemented. Elements with `sticky` render at their static position.

## Example

```css
.tooltip-container {
    position: relative;
}

.tooltip {
    position: absolute;
    bottom: 100%;
    left: 50%;
    transform: translateX(-50%);  /* transforms not supported; use calc */
    padding: 8px 12px;
    background: #333;
    color: white;
    border-radius: 6px;
    white-space: nowrap;
}
```

The tooltip is removed from flow, positioned above its `relative` parent, and sized to its content.
