---
sidebar_position: 6
---

# Painting

The paint stage converts a positioned `LayoutBox` tree into a backend-agnostic `DisplayList`. It lives in `crates/wgpu-html/src/paint.rs`.

## Entry Point

```rust
pub fn paint_tree_returning_layout(
    layout: &LayoutBox,
    cascaded: &CascadedTree,
    viewport_width: f32,
    viewport_height: f32,
) -> DisplayList
```

## DisplayList

The display list is a flat collection of draw commands plus clip range metadata:

```rust
pub struct DisplayList {
    pub quads: Vec<DisplayCommand>,
    pub images: Vec<DisplayCommand>,
    pub glyphs: Vec<DisplayCommand>,
    pub clips: Vec<ClipRange>,
    // ...
}
```

Each `DisplayCommand` carries:
- `rect` — position and size in absolute pixels
- `color` — RGBA (linear for GPU)
- `border_radius` — corner radii for SDF shading
- `clip_index` — which clip range this command belongs to
- `texture_id` / `uv_rect` — for image commands

## Paint Walk

The paint traversal walks the LayoutBox tree depth-first:

1. **Background** — emit quad for `background-color` (respects `background-clip`)
2. **Background image** — emit image tiles from `background_image`
3. **Borders** — emit border quads per side
4. **Text** — emit glyph commands from `text_run`
5. **Overflow clip** — push clip range if `overflow != visible`
6. **Children** — recurse into children
7. **Pop clip** — close the clip range

## Clip Ranges

`overflow: hidden` (and scroll/auto) produce clip ranges that partition draw commands:

```rust
pub struct ClipRange {
    pub rect: Rect,            // scissor rectangle
    pub border_radius: [f32; 4], // rounded corner radii
    pub glyph_range: Range<usize>,
    pub quad_range: Range<usize>,
    pub image_range: Range<usize>,
}
```

Clip ranges are stacked: nested `overflow: hidden` elements create nested ranges with intersected rectangles and inner-edge radii.

## Clipping Detail

- **Rounded clipping** — when a container has `border-radius` and `overflow: hidden`, the clip uses the inner padding-edge radii (outer radius minus border width), matching browser behavior.
- **Axis independence** — `overflow-x` and `overflow-y` can clip independently or collapse via `effective_overflow()`.
- **Null ranges** — empty clip ranges (no drawn children) are retained with remapped `clip_index` values to prevent index shifting.

## Background Images

Pre-computed during layout into `BackgroundImagePaint` with pre-positioned tile rects. Paint iterates tiles and emits textured quads, considering `background-size`, `background-position`, and `background-repeat`.

## Paint-Only Optimization

When the `PipelineCache` detects that only paint-affecting properties changed (color, opacity, background-color), layout is skipped and the existing LayoutBox tree is re-painted. The `patch_color_recurse()` function updates colors in-place on the LayoutBox tree before painting, avoiding full re-layout.
