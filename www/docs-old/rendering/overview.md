---
title: Rendering Pipeline Overview
---

# Rendering Pipeline Overview

The renderer (`wgpu-html-renderer`) converts a `DisplayList` into GPU draw calls. It owns the wgpu device, surface, and three GPU pipelines.

## DisplayList

```rust
pub enum DisplayCommandKind {
    Quad(Quad),
    Glyph(GlyphQuad),
    Image(ImageQuad),
}

pub struct DisplayList {
    commands: Vec<DisplayCommand>,
    clips: Vec<ClipSlot>,
    glyph_range: Option<(usize, usize)>,
    quad_range: Option<(usize, usize)>,
    image_range: Option<(usize, usize)>,
}
```

Each `DisplayCommand` carries:
- A `DisplayCommandKind` (Quad, Glyph, or Image).
- A `clip_index` — which scissor/clip slot constrains it.
- The absolute z-order (push-order through paint traversal).

## Three GPU Pipelines

| Pipeline | Shader | What it draws |
|---|---|---|
| **Quad** (`quad.wgsl`) | Instanced SDF quads | Backgrounds, borders, scrollbars, decorations |
| **Glyph** (`glyph.wgsl`) | Instanced textured quads | Text glyphs from the R8 atlas |
| **Image** | Instanced textured quads | `<img>` elements, `background-image` |

## Single Render Pass

Each frame renders in one pass (within the same command encoder):

```
1. Clear render target to clear_color
2. Quad pipeline → draw all quads
3. Image pipeline → draw all images
4. Glyph pipeline → draw all glyphs
```

Drawing order preserves the DisplayList's z-order within each command range. The render pass is structured so text appears on top of backgrounds/borders.

## sRGB Surface

The surface texture uses an sRGB format by default (selected from `wgpu::SurfaceCapabilities`). This means:
- Colors written by the quad pipeline are stored in sRGB-encoded bytes.
- The glyph pass uses a **non-sRGB view** of the same texture for gamma-correct display-space blending — anti-aliased text multiplies coverage × color in linear space, then the GPU's sRGB encode on write (or lack thereof, for the glyph view) produces correct visual results.

## Vsync

```rust
present_mode: wgpu::PresentMode::AutoVsync,
```

The surface uses VSync by default. This can be changed by the host before calling `surface.configure()`.

## Screenshot Capabilities

The renderer supports off-screen capture:

```rust
renderer.capture_to(&display_list, width, height, "frame.png")?;
renderer.capture_rect_to(&display_list, region, "region.png")?;
```

Captures allocate an off-screen render target, replay the entire display list into it, and write the result as a PNG. This works even for regions outside the visible viewport.
