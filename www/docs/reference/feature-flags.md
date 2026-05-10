---
sidebar_position: 3
---

# Feature Flags

wgpu-html has no compile-time feature flags. All core functionality is always enabled. The feature set is selected by which driver crate you depend on.

## Driver Selection

| Crate | Use Case |
|---|---|
| `wgpu-html-driver-winit` | Desktop applications with winit windows |
| `wgpu-html-driver-egui` | Embedding inside egui/eframe |
| `wgpu-html-driver-bevy` | Bevy game engine integration |

## Optional Crates

| Crate | Purpose |
|---|---|
| `wgpu-html-ui` | Component framework (Elm architecture, reactive state) |
| `wgpu-html-devtools` | Visual devtools panel |

## No Cargo Features

The workspace intentionally avoids Cargo features to keep the dependency tree simple. Instead of feature-gating optional functionality, separate crates are used:

```
# Core (always needed)
wgpu-html = "0.1"

# Window management (pick one)
wgpu-html-driver-winit = "0.1"
# OR wgpu-html-driver-egui = "0.1"
# OR wgpu-html-driver-bevy = "0.1"

# Optional extras
wgpu-html-ui = "0.1"         # if using component framework
wgpu-html-devtools = "0.1"   # if embedding devtools
```

This approach means you only compile what you actually use, without conditional compilation complexity.
