---
sidebar_position: 2
---

# Installation

## Add to Cargo.toml

```toml
[dependencies]
lui = "0.1"
```

With the winit integration harness (recommended for desktop apps):

```toml
[dependencies]
lui = "0.1"
lui-winit = "0.1"
```

For embedding inside an existing egui/eframe application:

```toml
[dependencies]
lui = "0.1"
lui-egui = "0.1"
```

## System Requirements

- **GPU with Vulkan, Metal, or DX12 support** — the default wgpu backend uses `wgpu::Backends::PRIMARY`, which selects the best available backend for the platform. The render backend is pluggable; future native backends (D3D12, Vulkan, OpenGL) are planned.
- **Working graphics drivers** — if you can run `wgpu` examples, you can run lui.
- **Fonts** — lui does not ship fonts. You must register `.ttf`, `.otf`, or `.ttc` font files at startup. The winit harness includes `register_system_fonts()` helpers.

## Crate Dependency Tree

```
lui (façade)
├── lui-parser
├── lui-style
│   └── lui-models
├── lui-layout
│   ├── lui-text (cosmic-text)
│   └── lui-events
├── lui-display-list          # backend-agnostic IR
├── lui-renderer-wgpu (wgpu)  # pluggable via RenderBackend trait
└── lui-tree
```

Integration crates layer on top:

```
lui-winit / lui-egui
└── lui
```

## Optional Features

| Feature | Crate | Description |
|---|---|---|
| Winit harness | `lui-winit` | Batteries-included window and event loop via winit. Viewport scrolling, clipboard, F12 screenshots. |
| egui backend | `lui-egui` | Embed lui inside an egui/eframe application window. |
| Component framework | `lui-ui` | Elm-architecture component model with reactive state, scoped CSS, and render caching. |
| Devtools | `lui-devtools` | Visual devtools panel for inspecting component trees and CSS styles at runtime. |

## Next Steps

Head to the [Quick Start guide](./quick-start) to get your first window on screen.
