# CLAUDE.md

## Project structure

V2 engine pipeline: `lui-parse` → `lui-cascade` → `lui-layout` → `lui-paint` → `lui-display-list` → `lui-renderer-wgpu`

| Crate | Path | Role |
|-------|------|------|
| lui-core | crates/lui-core | Shared types (CssValue, HtmlNode, Rect, etc.) |
| lui-parse | crates/lui-parse | HTML + CSS parser |
| lui-cascade | crates/lui-cascade | CSS cascade + computed styles |
| lui-layout | crates/lui-layout | Layout engine (block, flex, grid, table, inline, positioned) |
| lui-paint | crates/lui-paint | Layout tree → DisplayList (quads, glyphs, clips) |
| lui-glyph | crates/lui-glyph | Text shaping + atlas via cosmic-text |
| lui-display-list | crates-v1/lui-display-list | Backend-agnostic display list IR |
| lui-render-api | renderers/lui-render-api | RenderBackend trait |
| lui-renderer-wgpu | renderers/lui-renderer-wgpu | wgpu GPU renderer |
| lui-driver | drivers/lui-driver | V2 Runtime (owns cascade + layout + paint pipeline) |
| lui-driver-winit | drivers/lui-driver-winit | Winit window integration |
| lui-demo | demo/lui-demo | V2 demo app |

V1 engine crates live under `crates-v1/`, `drivers-v1/`, `demo-v1/`.

## Taking screenshots

The demo app supports headless screenshot capture for visual testing and debugging.

### From CLI (headless, no window interaction needed)

```bash
# Render default test.html
cargo run -p lui-demo -- --screenshot screenshot.png

# Render custom HTML from a file
cargo run -p lui-demo -- --html path/to/page.html --screenshot out.png

# Pipe HTML from stdin
echo '<div style="background:red; width:200px; height:100px"></div>' | cargo run -p lui-demo -- --screenshot out.png
```

### From interactive window

Press **F12** while the window is open to save `screenshot.png` in the working directory.

### Programmatically (in Rust)

```rust
// Via WinitDriver
driver.screenshot_to("output.png")?;
let pixels: Vec<u8> = driver.render_to_rgba()?;

// Via Runtime directly
rt.screenshot_to(&doc, 800, 600, "output.png")?;
let pixels = rt.render_to_rgba(&doc, 800, 600)?;
```

### Expected output for visual tests

Run `node .tests/screenshot.mjs` to generate expected reference screenshots. Compare against the output from the demo app to check for regressions.

## Running tests

```bash
# Layout engine tests (318 tests)
cargo test --package lui-layout --test tests

# Layout benchmarks
cargo bench --package lui-layout --bench layout_bench

# All workspace tests
cargo test --workspace
```

## Key conventions

- Layout positions (`LayoutBox.content.x/y`) are absolute from viewport origin
- Display list coordinates are in physical pixels
- `TextContext` is caller-owned, passed as `&mut` (never owned by layout/paint)
- `LayoutEngine` owns only the incremental cache, not the font system
- Arena allocation: `LayoutBox.children` uses `bumpalo::collections::Vec`
