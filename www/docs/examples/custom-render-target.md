---
sidebar_position: 5
---

# Custom Render Target

Rendering lui content to an off-screen texture for use in custom graphics pipelines.

## Headless Renderer

```rust
use lui_renderer_wgpu::Renderer;

let mut renderer = pollster::block_on(Renderer::headless());

// Build your display list via paint_tree_with_text()
let display_list = lui::paint_tree_with_text(
    &mut tree, &mut text_ctx, &mut image_cache,
    800, 600, 1.0, 0.0,
);

// Render to RGBA8 bytes (no window needed)
let pixels: Vec<u8> = renderer.render_to_rgba(&display_list, 800, 600)?;

// Use pixels in your graphics pipeline (upload as texture, save, etc.)
```

## Screenshot a Specific Element

```rust
use lui::screenshot_node_to;

screenshot_node_to(
    &mut tree, &mut text_ctx, &mut image_cache,
    &mut renderer, &["div.card", "p"].into_iter().map(|s| s.parse()).collect(),
    800, 600, 1.0,
    "element_screenshot.png",
)?;
```

## Using the RenderBackend Trait

Any type implementing `RenderBackend` can render a `DisplayList`. The wgpu backend is the default, but the trait is designed for pluggable backends:

```rust
use lui_render_api::RenderBackend;

fn render_frame(backend: &mut impl RenderBackend, list: &DisplayList) {
    backend.set_clear_color([1.0, 1.0, 1.0, 1.0]);
    backend.render(list);
}
```

## Use Cases

- **Off-screen compositing** — render a lui surface, then blend it into a 3D scene
- **Texture generation** — generate UI textures for game engines
- **CI screenshots** — automated visual regression testing
- **Export** — render to PNG/SVG without a window
