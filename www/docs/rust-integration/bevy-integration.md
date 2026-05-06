---
title: Bevy Integration
---

# Bevy Integration

The `wgpu-html-driver-bevy` crate provides `WgpuHtmlPlugin` — a single fullscreen transparent HTML overlay for Bevy applications.

## Architecture

wgpu-html runs its own headless wgpu device (independent of Bevy's render pipeline) and produces RGBA pixels via offscreen rendering. Those pixels are uploaded into a Bevy `Image` asset each frame. This decouples wgpu versions and avoids render-graph complexity.

- One DOM `Tree` covering the entire viewport
- One GPU render pass per frame
- Transparent clear color — only painted elements are visible
- CSS handles all layout and positioning within the overlay

## Quick Start

```toml
[dependencies]
wgpu-html-driver-bevy = { path = "drivers/wgpu-html-driver-bevy" }
bevy = "0.15"
```

```rust
use bevy::prelude::*;
use wgpu_html_driver_bevy::{WgpuHtmlPlugin, HtmlOverlay};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WgpuHtmlPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut html: NonSendMut<HtmlOverlay>) {
    let parsed = wgpu_html::parser::parse(r#"
        <style>
            .hud { padding: 20px; color: white; font-family: sans-serif; }
        </style>
        <div class="hud">
            <h1>Hello from wgpu-html!</h1>
        </div>
    "#);
    html.tree_mut().merge(parsed);
}
```

## HtmlOverlay API

`HtmlOverlay` is inserted as a non-send resource by the plugin. Access it in systems via `NonSendMut<HtmlOverlay>`.

### DOM Manipulation

```rust
// Append a node to the root
html.append(node);

// Append a node at a specific path
html.append_to(&[0, 2], node);

// Remove an element by ID
html.remove_by_id("my-panel");

// Clear all children
html.clear();

// Direct tree access for anything else
html.tree_mut().get_element_by_id("score").unwrap()
    .children[0].element = Element::Text("Score: 42".into());
```

### Merging Parsed HTML

Use `Tree::merge` to append parsed HTML fragments into the overlay:

```rust
let fragment = wgpu_html::parser::parse(r#"
    <div id="dialog" class="modal">...</div>
"#);
html.tree_mut().merge(fragment);
```

### Input Control

```rust
// Disable input capture (e.g. while player is moving)
html.set_captures_input(false);

// Re-enable (e.g. when opening a menu)
html.set_captures_input(true);
```

## CSS Styling

Use `<style>` tags in your HTML — the cascade discovers `StyleElement` nodes anywhere in the tree:

```rust
let ui = wgpu_html::parser::parse(r#"
    <style>
        .btn {
            padding: 8px 16px;
            border-radius: 6px;
            cursor: pointer;
            transition: background 0.15s;
        }
        .btn:hover { background: rgba(255,255,255,0.1); }
        .btn:active { transform: scale(0.96); }
    </style>
    <div class="btn">Click me</div>
"#);
html.tree_mut().merge(ui);
```

Full CSS cascade, selectors (classes, IDs, pseudo-classes, combinators), and properties are supported. See the [CSS docs](/docs/css/overview) for the complete property index.

## Event Handling

Attach callbacks directly on nodes:

```rust
use std::sync::Arc;

let mut parsed = wgpu_html::parser::parse(r#"<button id="play">Play</button>"#);
if let Some(btn) = parsed.root.as_mut().unwrap().find_by_id_mut("play") {
    btn.on_click.push(Arc::new(|_ev| {
        println!("Play clicked!");
    }));
}
html.tree_mut().merge(parsed);
```

## Devtools

The demo crate shows how to attach devtools as a secondary Bevy window, toggled with F11:

```rust
// In setup (exclusive system):
let devtools = Devtools::attach(&mut overlay.tree, false);

// Each frame:
devtools.poll(&overlay.tree);
if devtools.is_enabled() {
    // spawn/render devtools window
}
```

See `demo/wgpu-html-demo-bevy/src/devtools.rs` for the full implementation.

## DPI Scaling

The overlay automatically reads the window's `scale_factor()` and renders at physical pixel resolution. CSS pixel values map correctly at any DPI — a `400px` panel stays 400 CSS pixels regardless of display scaling.

## How It Works

Each frame the plugin:

1. Reads the window's physical dimensions and DPI scale
2. Runs the full pipeline: cascade → layout → paint → `DisplayList`
3. Uploads glyph atlas rasters
4. Calls `Renderer::render_to_rgba()` — renders the `DisplayList` into an offscreen texture and reads back RGBA pixels
5. Writes the pixels into the Bevy `Image` asset backing the fullscreen `ImageNode`

The overlay entity is spawned at `GlobalZIndex(i32::MAX)` with `position: absolute` covering 100% of the viewport.
