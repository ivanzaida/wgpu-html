---
title: Winit Harness (WgpuHtmlWindow)
---

# Winit Harness (WgpuHtmlWindow)

The `wgpu-html-winit` crate provides a batteries-included window harness. It handles window setup, input forwarding, focus management, scrolling, clipboard, and screenshots.

## One-Call Setup

```rust
use wgpu_html_winit::create_window;

let mut tree = Tree::new(root_node);

create_window(&mut tree)
    .with_title("My wgpu-html App")
    .with_size(1280, 720)
    .with_exit_on_escape(true)
    .with_clipboard_enabled(true)
    .with_screenshot_key(winit::keyboard::KeyCode::F12)
    .with_hook(MyHook)
    .run()
    .unwrap();
```

## WgpuHtmlWindow

Wraps `&mut Tree` and provides chainable configuration:

| Method | Default | Description |
|---|---|---|
| `with_title(s)` | `"wgpu-html"` | Window title |
| `with_size(w, h)` | `(1280, 720)` | Logical size |
| `with_exit_on_escape(b)` | `true` | Esc closes the window |
| `with_clipboard_enabled(b)` | `true` | Ctrl+A/Ctrl+C support |
| `with_screenshot_key(k)` | `F12` | Screenshot key |
| `with_hook(h)` | None | `AppHook` callbacks |
| `run()` | — | Starts the event loop (blocking) |

## AppHook Trait

```rust
pub trait AppHook {
    fn on_key(&mut self, ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse { ... }
    fn on_frame(&mut self, ctx: HookContext<'_>, timings: &FrameTimings) { ... }
    fn on_pointer_move(&mut self, ctx: HookContext<'_>, pointer_move_ms: f64, changed: bool) { ... }
    fn on_idle(&mut self) { ... }
    fn on_window_event(&mut self, ctx: HookContext<'_>, window_id: WindowId, event: &WindowEvent) -> bool { ... }
}
```

- **on_key**: Called before built-in keyboard handling. Return `EventResponse::Stop` to skip defaults.
- **on_frame**: Called after each GPU frame submission. Receives timing breakdown.
- **on_pointer_move**: Called after pointer dispatch.
- **on_idle**: Called once per event-loop iteration after all events are dispatched.

## HookContext

```rust
pub struct HookContext<'a> {
    pub tree: &'a mut Tree,
    pub renderer: &'a mut Renderer,
    pub text_ctx: &'a mut TextContext,
    pub image_cache: &'a mut ImageCache,
    pub last_layout: Option<&'a LayoutBox>,
    pub window: &'a Arc<Window>,
    pub event_loop: &'a ActiveEventLoop,
}
```

## FrameTimings

```rust
pub struct FrameTimings {
    pub frame_index: u64,
    pub cascade_ms: f64,
    pub layout_ms: f64,
    pub paint_ms: f64,
    pub render_ms: f64,
    pub total_ms: f64,
}
```

## Built-In Features

| Feature | Key/Behavior |
|---|---|
| Viewport scroll | Mouse wheel |
| Scrollbar drag | Click-drag on the scrollbar thumb |
| Clipboard | Ctrl+A select all, Ctrl+C copy |
| Screenshot | F12 → `screenshot_*.png` |
| Tab navigation | Tab / Shift+Tab focus cycling |
| Escape exit | Esc closes the window |

## Type Translators

```rust
pub fn mouse_button(button: WinitMouseButton) -> wgpu_html_tree::MouseButton;
pub fn key_to_dom_key(key: &str) -> String;
pub fn keycode_to_dom_code(code: KeyCode) -> String;
pub fn keycode_to_modifier(code: KeyCode) -> Option<Modifier>;
```

These map winit's key/mouse types to wgpu-html's internal types.

## System Font Discovery

```rust
use wgpu_html_winit::discover_system_fonts;

let variants = discover_system_fonts();  // Vec<SystemFontVariant>
```

Convenience wrappers around `system_font_variants()` and `register_system_fonts()`.

## Complete Example

```rust
use wgpu_html::{parser, tree::{Tree, Node, Element}};
use wgpu_html_winit::create_window;

let html = r#"<html><body>
    <h1>Hello from wgpu-html!</h1>
    <button id="btn">Click me</button>
</body></html>"#;

let document = parser::parse(html);
let mut tree = Tree::new(Node::root(document));

// Register fonts
wgpu_html::tree::register_system_fonts(&mut tree, "sans-serif");

// Hook to handle custom logic
struct MyApp;
impl wgpu_html_winit::AppHook for MyApp {
    fn on_frame(&mut self, ctx: wgpu_html_winit::HookContext<'_>, t: &wgpu_html_winit::FrameTimings) {
        // Called every frame — access tree, renderer, layout
    }
}

create_window(&mut tree)
    .with_title("My App")
    .with_size(1024, 768)
    .with_hook(MyApp)
    .run()
    .unwrap();
```
