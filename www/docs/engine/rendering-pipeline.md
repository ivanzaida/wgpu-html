---
sidebar_position: 2
---

# Rendering Pipeline

The GPU rendering stage is the final step in the pipeline. It consumes a `DisplayList` (from `lui-display-list`) and produces a frame via any `RenderBackend` implementation.

## Architecture

The rendering layer is split into three crates:

- **`lui-display-list`** — backend-agnostic IR (`DisplayList`, `Quad`, `GlyphQuad`, `ImageQuad`, `ClipRange`, `Rect`, `Color`, `FrameOutcome`)
- **`lui-render-api`** — the `RenderBackend` trait that every GPU renderer implements
- **`lui-renderer-wgpu`** — the wgpu reference backend

## Renderer Setup (wgpu)

The wgpu backend (`renderers/lui-renderer-wgpu/src/lib.rs`) is created with:

```rust
use lui_renderer_wgpu::Renderer;

let renderer = pollster::block_on(Renderer::new(window, width, height));
```

This acquires a wgpu `Instance`, `Adapter`, `Device`, `Queue`, and `Surface`, then initializes the three GPU pipelines.

## Three GPU Pipelines

### Quad Pipeline (`quad_pipeline.rs`)

Instanced rectangle rendering with SDF (Signed Distance Field) shading:
- **Background fills** — solid colors, with optional rounded corners via SDF
- **Borders** — solid, dashed, and dotted stroke patterns per side
- **Selection highlights** — text selection background quads
- **Scrollbars** — track and thumb rectangles

The WGSL shader (`quad.wgsl`) computes per-pixel SDF distance for rounded rectangles, enabling smooth anti-aliased corners and borders.

### Glyph Pipeline (`glyph_pipeline.rs`)

Text rendering from the glyph atlas:
- **Instanced glyph quads** — one instance per glyph, carrying UV coords, position, and color
- **Alpha-tested** — discards fragments below a coverage threshold from the atlas texture
- **Dynamic offset** — per-draw uniform buffer for clip rect and radii

### Image Pipeline (`image_pipeline.rs`)

Textured rectangle rendering:
- **`<img>` elements** — decoded images uploaded as GPU textures
- **Background images** — CSS `background-image` (URLs and gradients)
- **Animated GIF/WebP** — frame selection driven by a process-wide clock

## DisplayList Consumption

`Renderer::render(&DisplayList)` walks the display list and issues draw commands:

1. Iterate `DisplayList::quads` for background/fill commands
2. Iterate `DisplayList::glyphs` for text commands
3. Iterate `DisplayList::images` for image commands
4. For each clip range, pre-compute scissor rects

## Clipping System

Overflow clipping uses a two-level approach:
1. **CPU scissor pre-pass** — `set_scissor_rect` clips to rectangular bounds
2. **GPU SDF discard** — fragment shader discards pixels outside rounded masks for `border-radius` clipping
3. **Clip ranges** — quad/glyph instances are partitioned into ranges, each tagged with a clip index

## Screen Capture

```rust
renderer.capture_to(vec);        // Capture full surface
renderer.capture_rect_to(vec, r); // Capture a region
```

Outputs RGBA8 pixel data as a `Vec<u8>`. The winit harness uses this for F12 screenshots.

## Performance Notes

- **Instance buffers** grow in powers of two, avoiding per-frame allocation
- **Uniform buffers** use dynamic offsets to pack multiple draw calls into one binding
- **Draw calls** are batched by pipeline (quad → glyph → image) within each clip range
