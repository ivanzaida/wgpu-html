---
sidebar_position: 9
---

# Clipping and Overflow

## Overflow Values

| Value | Behavior |
|---|---|
| `visible` | No clipping. Content extends beyond padding box (default). |
| `hidden` | Content clipped to padding box. |
| `scroll` | Content clipped; scrollbars always visible. |
| `auto` | Content clipped; scrollbars only when needed. |

Per-axis control via `overflow-x` and `overflow-y`:

```css
.box {
    overflow-x: hidden;
    overflow-y: auto;
}
```

When only one axis is clipped, the `visible` axis extends its clip rect to effectively infinite range (±1,000,000 px).

## Clipping Mechanics

Overflow clipping uses a two-level approach:

1. **Rectangular scissor** (`set_scissor_rect`) — clips to the padding-box rectangle on the GPU
2. **Rounded SDF discard** — fragment shader discards pixels outside the rounded inner-padding edge when `border-radius` is present on a clipping container

The inner-padding-edge radii are computed by shrinking the outer `border-radius` by the border thickness on each side, matching browser behavior.

## Nested Clipping

When multiple `overflow ≠ visible` elements are nested, their clip rects are intersected:

- Scissor rect: intersection of all ancestor clip rects
- Rounded SDF: only the innermost clip's radii (composing multiple rounded clips is not yet supported)

## Clip Ranges in DisplayList

Paint operations are partitioned into clip ranges:

```rust
struct ClipRange {
    rect: Rect,              // scissor rectangle
    border_radius: [f32; 4],  // rounded corner radii (for SDF discard)
    glyph_range: Range<usize>,
    quad_range: Range<usize>,
    image_range: Range<usize>,
}
```

## Border-Radius Clipping

When a container has both `overflow: hidden` and `border-radius`, clipping follows the rounded inner-padding edge:

```css
.rounded-clip {
    overflow: hidden;
    border-radius: 12px;
    /* Content clips to the inner rounded rect (12px - border width) */
}
```

This works for both circular and elliptical radii (the `/` syntax in `border-radius`).

## Known Limitations

- Only one nested rounded clip is composed (innermost wins)
- `overflow: clip` semantics not distinct from `hidden`
- No `clip-path` support
- Scroll offsets inside scroll containers don't yet adjust hit-test coordinates
