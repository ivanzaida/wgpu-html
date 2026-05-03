---
title: Paint Translation
---

# Paint Translation (LayoutBox → DisplayList)

The paint stage (`wgpu-html::paint`) walks the `LayoutBox` tree depth-first and emits a backend-agnostic `DisplayList`. The renderer consumes the display list without any knowledge of CSS or layout.

## Entry Points

```rust
// Cascade + layout + paint in one call
pub fn paint_tree_with_text(
    tree: &Tree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
) -> DisplayList;

// Paint a pre-computed layout
pub fn paint_layout(root: &LayoutBox, list: &mut DisplayList);

// Paint with active text selection highlight
pub fn paint_layout_with_selection(
    root: &LayoutBox, list: &mut DisplayList,
    selection: Option<&TextSelection>,
    selection_colors: SelectionColors,
);
```

## Background Painting

For each box with a non-None `background`:

1. The `background_rect` (driven by `background-clip`) is used as the quad rectangle.
2. `background_radii` (already reduced per clip rules) is used for corner rounding.
3. Color is converted from sRGB to linear space before emission.
4. A filled quad (`stroke = [0; 4]`) is pushed with the background color.

```rust
// Simplified: solid background
list.push_quad(Quad {
    rect: box_.background_rect,
    color: resolve_color(box_.background.unwrap()),
    radii_h: box_.background_radii,
    radii_v: box_.background_radii,
    stroke: [0.0; 4],
    pattern: [0.0; 4],
});
```

## Border Painting

Border painting emits `Quad` commands with non-zero `stroke` fields:

### Uniform Solid

When all four sides share the same color and style, a **single SDF ring quad** is emitted. All four stroke widths are non-zero; the shader paints the full ring.

### Mixed Per-Side

When sides differ in color or style, **per-side one-sided ring quads** are emitted — one per edge, each with exactly one positive stroke component:

```rust
// Top border
list.push_quad(Quad {
    rect: border_rect,
    stroke: [border_w_top, 0.0, 0.0, 0.0],
    radii_h, radii_v, pattern,
});
// Right border, Bottom, Left...
```

### Border Styles

| Style | Rendering |
|---|---|
| `solid` | Continuous SDF ring |
| `dashed` | Dashed pattern: `(1, dash_len, gap_len, _)` |
| `dotted` | Dotted pattern: `(2, 0.01, gap_len, _)` |
| `none`, `hidden` | Skipped entirely |
| `double`, `groove`, `ridge`, `inset`, `outset` | Solid fallback |

Dashed/dotted patterns use a segment-loop for straight edges. For rounded corners, uniform circular rings with the pattern descriptor produce correct pattern continuation.

## Text Painting

Glyph quads are emitted from `ShapedRun::glyphs`:

```rust
for glyph in &run.glyphs {
    list.push_glyph(GlyphQuad {
        rect: Rect::new(box_x + glyph.x, box_y + glyph.y, glyph.w, glyph.h),
        color: glyph.color,
        uv_min: glyph.uv_min,
        uv_max: glyph.uv_max,
    });
}
```

Text decorations (underline, line-through, overline) emit thin filled quads at the appropriate y-offset relative to the text baseline.

## Overflow Clipping

```rust
struct ClipFrame {
    rect: Rect,
    radii_h: [f32; 4],
    radii_v: [f32; 4],
}
```

Paint maintains a clip stack. When a box has `overflow: hidden | scroll | auto`:
1. `list.push_clip(Rect::new(...), radii_h, radii_v)` pushes a new clip slot.
2. All descendants pushed afterward get `clip_index` pointing to this slot.
3. On exiting the box: `list.pop_clip()` restores the previous clip.

The clip stack intersects: a descendant inside two `overflow: hidden` ancestors draws with the innermost clip. The renderer applies scissor rects for rectangular clips; rounded clips additionally run the SDF discard in the fragment shader.

## Opacity

Opacity is inherited multiplicatively. Each box's `opacity` is `parent_accumulated × box_.opacity`. The paint stage applies opacity by scaling the RGBA alpha:

```rust
fn apply_opacity(mut color: Color, opacity: f32) -> Color {
    color[3] *= opacity.clamp(0.0, 1.0);
    color
}
```

This is applied to background, border, and text colors alike.

## Image Painting

### `<img>` Elements

`LayoutBox::image` contains decoded RGBA data. A single image quad is emitted covering the content rect:

```rust
list.push_image(ImageQuad {
    rect: box_.content_rect,
    uv_min: [0.0, 0.0],
    uv_max: [1.0, 1.0],
    image_id: image.image_id,
    tint: [1.0; 4],
});
```

### background-image

`LayoutBox::background_image` carries pre-computed tile rectangles. For `background-repeat`, multiple image quads are emitted — one per tile position. For `no-repeat`, a single quad. All tiles are clipped to `background_rect` during layout.

## DisplayList::finalize()

After paint, `finalize()` does two things:

1. **Empty clip range removal**: Clips that enclose no visible commands are `retain`-dropped.
2. **Index remapping**: Every `DisplayCommand::clip_index` is patched to match the new slot positions after removal. Commands whose clip was dropped fall back to the nearest surviving predecessor.

This is critical: without the remap, glyphs after an overflow container would reference stale indices and render invisible — a known-fixed bug documented in `AGENTS.md`.

```rust
list.finalize();  // Must call before passing to Renderer
```
