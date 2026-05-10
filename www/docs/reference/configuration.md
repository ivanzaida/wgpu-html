---
sidebar_position: 2
---

# Configuration

## Tree Configuration

```rust
let mut tree = parse(html);

// Control <body> wrapping for multi-root documents
tree.with_body(true); // wrap multiple root nodes in <body>

// Disable the built-in UA stylesheet
tree.with_ua_stylesheet(false); // true by default

// DPI override for testing (None = use host window DPI)
tree.set_dpi_scale_override(Some(1.5));

// Asset cache TTL for images
tree.asset_cache_ttl = Some(Duration::from_secs(300));

// Asset root for resolving relative URLs
tree.set_asset_root(PathBuf::from("./assets"));

// Selection colors
tree.set_selection_colors(
    [0.2, 0.4, 0.8, 0.4],  // background RGBA
    [1.0, 1.0, 1.0, 1.0],  // foreground RGBA
);

// Locale for date formatting
tree.locale = Arc::new(MyLocale::new());
```

## Renderer Configuration

The renderer auto-configures from the window. For headless mode:

```rust
let renderer = Renderer::headless();
```

## Surface Configuration

Surface format is platform-dependent:
- **Windows**: `Bgra8UnormSrgb`
- **Other**: `Rgba8UnormSrgb`

Present mode: `Fifo` (vsync) with `AutoVsync` fallback.

## Profiler Configuration

```rust
let mut devtools = Devtools::new(true); // Enable profiler
let mut devtools = Devtools::attach(&mut tree, true); // With profiler
```

The profiler maintains a 240-frame ring buffer (~4s at 60fps) of cascade, layout, and paint timing data.

## Limiting CSS Resolution

Set resource limits via tree options:

```rust
// Maximum decoded image dimensions
// (default: 8192x8192, images exceeding this are scaled down)

// Maximum number of gradient color stops
// (no explicit limit, but memory grows with stop count)
```

## Feature Flags

No compile-time feature flags are currently used. All functionality is always available. Driver selection is done by depending on the desired driver crate:

- `wgpu-html-driver-winit` for winit windows
- `wgpu-html-driver-egui` for egui integration
- `wgpu-html-driver-bevy` for Bevy integration
- Custom `Driver` trait implementation for anything else
