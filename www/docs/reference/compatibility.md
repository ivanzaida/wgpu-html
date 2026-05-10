---
sidebar_position: 6
---

# Compatibility

## GPU Support

| Backend | Platform | Status |
|---|---|---|
| Vulkan | Windows, Linux | ✅ Supported |
| DirectX 12 | Windows | ✅ Supported |
| Metal | macOS | ✅ Supported |
| WebGPU | Web (wasm) | Planned |
| OpenGL | All | Not supported |

## Crate Version Pinnings

Workspace crates use pinned version ranges. Key dependencies:

| Dependency | Role |
|---|---|
| `wgpu ~0.22` | GPU API abstraction |
| `winit ~0.30` | Window creation and events |
| `cosmic-text ~0.12` | Text shaping (HarfBuzz) |
| `ureq ~3.0` | HTTP image loading |
| `image ~0.25` | Image decoding (PNG, JPEG, GIF, BMP, WebP) |
| `arboard ~3.4` | Clipboard access |
| `rfd ~0.15` | Native file dialogs |
| `egui ~0.31` | egui integration (driver) |
| `bevy ~0.15` | Bevy integration (driver) |

## Rust Version

Requires Rust 1.78 or later (for `impl Trait` in associated types and other recent stabilizations).

## Platform-Specific Notes

### Windows
- DX12 is the default backend
- System fonts in `C:\Windows\Fonts\`
- BGRA8 surface format

### Linux
- Vulkan is the default backend
- System fonts in `/usr/share/fonts/` and `~/.local/share/fonts/`
- May need `libx11-dev`, `libwayland-dev`, `libudev-dev` for winit

### macOS
- Metal is the default backend
- System fonts in `/System/Library/Fonts/`, `/Library/Fonts/`, `~/Library/Fonts/`

## Browser Compatibility Goals

lui aims for CSS behavior parity with web browsers, not pixel-perfect rendering. The goal is to make CSS authored for browsers work correctly in lui, not to match every browser quirk.

| Feature | Goal |
|---|---|
| Box model | CSS3 parity |
| Flexbox | Level 1 complete |
| Grid | Level 1 with limitations |
| Selectors | Level 4 complete |
| Colors | sRGB with alpha, CSS Color 4 |
| Text | Basic styling, shaping via HarfBuzz |
| Cascade | CSS Cascade 3 with !important and CSS-wide keywords |

## Known Divergences

- `em`/`rem` default to 16px when no font-size is inherited (browsers use UA defaults)
- `sticky` positioning degrades to `relative`
- `z-index` sorts siblings but doesn't create independent stacking contexts
- No `float` layout
- No baseline alignment in flex/grid
- Dashed/dotted on elliptical rounded boxes falls back to straight segments at corners
