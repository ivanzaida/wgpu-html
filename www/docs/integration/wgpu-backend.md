---
sidebar_position: 2
---

# wgpu Backend

`lui-renderer-wgpu` is the reference implementation of the `RenderBackend` trait. It manages a wgpu device, surface, and three GPU pipelines.

## Render Backend Architecture

Rendering in lui is split into three layers:

| Crate | Role |
|---|---|
| `lui-display-list` | Backend-agnostic IR: `DisplayList`, `Quad`, `GlyphQuad`, `ImageQuad`, `Rect`, `Color` |
| `lui-render-api` | `RenderBackend` trait that any GPU renderer must implement |
| `lui-renderer-wgpu` | wgpu implementation (the default backend) |

Any crate implementing `RenderBackend` can be plugged into the `Runtime` — the engine, layout, text, and paint stages never touch GPU types.

## Backend Selection

The wgpu backend uses `wgpu::Backends::PRIMARY`, which selects the best available backend:

| Platform | Backend |
|---|---|
| Windows | DirectX 12 (or Vulkan) |
| Linux | Vulkan |
| macOS | Metal |
| Web (via wasm) | WebGPU or WebGL |

## Surface Configuration

```rust
use lui_renderer_wgpu::Renderer;

let renderer = pollster::block_on(Renderer::new(window, 800, 600));
```

The surface is configured with:
- **Format**: `Bgra8UnormSrgb` (Windows) or `Rgba8UnormSrgb` (others)
- **Present mode**: `Fifo` (vsync)
- **Alpha mode**: `Opaque`

## Three GPU Pipelines

### Quad Pipeline
- Instanced rectangles with SDF for rounded corners and borders
- BGRA8 sRGB surface, alpha blending
- Dynamic offset uniforms per clip range
- WGSL shader: `quad.wgsl`

### Glyph Pipeline
- Instanced glyph quads with alpha-tested atlas sampling
- Non-sRGB surface view (blending in display space)
- Per-glyph color, linear filtering
- WGSL shader: `glyph.wgsl`

### Image Pipeline
- Textured quads for `<img>` and background images
- sRGB surface, alpha blending with per-image opacity
- Linear filtering, clamp-to-edge addressing
- WGSL shader: `image.wgsl`

## Headless Rendering

For off-screen rendering (no window, e.g., for screenshots or server-side rendering):

```rust
let renderer = pollster::block_on(Renderer::headless());
let rgba_bytes = renderer.render_to_rgba(&display_list, 800, 600)?;
```

The headless renderer allocates a texture (no surface), renders to it, and reads back the RGBA8 pixels synchronously.

## Dynamic Buffers

- **Instance buffers** — grow in powers of two, avoiding per-frame allocation
- **Uniform buffers** — recreated on resize
- **Glyph atlas** — 2048×2048 R8Unorm texture, CPU-uploaded each frame

## Resize Handling

```rust
renderer.resize(new_width, new_height);
```

Recreates the surface configuration and uniform buffers. Called automatically by winit `Resized` events via the driver.

## Frame Outcome

```rust
pub enum FrameOutcome {
    Presented,
    Reconfigure,  // surface lost, must reconfigure
    Skipped,       // nothing to draw
}
```

The driver handles `Reconfigure` by calling `renderer.resize()` and retrying.

## Screen Capture

```rust
// Full frame screenshot
renderer.capture_to(&display_list, 800, 600, "screenshot.png")?;

// Rectangular region
renderer.capture_rect_to(&display_list, Rect::new(x, y, w, h), "region.png")?;

// Raw RGBA8 bytes
let pixels: Vec<u8> = renderer.render_to_rgba(&display_list, 800, 600)?;

// Schedule next render() to save
renderer.capture_next_frame_to("next_frame.png");
```

The demo also supports F12 for screenshots and `make_screenshot <query>` via STDIN.
