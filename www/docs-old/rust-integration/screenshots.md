---
title: Screenshots
---

# Screenshots

lui supports capturing rendered output to PNG files, both from the live surface and off-screen.

## F12 → PNG Screenshot

In the winit harness (`LuiWindow`), pressing F12 saves a PNG of the full viewport:

```rust
create_window(&mut tree)
    .with_screenshot_key(winit::keyboard::KeyCode::F12)
    .run()
    .unwrap();
```

The file is written to `screenshot_0001.png` (incrementing counter) in the current directory. The key can be changed via `with_screenshot_key()` or disabled by passing `None`.

## Off-Screen Capture (Full)

```rust
use lui::renderer::Renderer;

renderer.capture_to(
    &display_list,
    1280,  // width (physical pixels)
    720,   // height
    "output.png",
)?;
```

`capture_to()` allocates an off-screen render target at the given dimensions, replays the full display list into it, and writes the result as a PNG. This works regardless of the surface size — useful for headless rendering.

## Off-Screen Capture (Region)

```rust
use lui::renderer::Rect;

let region = Rect::new(100.0, 50.0, 400.0, 300.0);
renderer.capture_rect_to(
    &display_list,
    region,
    "region.png",
)?;
```

`capture_rect_to()` is equivalent to `capture_to(&list.translated(-region.x, -region.y), region.w, region.h, path)`. It captures exactly the specified rectangular area — even if it lies outside the visible viewport.

## Single-Element Screenshot

```rust
use lui::screenshot_node_to;

screenshot_node_to(
    &tree,
    &mut text_ctx,
    &mut image_cache,
    &mut renderer,
    &[0, 2, 1],  // path to the target element
    1280.0,       // viewport_w
    720.0,        // viewport_h
    1.0,          // scale
    "element.png",
)?;
```

This captures a single element at full size, including content that extends beyond the visible viewport. It works by:

1. Running cascade + layout against the full viewport dimensions.
2. Finding the target element's `border_rect` in the layout.
3. Calling `capture_rect_to()` with that region.

The element is captured with its rendered appearance — backgrounds, borders, text, child elements.

## Error Handling

```rust
pub enum ScreenshotError {
    Io(std::io::Error),
    Image(image::ImageError),
}

pub enum NodeScreenshotError {
    NoLayout,            // tree produced no layout
    NodeNotFound(Vec<usize>),  // path not found in layout
    EmptyRect,           // node has empty border rect
    Render(ScreenshotError),
}
```

## Implementation Detail

Captures work by:

1. Creating an off-screen texture at the target size.
2. Running the full render pass (clear → quads → images → glyphs) into it.
3. Copying the texture to a staging buffer.
4. Reading the buffer back from GPU and encoding as PNG.

The surface format (sRGB or non-sRGB) is preserved in the capture, so PNG output matches what appears on screen.

## Programmatic Screenshot from Winit Harness

Inside an `AppHook::on_key`:

```rust
fn on_key(&mut self, ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse {
    if event.logical_key == "F12" {
        if let Some(layout) = ctx.last_layout {
            let list = lui::paint::paint_tree_with_text(
                ctx.tree, ctx.text_ctx, ctx.image_cache,
                ctx.window.inner_size().width as f32,
                ctx.window.inner_size().height as f32,
                ctx.window.scale_factor() as f32,
            );
            ctx.renderer.capture_to(
                &list,
                ctx.window.inner_size().width,
                ctx.window.inner_size().height,
                "my_screenshot.png",
            ).ok();
        }
        return EventResponse::Stop;
    }
    EventResponse::Continue
}
```
