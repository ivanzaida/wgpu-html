---
sidebar_position: 10
---

# Resource Loading

lui handles fonts, images, and linked stylesheets through a centralized asset system in `lui-assets`.

## Image Loading

Image loading lives in `lui-layout-old` with caching in `lui-assets`.

### Supported Sources

| Scheme | Supported |
|---|---|
| `http(s)://` | ✅ (ureq + rustls) |
| `data:` URIs | ✅ (base64 + percent-encoded) |
| Local file paths | ✅ |
| `.png` | ✅ |
| `.jpg`/`.jpeg` | ✅ |
| `.gif` (animated) | ✅ |
| `.bmp` | ✅ |
| `.webp` (animated) | ✅ |

### Caching

Two-level process-wide cache:
- **Raw cache** — fetched bytes keyed by URL
- **Sized cache** — decoded RGBA8 buffers keyed by (URL, width, height)
- TTL-based eviction with byte budget
- `Cache-Control: max-age` respected per-URL

### Preloading

```rust
tree.preload_asset("https://example.com/image.png");
tree.preload_queue.push("https://example.com/bg.jpg".into());
```

The preload queue is consumed at layout time. Preloaded images are fetched and decoded before the first frame.

### Asset Root

```rust
tree.set_asset_root(PathBuf::from("./assets"));
```

Relative URLs (e.g., `src="img/logo.png"`) are resolved relative to the asset root.

## Gradient Rendering

CSS gradients are rasterized to RGBA pixel buffers during layout:

- **`linear-gradient()`** — with angle and color stop support
- **`radial-gradient()`** — circle/ellipse with position and color stops
- **`conic-gradient()`** — with angle offset and color stops
- **`repeating-*` variants** — all three gradient types support repeating

Gradients are rendered as image quads in the image pipeline, meaning they share the same clipping and opacity handling as `<img>` elements.

## Font Loading

Fonts must be registered before cascade and layout:

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Regular.ttf".into(),
    weight: 400,
    style: FontStyle::Normal,
});

// Register multiple weights/styles for the same family
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Bold.ttf".into(),
    weight: 700,
    ..Default::default()
});
```

## Linked Stylesheets

```rust
tree.register_linked_stylesheet(
    "/styles/main.css".into(),
    std::fs::read_to_string("styles/main.css").unwrap().into(),
);
```

Linked stylesheets appear as `link` elements in the tree and are collected during cascade. They participate in the normal rule matching and cascade process.

## CSS Import Resolution

```rust
pub fn resolve_css_imports(tree: &mut Tree, assets: &mut ImageCache) -> bool;
```

Resolves any `@import` rules in stylesheets, fetching and inlining the imported CSS.

## Animated Images

GIF and WebP animations are supported. Frame selection uses a process-wide clock anchor for synchronized playback. The image cache stores decoded frames, and the renderer displays the current frame per tick.
