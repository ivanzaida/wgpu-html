---
sidebar_position: 4
---

# Running Examples

The workspace includes a demo application and several example HTML pages.

## Running the Demo

```bash
cargo run -p lui-demo
```

This launches a winit window with the default demo page. The demos live under `demo/lui-demo/html/`.

## Running a Specific HTML File

```bash
cargo run -p lui-demo -- demo/lui-demo/html/styled-inputs.html
```

## Demo Controls

| Key | Action |
|---|---|
| `Esc` | Exit the demo |
| `F12` | Save a PNG screenshot |
| `Ctrl+A` | Select all text |
| `Ctrl+C` | Copy selection to clipboard |
| `F9` | Print profiler summary to stderr |
| Mouse wheel | Scroll viewport |

## Changing the Default Page

The default demo page is set in `crates/lui-demo/src/main.rs` via `const DEFAULT_DOC`. To change it permanently, edit that include:

```rust
const DEFAULT_DOC: &str = include_str!("../html/your-page.html");
```

## Available Demo Pages

| File | Description |
|---|---|
| `flex-browser-like.html` | Flex-based layout with navigation, content, and sidebar |
| `grid.html` | Holy grail layout and photo gallery using CSS Grid |
| `styled-inputs.html` | Form controls with various input types |
| `overflow.html` | Overflow clipping demos (visible, hidden, rounded) |
| `img-test.html` | Image loading and display |
| `gif.html` | Animated GIF rendering |
| `table.html` | Table element parsing and styling |

## Screenshots

Press `F12` to save `screenshot-<unix>.png` in the current working directory. For programmatic screenshots, see the [Public API](../reference/public-api) reference.

## Profiling

Press `F9` to dump a profiler summary to stderr showing cascade, layout, and paint timing per frame.
