---
sidebar_position: 6
---

# Native Apps

Tips for building native desktop applications with wgpu-html.

## Minimal App Template

```rust
use wgpu_html_parser::parse;
use wgpu_html_driver_winit::WinitDriver;
use winit::event_loop::EventLoop;
use winit::window::Window;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"<html><body style="margin:0; font-family:sans-serif;">
        <h1>My App</h1>
    </body></html>"#;

    let mut tree = parse(html);
    tree.register_system_fonts("sans-serif");

    let event_loop = EventLoop::new()?;
    let window = Arc::new(event_loop.create_window(
        Window::default_attributes().with_title("My App")
    )?);
    let mut driver = WinitDriver::bind(window, tree);

    event_loop.run_app(&mut App { driver })?;
    Ok(())
}

struct App { driver: WinitDriver }

impl ApplicationHandler for App {
    fn window_event(&mut self, event_loop: &ActiveEventLoop,
                    _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::RedrawRequested => {
                self.driver.handle_event(&event);
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => { self.driver.handle_event(&event); }
        }
    }
}
```

## Application Structure

For non-trivial apps, separate concerns:

```
src/
├── main.rs          # Event loop + driver setup
├── ui.rs            # HTML templates + Tree construction
├── app_state.rs     # Application state (shared data)
├── callbacks.rs     # Event handlers (on_click, on_input, etc.)
└── assets.rs        # Font registration + image preloading
```

## Thread Safety

- `Tree` is NOT `Send` or `Sync` — all mutations happen on the main thread
- Callbacks use `Arc<dyn Fn + Send + Sync>` — closures must be thread-safe
- Image loading uses a background worker pool but returns results on the main thread via `ImageCache`

## Performance Notes

- The `PipelineCache` avoids redundant cascade and layout when only interaction state changed
- For dynamic content, use `tree.append_node()` and `tree.remove_node()` rather than re-parsing
- `tree.insert_node()` and template content insertion trigger incremental layout rather than full re-layout
- Use `tree.preload_asset()` for images that will be shown, to avoid pop-in

## Packaging

- Bundle font files with your application (or use system fonts)
- Bundle any local image assets referenced in HTML
- Consider embedding HTML/CSS as compile-time `include_str!()` for distribution
- For production builds, use `cargo build --release` to get optimized GPU shader compilation
