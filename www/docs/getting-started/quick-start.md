---
sidebar_position: 3
---

# Quick Start

This guide walks you through getting "Hello World" rendered on screen with lui in a winit window.

## Minimal Example — Parse, Cascade, Layout, Paint

```rust
use lui::parse;
use lui_style::cascade;
use lui_layout_old::layout_with_text;

let html = r#"
    <!DOCTYPE html>
    <html>
        <body>
            <div style="color: white; font-size: 32px; text-align: center;">
                Hello, world!
            </div>
        </body>
    </html>
"#;

let mut tree = parse(html);

tree.register_font(
    lui_tree::FontFace {
        family: "sans-serif".into(),
        file: "C:/Windows/Fonts/segoeui.ttf".into(),
        ..Default::default()
    }
);

// Cascade → Layout → Paint
let cascaded = cascade(&tree);
let layout = layout_with_text(&cascaded, 800.0);
let display_list = lui::paint::paint_tree_returning_layout(
    &layout, &cascaded, 800.0, 600.0,
);
```

## Full winit Window Example

```rust
use lui::parse;
use lui_winit::{create_window, AppHook};

fn main() {
    let html = r#"<!DOCTYPE html>
<html>
<body style="margin: 0; background: #1a1a2e; color: #e0e0e0;
             display: flex; justify-content: center; align-items: center; height: 100vh;">
    <div style="text-align: center;">
        <h1 style="font-size: 48px; margin: 0;">Hello, lui!</h1>
        <p style="font-size: 18px; color: #888;">GPU-accelerated HTML/CSS in Rust</p>
    </div>
</body>
</html>"#;

    let mut tree = parse(html);
    lui_winit::register_system_fonts(&mut tree, "sans-serif");

    create_window(&mut tree)
        .with_title("Hello lui")
        .with_size(800, 600)
        .run();
}
```

## Expected Outcome

A dark window (800×600) with centered white text reading "Hello, lui!" and a grey subtitle. The window handles:

- **Resizing** — layout re-runs on window resize
- **Scrolling** — mouse wheel scrolls the viewport
- **`Esc` to exit** — built into the harness
- **`F12` screenshot** — saves `screenshot-<unix>.png` to disk
- **`Ctrl+A` / `Ctrl+C`** — text selection and clipboard copy

:::tip Using non-system fonts

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "path/to/your-font.ttf".into(),
    ..Default::default()
});
```
:::

:::note Viewport size
The layout viewport width is the window's physical width accounting for DPI scale factor. The harness handles this automatically.
:::

## Next Steps

- Explore [Supported CSS](../features/supported-css) to see what's available
- Read about [Embedding](../integration/embedding) for advanced integration patterns
