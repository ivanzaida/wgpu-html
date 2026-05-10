---
title: Quick Start
---

# Quick Start

This guide walks you through getting "Hello World" rendered on screen with lui in a winit window.

## Minimal Example — Parse, Cascade, Layout, Paint

The core pipeline is four steps:

```rust
use lui::parse;
use lui_style::cascade;
use lui_layout::layout_with_text;

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

// 1. Parse HTML into a tree
let mut tree = parse(html);

// 2. Register fonts (required before cascading/layout)
tree.register_font(
    lui_tree::FontFace {
        family: "sans-serif".into(),
        file: "C:/Windows/Fonts/segoeui.ttf".into(),
        ..Default::default()
    }
);

// 3. Cascade styles
let cascaded = cascade(&tree);

// 4. Layout (needs a viewport width)
let layout = layout_with_text(&cascaded, 800.0);

// 5. Paint to display list
let display_list = lui::paint::paint_tree_returning_layout(
    &layout,
    &cascaded,
    800.0,
    600.0,
);
```

At this point you have a `DisplayList` — the backend-agnostic draw-command list — but no window to show it in.

## Full winit Window Example

For a real application, use the `lui-winit` harness which handles window creation, event loops, rendering, scrolling, and clipboard:

```rust
use lui::parse;
use lui_tree::{Tree, FontFace};
use lui_winit::{create_window, AppHook, HookContext, EventResponse};

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

    // Register system fonts
    lui_winit::register_system_fonts(&mut tree, "sans-serif");

    // Launch window
    create_window(&mut tree)
        .with_title("Hello lui")
        .with_size(800, 600)
        .run();
}
```

## Expected Outcome

You should see a dark window (800×600) with centered white text reading "Hello, lui!" and a subtitle in grey. The window handles:

- **Resizing** — layout re-runs on window resize
- **Scrolling** — mouse wheel scrolls the viewport
- **`Esc` to exit** — built into the harness
- **`F12` screenshot** — saves `screenshot-<unix>.png` to disk
- **`Ctrl+A` / `Ctrl+C`** — text selection and clipboard copy

:::tip Using non-system fonts
If you don't have access to system fonts, register a `.ttf` file manually:

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "path/to/your-font.ttf".into(),
    ..Default::default()
});
```

Multiple fonts can be registered for the same family with different `weight` and `style` values.
:::

:::note Viewport size
The layout viewport width is the window's physical width (accounting for DPI scale factor). The harness handles this automatically — you don't need to pass a fixed size.
:::

## Next Steps

- Read about the [Component Framework](../component-framework/) for building interactive UIs
- Explore [CSS properties](../css/property-index) to see what's supported
- See [Rust Integration](../rust-integration/overview) for advanced embedding patterns
