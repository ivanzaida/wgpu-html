---
sidebar_position: 6
---

# Devtools Overlay

Demonstrates the built-in devtools panel for inspecting component trees and CSS styles at runtime.

## Enabling Devtools

```rust
use wgpu_html_devtools::Devtools;

// Attach devtools to a tree — this registers an F11 handler
let mut devtools = Devtools::attach(&mut tree, false); // false = profiler off
```

## Second Window (Recommended)

The devtools render in their own window alongside your application:

```rust
let devtools_win = Arc::new(event_loop.create_window(
    Window::default_attributes()
        .with_title("wgpu-html Devtools")
)?);
let mut devtools_driver = WinitDriver::bind(devtools_win, Tree::default());

// In event loop, maintain devtools tree:
devtools.poll_with_layout(&tree, runtime.layout());
devtools_driver.dispatch_to(&event, devtools.tree_mut());
devtools_driver.render(devtools.tree_mut());
```

## Features

- **Component tree browser** — expandable tree view of all DOM nodes
- **Styles inspector** — computed CSS properties for the selected element
- **Breadcrumb bar** — ancestor path from root to selected element
- **Pick mode** — click an element in the main window to inspect it
- **Hover highlight** — outlines the hovered element in the main window

## Devtools API

```rust
devtools.toggle();           // Toggle devtools on/off
devtools.enable();           // Enable devtools
devtools.disable();          // Disable devtools

devtools.is_pick_mode();     // Check if pick mode is active
devtools.hovered_path();     // Get currently hovered element path (for painting highlight)

if let Some(path) = devtools.hovered_path() {
    // Paint a highlight rectangle around this element
}
```

## Key Bindings

| Key | Action |
|---|---|
| F11 | Toggle devtools window |
| Click in devtools | Select element for inspection |
| Pick mode + click | Select element in main window |
