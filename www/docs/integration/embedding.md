---
sidebar_position: 1
---

# Embedding lui

lui is designed to be embedded inside a Rust application. The core library is windowing-system agnostic — you connect it to any window via the `Driver` trait or use the provided drivers.

## Architecture

```
Your Application
    │
    ├── Tree (DOM + fonts + callbacks)
    ├── Runtime<YourDriver> (engine + GPU state)
    └── YourDriver (winit / egui / Bevy / custom)
```

## Step 1: Create a Tree

```rust
use lui_parser::parse;

let mut tree = parse(r#"<html><body>
    <div style="padding: 20px; font-family: sans-serif;">
        <h1>Hello, World!</h1>
    </div>
</body></html>"#);

tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Regular.ttf".into(),
    ..Default::default()
});
```

## Step 2: Choose a Driver

### winit (desktop)

```rust
use lui_driver_winit::WinitDriver;

let event_loop = EventLoop::new()?;
let window = Arc::new(event_loop.create_window(Window::default_attributes())?);
let mut driver = WinitDriver::bind(window, tree);

// In your event loop:
driver.handle_event(&event); // dispatches + renders
```

### egui / eframe

```rust
use lui_driver_egui::EguiRunner;

let mut runner = EguiRunner::new(window, 800, 600);

egui::CentralPanel::default().show(ctx, |ui| {
    let output = runner.show(ui, &mut tree, ui.available_size());
    if output.layout_available {
        // Submit output.display_list to GPU
    }
});
```

### Bevy

```rust
use lui_driver_bevy::{LuiPlugin, HtmlOverlay};

app.add_plugins(LuiPlugin);

fn setup(mut commands: Commands) {
    commands.insert_resource(HtmlOverlay::new());
}
```

## Step 3: Wire Input

```rust
// winit example: translate WindowEvents to engine input
match event {
    WindowEvent::CursorMoved { position, .. } => {
        runtime.on_pointer_move(&mut tree, position.x, position.y);
    }
    WindowEvent::MouseInput { button, state, .. } => {
        let pressed = state == ElementState::Pressed;
        runtime.on_mouse_button(&mut tree, button.into(), pressed);
    }
    WindowEvent::KeyboardInput { event, .. } => {
        runtime.on_key(&mut tree, key, code, pressed, repeat, text);
    }
    WindowEvent::MouseWheel { delta, .. } => {
        runtime.on_wheel_event(&mut tree, x, y, dx, dy, mode);
    }
}
```

The `WinitDriver::handle_event()` does all of this automatically for winit windows.

## Step 4: Render

Each frame, the engine runs cascade → layout → paint → GPU render:

```rust
let timings = runtime.render_frame(&mut tree);
println!("Frame: cascade={:.2}ms layout={:.2}ms paint={:.2}ms",
    timings.cascade_ms, timings.layout_ms, timings.paint_ms);
```

## Custom Drivers

Implement the `Driver` trait from `lui-driver`:

```rust
impl Driver for MyWindow {
    type Surface = MySurface;

    fn surface(&self) -> &Arc<Self::Surface> { &self.surface }
    fn inner_size(&self) -> (u32, u32) { (self.width, self.height) }
    fn scale_factor(&self) -> f64 { self.dpi }
    fn request_redraw(&self) { /* request new frame */ }
    fn set_cursor(&self, cursor: Cursor) { /* set OS cursor */ }
    // Optional: clipboard, profiling
}
```

Then wrap with `Runtime<MyWindow>`:

```rust
let mut runtime = Runtime::new(my_window, 800, 600);
runtime.render_frame(&mut tree);
```
