# wgpu-html

GPU-accelerated HTML/CSS renderer for Rust, built on [wgpu](https://github.com/gfx-rs/wgpu).

## ⚠️ Very early work in progress — not usable yet

This project is under heavy development. Nothing is stable. APIs change daily. Many CSS features are missing or broken. Do not use for anything serious.

## What it tries to do

Parse real HTML5, apply CSS (Flexbox + Grid), lay out text with shaping, and paint everything through custom GPU pipelines — all in Rust, zero JavaScript.

| Stage | Crate |
|---|---|
| HTML/CSS parsing | `wgpu-html-parser` |
| Style cascade | `wgpu-html-style` |
| Layout | `wgpu-html-layout` |
| Paint | `wgpu-html` |
| GPU render | `wgpu-html-renderer` |

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
