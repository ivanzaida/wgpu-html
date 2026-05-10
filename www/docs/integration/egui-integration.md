---
sidebar_position: 4
---

# egui Integration

The `wgpu-html-driver-egui` crate lets you embed wgpu-html inside an egui or eframe application.

## Basic Usage

```rust
use wgpu_html_driver_egui::EguiRunner;

// Window must implement HasWindowHandle + HasDisplayHandle
let mut runner = EguiRunner::new(window, 800, 600);

// In your egui frame:
egui::CentralPanel::default().show(ctx, |ui| {
    let output = runner.show(ui, &mut tree, ui.available_size());
    if output.layout_available {
        // Submit output.display_list to your wgpu render pass
        // output.response contains the egui Response for interaction
    }
});
```

## HtmlOutput

```rust
pub struct HtmlOutput {
    pub response: Response,           // egui response (focus, interaction)
    pub display_list: DisplayList,    // draw command list for GPU
    pub layout_available: bool,       // false if tree is empty
    pub timings: PipelineTimings,     // cascade/layout/paint timing
}
```

## Input Forwarding

For proper keyboard and focus handling, forward egui input events:

```rust
// In each frame:
forward_input_events(&ui, &mut tree);

// Or manually:
forward_key(&mut tree, egui::Key::Enter, true, false); // key press
```

## Pointer Translation

```rust
use wgpu_html_driver_egui::pointer_button;

let mouse_button: MouseButton = pointer_button(egui::PointerButton::Primary);
```

## Limitations

- The egui driver renders wgpu-html content into a texture that egui draws as an image widget. There's one extra texture copy compared to native winit rendering.
- Focus management requires explicit forwarding since egui and wgpu-html have separate focus systems.
- The egui driver does not handle OS-level clipboard directly; use the tree's clipboard events instead.
