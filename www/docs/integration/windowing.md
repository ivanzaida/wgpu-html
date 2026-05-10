---
sidebar_position: 3
---

# Windowing

lui uses winit for windowing. The `lui-driver-winit` crate provides a complete window driver.

## WinitDriver

```rust
use lui_driver_winit::WinitDriver;

let event_loop = EventLoop::new()?;
let window = Arc::new(event_loop.create_window(
    Window::default_attributes()
        .with_title("My App")
        .with_inner_size(PhysicalSize::new(800, 600))
)?);

let mut driver = WinitDriver::bind(window, tree);
```

## Event Handling

`WinitDriver::handle_event()` handles all window events automatically:

```rust
// In ApplicationHandler::window_event():
match event {
    WindowEvent::RedrawRequested => {
        driver.handle_event(&event);
    }
    WindowEvent::CursorMoved { .. } |
    WindowEvent::MouseInput { .. } |
    WindowEvent::MouseWheel { .. } |
    WindowEvent::KeyboardInput { .. } |
    WindowEvent::Resized { .. } |
    WindowEvent::ScaleFactorChanged { .. } => {
        driver.handle_event(&event);
    }
    _ => {}
}
```

Returns `Option<PipelineTimings>` on render frames for profiling.

## Built-in Features

The winit driver includes:

| Feature | Key/Trigger |
|---|---|
| Viewport scrolling | Mouse wheel |
| Scrollbar paint + drag | Mouse on scrollbar |
| Text selection | Drag-select |
| Clipboard copy | Ctrl+C |
| Clipboard paste | Ctrl+V |
| Clipboard cut | Ctrl+X |
| Select all | Ctrl+A |
| Screenshot | F12 (configurable) |
| Devtools toggle | F11 (when `Devtools::attach()` used) |
| Profiler dump | F9 (when profiler enabled) |
| Exit | Escape (configurable) |
| System font discovery | Automatic at startup |

## Multiple Windows

```rust
let devtools_win = Arc::new(event_loop.create_window(
    Window::default_attributes().with_title("Devtools")
)?);
let mut dd = WinitDriver::bind(devtools_win, Tree::default());

// In event loop, dispatch to both:
driver.handle_event(&event);
dd.dispatch_to(&event, devtools.tree_mut());
dd.render(devtools.tree_mut());
```

## DPI Handling

DPI scale factor is tracked from the winit window. The engine stores a `dpi_scale_override` on the tree for testing:

```rust
tree.set_dpi_scale_override(Some(1.5)); // Force 1.5x DPI
let effective = tree.effective_dpi_scale(host_scale); // Resolves override vs host
```

Physical pixel coordinates in layout and paint account for the DPI scale factor.

## Cursor Management

CSS `cursor` property values are mapped to OS cursor icons:

| CSS Cursor | OS Cursor |
|---|---|
| `pointer` | Hand |
| `text` | I-beam |
| `move` | Move/SizeAll |
| `default` | Arrow |
| `not-allowed` | NotAllowed |
| `crosshair` | Crosshair |
| `col-resize` | SizeWe |
| `row-resize` | SizeNs |
| etc. | etc. |
