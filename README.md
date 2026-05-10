# lui

GPU-accelerated HTML/CSS renderer for Rust, built on [wgpu](https://github.com/gfx-rs/wgpu).

## Status

Active development. Core pipeline is functional: full HTML5 parsing, CSS cascade with `@media`, Flexbox + Grid layout, inline text with shaping, GPU rendering (quads, glyphs, images, gradients), mouse+keyboard interactivity, text editing, text selection, scrolling, and a component framework (`lui-ui`). Some CSS features are missing (see [Supported CSS](https://ivanzaida.github.io/lui/docs/features/supported-css)).

## What it tries to do

Parse real HTML5, apply CSS (Flexbox + Grid), lay out text with shaping, and paint everything through custom GPU pipelines — all in Rust, zero JavaScript.

| Stage | Crate |
|---|---|
| HTML/CSS parsing | `lui-parser` |
| Style cascade | `lui-style` |
| Layout | `lui-layout` |
| Paint | `lui` |
| GPU render | `lui-renderer` |
| Text shaping | `lui-text` |
| DOM tree + events | `lui-tree` |
| Component framework | `lui-ui` |
| Winit window harness | `lui-winit` |
| Devtools | `lui-devtools` |

## Getting started

```rust
// Parse HTML
let mut tree = lui_parser::parse(r#"<h1>hello <span style="color: red">world</span></h1>"#);

// Register fonts
tree.register_font(FontFace::from_file("Roboto-Regular.ttf", 0).unwrap());

// Full pipeline: cascade → layout → paint
let (display_list, layout) = lui::paint_tree_returning_layout(
    &mut tree, &mut text_ctx, &image_cache, 800, 600, 1.0,
);
```

## Documentation

- [Docs site](https://ivanzaida.github.io/lui/)
- [Implementation status](https://ivanzaida.github.io/lui/docs/status)
- [CSS roadmap](https://ivanzaida.github.io/lui/docs/css/css-roadmap)
- [vs RmlUI comparison](https://ivanzaida.github.io/lui/docs/comparison-lui-vs-rmlui)

## License

MIT
