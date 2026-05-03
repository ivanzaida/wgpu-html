---
id: installation
title: Installation
---

# Installation

## Add to Cargo.toml

Add `wgpu-html` to your project's `Cargo.toml`:

```toml
[dependencies]
wgpu-html = "0.1"
```

If you want the winit integration harness (recommended for desktop apps):

```toml
[dependencies]
wgpu-html = "0.1"
wgpu-html-winit = "0.1"
```

For embedding inside an existing egui/eframe application:

```toml
[dependencies]
wgpu-html = "0.1"
wgpu-html-egui = "0.1"
```

## System Requirements

- **GPU with Vulkan, Metal, or DX12 support** — wgpu uses `wgpu::Backends::PRIMARY`, which selects the best available backend for the platform.
- **Working graphics drivers** — if you can run `wgpu` examples, you can run wgpu-html.
- **Fonts** — wgpu-html does not ship fonts. You must register `.ttf`, `.otf`, or `.ttc` font files at startup. The winit harness includes `system_font_variants()` and `register_system_fonts()` helpers for Windows, Linux, and macOS.

## Crate Dependency Tree

The crates form a layered dependency graph. You typically only depend on the top-level façades:

```
wgpu-html (façade)
├── wgpu-html-parser
├── wgpu-html-style
│   └── wgpu-html-models
├── wgpu-html-layout
│   ├── wgpu-html-text (cosmic-text)
│   └── wgpu-html-events
├── wgpu-html-renderer (wgpu)
└── wgpu-html-tree
```

Integration crates layer on top:

```
wgpu-html-winit / wgpu-html-egui
└── wgpu-html
```

The component framework is optional:

```
wgpu-html-ui
└── wgpu-html
```

## Optional Features

| Feature | Crate | Description |
|---|---|---|
| Winit harness | `wgpu-html-winit` | Batteries-included window and event loop via winit. Includes viewport scrolling, scrollbar paint/drag, clipboard, and F12 screenshots. |
| egui backend | `wgpu-html-egui` | Embed wgpu-html inside an egui/eframe application window. |
| Component framework | `wgpu-html-ui` | Elm-architecture component model with reactive state, scoped CSS, and render caching. |
| Devtools | `wgpu-html-devtools` | Visual devtools panel for inspecting component trees and CSS styles at runtime. |

## Next Steps

Head to the [Quick Start guide](./quick-start) to get your first window on screen.
