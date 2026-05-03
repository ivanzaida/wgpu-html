# wgpu-html

GPU-accelerated HTML/CSS renderer for Rust, built on [wgpu](https://github.com/gfx-rs/wgpu).

## Status

Active development. Core pipeline is functional: full HTML5 parsing, CSS cascade with `@media`, Flexbox + Grid layout, inline text with shaping, GPU rendering (quads, glyphs, images), mouse+keyboard interactivity, text editing, text selection, scrolling, and a component framework (`wgpu-html-ui`). Some CSS features are missing (see [full status](./docs/full-status.md)).

## What it tries to do

Parse real HTML5, apply CSS (Flexbox + Grid), lay out text with shaping, and paint everything through custom GPU pipelines — all in Rust, zero JavaScript.

| Stage | Crate |
|---|---|
| HTML/CSS parsing | `wgpu-html-parser` |
| Style cascade | `wgpu-html-style` |
| Layout | `wgpu-html-layout` |
| Paint | `wgpu-html` |
| GPU render | `wgpu-html-renderer` |
| Text shaping | `wgpu-html-text` |
| DOM tree + events | `wgpu-html-tree` |
| Component framework | `wgpu-html-ui` |
| Winit window harness | `wgpu-html-winit` |
| Devtools | `wgpu-html-devtools` |

## Getting started

```rust
// Parse HTML
let mut tree = wgpu_html_parser::parse(r#"<h1>hello <span style="color: red">world</span></h1>"#);

// Register fonts
tree.register_font(FontFace::from_file("Roboto-Regular.ttf", 0).unwrap());

// Full pipeline: cascade → layout → paint
let (display_list, layout) = wgpu_html::paint_tree_returning_layout(
    &mut tree, &mut text_ctx, &image_cache, 800, 600, 1.0,
);
```

## Documentation

- [Docs site](https://ivanzaida.github.io/wgpu-html/) (also WIP)
- [Full status](./docs/full-status.md)
- [Roadmap](./docs/roadmap.md)
- [vs RmlUI comparison](./docs/wgpu-html-vs-rmlui.md)

## License

MIT
