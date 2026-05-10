---
sidebar_position: 7
---

# Images

## `<img>` Element

Images are loaded, decoded, and rendered through the image pipeline on the GPU. Supported attributes: `src`, `width`, `height`, `alt`, `loading`, `decoding`, `srcset`, `sizes`.

```html
<img src="photo.jpg" width="400" height="300" alt="A photo">
<img src="https://example.com/image.png">
<img src="data:image/png;base64,iVBORw0KGgo...">
```

## Supported Formats

| Format | Static | Animated |
|---|---|---|
| PNG | ✅ | — |
| JPEG | ✅ | — |
| BMP | ✅ | — |
| GIF | ✅ | ✅ |
| WebP | ✅ | ✅ |

## Background Images

CSS `background-image` supports both URL images and CSS gradients:

```css
.banner {
    background-image: url("bg.jpg");
    background-size: cover;
    background-position: center;
    background-repeat: no-repeat;
}
```

Multiple background layers: only the first is consumed.

## Image Loading and Caching

Images are loaded asynchronously with a worker thread pool:

- **Non-blocking** — loading doesn't block the main render thread
- **Two-level cache** — raw bytes + decoded RGBA8 buffers
- **TTL eviction** — configurable via `tree.asset_cache_ttl`
- **`Cache-Control: max-age`** — respected per-URL
- **Retry with exponential backoff** for HTTP failures

## Preloading

```rust
tree.preload_asset("https://example.com/bg.jpg");
```

Preloaded images are fetched before the first frame, avoiding pop-in.

## Asset Root

```rust
tree.set_asset_root(PathBuf::from("./assets"));
```

Relative URLs (e.g., `src="img/photo.jpg"`) are resolved relative to the asset root.

## CSS Gradients

All standard gradient types are supported and rasterized to RGBA textures at layout time:

```css
.box {
    background-image: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    background-image: radial-gradient(circle at center, #fff 0%, #000 100%);
    background-image: conic-gradient(from 90deg, red, yellow, green, blue, red);
    background-image: repeating-linear-gradient(0deg, #ccc 0px, #ccc 10px, transparent 10px, transparent 20px);
}
```

## Size Constraints

- Maximum decoded image dimensions: 8192×8192
- Images exceeding this are scaled down during decode
- `width`/`height` HTML attributes take precedence over intrinsic dimensions

## Current Limitations

- No `object-fit` or `object-position`
- No `image-rendering` (sampling mode)
- No `image-orientation` (EXIF rotation)
- `<picture>` with `<source>` media selection is limited
- Image maps (`<map>`, `<area>`) are not supported
