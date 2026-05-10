---
sidebar_position: 3
---

# Feature Flags

lui has no compile-time feature flags. All core functionality is always enabled. The feature set is selected by which driver crate you depend on.

## Driver Selection

| Crate | Use Case |
|---|---|
| `lui-driver-winit` | Desktop applications with winit windows |
| `lui-driver-egui` | Embedding inside egui/eframe |
| `lui-driver-bevy` | Bevy game engine integration |

## Optional Crates

| Crate | Purpose |
|---|---|
| `lui-ui` | Component framework (Elm architecture, reactive state) |
| `lui-devtools` | Visual devtools panel |

## No Cargo Features

The workspace intentionally avoids Cargo features to keep the dependency tree simple. Instead of feature-gating optional functionality, separate crates are used:

```
# Core (always needed)
lui = "0.1"

# Window management (pick one)
lui-driver-winit = "0.1"
# OR lui-driver-egui = "0.1"
# OR lui-driver-bevy = "0.1"

# Optional extras
lui-ui = "0.1"         # if using component framework
lui-devtools = "0.1"   # if embedding devtools
```

This approach means you only compile what you actually use, without conditional compilation complexity.
