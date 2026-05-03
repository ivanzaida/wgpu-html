---
title: egui Backend
---

# egui Backend

The `wgpu-html-egui` crate provides an alternative integration path for embedding wgpu-html content inside egui/eframe applications.

## Overview

Instead of owning a window and event loop (like `wgpu-html-winit`), the egui backend runs inside an existing egui frame. It allocates an egui region, forwards pointer/key events into a `Tree`, and produces a `DisplayList` that can be consumed by either:

1. A lightweight egui painter (CSS box quads only).
2. The full `wgpu-html-renderer` pipelines (for GPU-accurate glyph and image rendering).

## HtmlState

```rust
pub struct HtmlState {
    text_ctx: TextContext,
    image_cache: ImageCache,
    display_list: DisplayList,
    last_layout: Option<LayoutBox>,
    timings: PipelineTimings,
}
```

`HtmlState` is the stateful host for one HTML document. Create one per embedded document:

```rust
let mut state = HtmlState::new();
```

## Rendering Inside egui

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Reserving space for the HTML content
        let desired_size = egui::vec2(800.0, 600.0);

        let output = self.html_state.show(
            ui, &mut self.tree, desired_size,
        );

        // Access the DisplayList for GPU rendering
        let display_list = output.display_list();
    });
}
```

`HtmlState::show()`:
1. Allocates an egui region with `Sense::click_and_drag()`.
2. Syncs modifier state from egui to the tree.
3. Forwards pointer events and keyboard input.
4. Runs `paint_tree_returning_layout_profiled()`.
5. Returns the `DisplayList`, `LayoutBox`, and timings.

## --renderer=egui Flag

The demo crate (`wgpu-html-demo`) supports an `--renderer=egui` CLI flag that switches the rendering backend:

```bash
cargo run -p wgpu-html-demo -- --renderer=egui
```

This uses the egui backend to render in an eframe window rather than the native winit harness.

## Use Cases

- **Developer tools**: Embed wgpu-html as a property inspector or log viewer alongside egui controls.
- **Game UI**: Use egui for debug overlays and wgpu-html for styled game menus in the same window.
- **Mixed UI**: egui handles immediate-mode controls (sliders, buttons) while wgpu-html handles document-style UI (help pages, settings panels).

## Limitations

- **Glyph rendering**: The built-in egui painter draws only box quads. For accurate text, consume `HtmlState::display_list()` with your own `wgpu-html-renderer`.
- **Image rendering**: Same limitation — images need the full renderer pipeline.
- **No viewport scroll**: The egui region is sized to the content; overflow behavior is up to the host.
- **Single document**: `HtmlState` holds one `Tree` and one `LayoutBox`. Multiple documents need multiple `HtmlState` instances.
