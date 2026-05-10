---
title: Integration Guide
---

# Integration Guide

This guide walks through embedding lui in a custom Rust application from scratch, with full control over the event loop.

## Step 1: Add Dependencies

```toml
[dependencies]
lui = "0.1"
winit = "0.30"
```

## Step 2: Parse HTML/CSS

```rust
use lui::{parser, tree::{Tree, Node, Element}};

let html = r#"
    <!DOCTYPE html>
    <html><head><style>
        body { margin: 0; font-family: sans-serif; }
        h1 { color: #333; }
        .container { display: flex; gap: 16px; padding: 20px; }
    </style></head>
    <body>
        <h1>Hello lui</h1>
        <div class="container">
            <div style="background:#e0e0e0;padding:16px;">Box 1</div>
            <div style="background:#d0d0d0;padding:16px;">Box 2</div>
        </div>
    </body></html>
"#;

let document = parser::parse(html);
let mut tree = Tree::new(Node::root(document));
```

## Step 3: Register Fonts

```rust
use std::sync::Arc;
use lui::tree::{FontFace, FontStyleAxis, register_system_fonts};

// Register system fonts for the sans-serif family
register_system_fonts(&mut tree, "Arial");
register_system_fonts(&mut tree, "Helvetica");
register_system_fonts(&mut tree, "DejaVu Sans");

// Or register a custom font
tree.register_font(FontFace {
    family: "Inter".into(),
    weight: 400,
    style: FontStyleAxis::Normal,
    data: Arc::from(include_bytes!("Inter-Regular.otf") as &[u8]),
});
```

## Step 4: Register Linked Stylesheets

```rust
tree.register_linked_stylesheet(
    "styles/main.css",
    "body { background: #fafafa; }"
);
```

## Step 5: Set Up Callbacks

```rust
use lui::tree::MouseEvent;

if let Some(btn) = tree.get_element_by_id("my-button") {
    btn.on_click = Some(Arc::new(|ev: &MouseEvent| {
        println!("Clicked at {:?}!", ev.pos);
    }));
}
```

## Step 6: Create the Render Loop

```rust
use lui::renderer::Renderer;
use lui::text::TextContext;
use lui::layout::ImageCache;

async fn run() {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = Arc::new(event_loop.create_window(
        winit::window::WindowAttributes::default()
            .with_title("lui App")
            .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)),
    ).unwrap());

    let mut renderer = Renderer::new(window.clone(), 1280, 720).await;
    let mut text_ctx = TextContext::new(lui::renderer::GLYPH_ATLAS_SIZE);
    let mut image_cache = ImageCache::new();

    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

        match event {
            winit::event::Event::WindowEvent { event: we, .. } => match we {
                winit::event::WindowEvent::RedrawRequested => {
                    let scale = window.scale_factor() as f32;
                    let size = window.inner_size();

                    let (list, layout) = lui::paint_tree_returning_layout(
                        &tree, &mut text_ctx, &mut image_cache,
                        size.width as f32, size.height as f32, scale,
                    );

                    renderer.render(&list, size.width, size.height).unwrap();
                    window.request_redraw();
                }
                winit::event::WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                }
                _ => {}
            },
            _ => {}
        }
    }).unwrap();
}
```

## Step 7: Handle Input Events

```rust
use lui::interactivity;
use lui::tree::MouseButton;

// In your WindowEvent match:
winit::event::WindowEvent::CursorMoved { position, .. } => {
    let scale = window.scale_factor() as f32;
    let pos = (position.x as f32, position.y as f32);
    if let Some(ref layout) = layout {
        interactivity::pointer_move(&mut tree, layout, pos);
    }
}
winit::event::WindowEvent::MouseInput { state, button, .. } => {
    if let Some(ref layout) = layout {
        let wb = match button {
            winit::event::MouseButton::Left => MouseButton::Primary,
            winit::event::MouseButton::Right => MouseButton::Secondary,
            winit::event::MouseButton::Middle => MouseButton::Middle,
            _ => MouseButton::Other(0),
        };
        match state {
            winit::event::ElementState::Pressed =>
                interactivity::mouse_down(&mut tree, layout, pos, wb),
            winit::event::ElementState::Released =>
                interactivity::mouse_up(&mut tree, layout, pos, wb),
        }
    }
}
winit::event::WindowEvent::MouseWheel { delta, .. } => {
    let dy = match delta {
        winit::event::MouseScrollDelta::LineDelta(_, y) => y * 20.0,
        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
    };
    tree.dispatch_scroll(&layout, pos, dy, &mut tree.interaction.scroll_offsets_y);
}
```

## Step 8: Call Paint/Update Each Frame

The integration loop is:

```
1. Forward input events → tree dispatch
2. paint_tree_returning_layout() → (DisplayList, LayoutBox)
3. renderer.render() → GPU frame
4. Keep LayoutBox for next frame's hit-testing
```

For better performance, use `paint_tree_cached()` which skips cascade + layout when nothing changed.
