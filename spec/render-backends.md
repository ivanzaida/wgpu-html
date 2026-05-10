# Pluggable Render Backends

## Goal

Replace the monolithic `lui-renderer` with a trait-based architecture
supporting native graphics backends:

- **wgpu** (current, wraps D3D12/Vulkan/Metal/GL internally)
- **D3D12** (native, Windows)
- **D3D11** (native, Windows legacy)
- **Vulkan** (native, cross-platform)
- **OpenGL** (native, legacy/embedded)

## Current Architecture

```
DisplayList (IR)          -- defined in lui-renderer/src/paint.rs
       |                     (zero wgpu types, pure f32/u32/Arc<Vec<u8>>)
       v
Renderer struct           -- lui-renderer/src/lib.rs
  +-- QuadPipeline           quad_pipeline.rs    (quad.wgsl, 313 lines)
  +-- GlyphPipeline          glyph_pipeline.rs   (glyph.wgsl)
  +-- ImagePipeline          image_pipeline.rs    (image.wgsl)
  +-- Screenshot             screenshot.rs
       |
       v
Driver (Runtime)          -- drivers/lui-driver/src/lib.rs
  calls: atlas.upload(&renderer.queue, renderer.glyph_atlas_texture())
  calls: renderer.render(&display_list)
```

wgpu coupling points:
- `Renderer` struct fields (device, queue, surface, pipelines)
- `Atlas::upload()` in lui-text takes `&wgpu::Queue` + `&wgpu::Texture`
- Driver accesses `renderer.queue` and `renderer.glyph_atlas_texture()` directly

Everything above the renderer (paint, layout, cascade, tree, parser) is
already backend-agnostic.

## Target Architecture

```
lui-display-list    -- new crate: DisplayList, Rect, Color, Quad, etc.
       |
       v
lui-render-api      -- new crate: RenderBackend trait
       |
       +---> lui-renderer        (WgpuRenderer, existing code)
       +---> lui-renderer-dx12   (Dx12Renderer)
       +---> lui-renderer-dx11   (Dx11Renderer)
       +---> lui-renderer-vk     (VulkanRenderer)
       +---> lui-renderer-gl     (GlRenderer)
       |
       v
Driver (Runtime<B: RenderBackend>)
```

## Phases

### Phase 0: Preparation (1 day) ✓

- [x] Change `wgpu::Backends::DX12` to `Backends::PRIMARY`
- [ ] Audit all `pub use wgpu` re-exports and wgpu type leaks

### Phase 1: Extract DisplayList crate (1 day) ✓

- [x] Create `lui-display-list` crate
- [x] Move from `lui-renderer/src/paint.rs`:
  - `DisplayList`, `Quad`, `GlyphQuad`, `ImageQuad`, `ClipRange`
  - `DisplayCommand`, `DisplayCommandKind`
  - `Rect`, `Color`, `FrameOutcome`
- [x] Re-export from `lui-renderer` for backward compat
- [x] Update all imports across the workspace

### Phase 2: Define RenderBackend trait (2-3 days) ✓

- [x] Create `lui-render-api` crate
- [x] Define `RenderBackend` trait (resize, set_clear_color, upload_atlas_region,
      render, render_to_rgba, capture_to, capture_next_frame_to, glyph_atlas_size)
- [x] Atlas upload folded into `RenderBackend` (no separate `AtlasUploader`)
- [x] Define `RenderError` enum

### Phase 3: Refactor wgpu renderer (2-3 days) ✓

- [x] `Renderer: RenderBackend` impl
- [x] Move atlas upload into renderer via `upload_atlas_region` trait method
- [x] Replace `atlas.upload(&queue, &texture)` in all drivers with
      `atlas.flush_dirty(|r, data| backend.upload_atlas_region(...))`
- [x] Convert `clear_color` assignments to `set_clear_color([f32; 4])`
- [ ] Remove `pub use wgpu` re-export (deferred: still needed for Driver trait's rwh bounds)
- [ ] Remove `glyph_atlas_texture()` from public API (deferred: no callers remain, can remove anytime)
- [x] All existing tests/drivers keep working

### Phase 4: Genericize Driver (2-3 days)

- [ ] `Runtime<B: RenderBackend>` instead of `Runtime` with concrete `Renderer`
- [ ] Update winit driver
- [ ] Update bevy driver
- [ ] Update egui driver
- [ ] Ensure `Driver` trait bounds don't leak backend-specific window handle types

### Phase 5: Native backends

Each backend requires:
1. Device/surface initialization
2. Buffer management (vertex, instance, uniform)
3. Shader port (see Shader Porting below)
4. Pipeline state (blend, scissor, draw calls)
5. Texture management (atlas + per-image cache)
6. Screenshot/readback

#### Phase 5a: D3D12 backend (2-3 weeks)

- [ ] Crate: `lui-renderer-dx12`
- [ ] Dependency: `windows` crate (`ID3D12Device`, `ID3D12GraphicsCommandList`, etc.)
- [ ] Port shaders: WGSL -> HLSL SM6
- [ ] Root signature = bind group layout equivalent
- [ ] PSO = render pipeline equivalent
- [ ] Descriptor heaps for textures
- [ ] Fence-based frame synchronization

#### Phase 5b: Vulkan backend (2-3 weeks)

- [ ] Crate: `lui-renderer-vk`
- [ ] Dependency: `ash` crate
- [ ] Port shaders: WGSL -> SPIR-V (use `naga` for automated translation)
- [ ] VkDescriptorSet = bind group
- [ ] VkPipeline = render pipeline
- [ ] VkFence / VkSemaphore synchronization
- [ ] Most boilerplate-heavy backend

#### Phase 5c: D3D11 backend (3-4 weeks)

- [ ] Crate: `lui-renderer-dx11`
- [ ] Dependency: `windows` crate (`ID3D11Device`, `ID3D11DeviceContext`)
- [ ] Port shaders: WGSL -> HLSL SM5
- [ ] No explicit descriptor sets — constant buffers via `VSSetConstantBuffers`
- [ ] State machine model (no pipeline objects)
- [ ] Simpler than D3D12/Vulkan but more different conceptually

#### Phase 5d: OpenGL backend (3-4 weeks)

- [ ] Crate: `lui-renderer-gl`
- [ ] Dependency: `glow` or `glutin` + raw GL
- [ ] Port shaders: WGSL -> GLSL 3.3+ (or 4.3 for compute)
- [ ] VAO + VBO model
- [ ] glUniform for globals
- [ ] State machine (glEnable, glBlendFunc, glScissor)
- [ ] Most different from modern APIs

### Phase 6: Visual regression testing (1 week)

- [ ] Screenshot comparison harness
- [ ] Golden images from wgpu backend
- [ ] Per-pixel diff tolerance
- [ ] CI integration for each backend
- [ ] Test matrix: styled-inputs, forms, text selection, scrolling, flex layout

## Shader Porting

The three WGSL shaders contain non-trivial rendering math:

### quad.wgsl (~313 lines)
- Rounded-corner SDF with independent per-corner elliptical radii
- Stroked rings (border rendering)
- Dash/dot pattern via arc-length parametrization
- Rounded clip region SDF discard
- sRGB-correct alpha blending

### glyph.wgsl
- R8 atlas sampling (coverage * color)
- Rounded clip SDF discard
- Linear-space output (non-sRGB view)

### image.wgsl
- RGBA texture sampling with opacity
- Rounded clip SDF discard

### Porting strategy

| Target | Source | Tool |
|--------|--------|------|
| HLSL SM6 (D3D12) | WGSL | naga or manual |
| HLSL SM5 (D3D11) | WGSL | naga or manual |
| SPIR-V (Vulkan) | WGSL | naga (`wgsl-in` -> `spv-out`) |
| GLSL 3.3+ (OpenGL) | WGSL | naga (`wgsl-in` -> `glsl-out`) |

**naga** (used by wgpu internally) can translate WGSL to SPIR-V, HLSL,
GLSL, and MSL. This could automate shader porting for all backends.
Risk: naga's HLSL/GLSL output may need manual fixups for edge cases
(especially the SDF math and sRGB handling).

## Estimation Summary

| Phase | Scope | Effort |
|-------|-------|--------|
| 0. Prep (Backends::all) | Trivial | 1 hour |
| 1. Extract DisplayList crate | Small | 1 day |
| 2. RenderBackend trait | Medium | 2-3 days |
| 3. Refactor wgpu impl | Medium | 2-3 days |
| 4. Genericize drivers | Medium | 2-3 days |
| 5a. D3D12 native | Large | 2-3 weeks |
| 5b. Vulkan native | Large | 2-3 weeks |
| 5c. D3D11 native | Large | 3-4 weeks |
| 5d. OpenGL native | Large | 3-4 weeks |
| 6. Visual regression tests | Medium | 1 week |
| **Total** | | **~14-19 weeks** |

Phases 0-4 (trait extraction) can ship independently in ~1.5 weeks.
Phase 5 backends are independent of each other and can be parallelized.

## Key Risks

1. **Shader fidelity** -- the quad SDF math is complex; per-pixel
   differences across backends will need tolerance thresholds
2. **naga output quality** -- automated WGSL->HLSL/GLSL may need
   manual fixups for sRGB, precision, or driver quirks
3. **Atlas upload timing** -- current pattern uploads mid-frame from
   the driver; backends may need different synchronization
4. **Window handle abstraction** -- each backend needs platform-specific
   surface creation (HWND, VkSurfaceKHR, GLContext)
5. **Testing coverage** -- without visual regression CI, subtle
   rendering differences will go unnoticed

## Decision Log

| Decision | Options | Chosen | Rationale |
|----------|---------|--------|-----------|
| Trait style | `dyn RenderBackend` vs generic `B: RenderBackend` | TBD | Generic avoids vtable overhead; dyn enables runtime selection |
| Shader porting | naga automated vs manual port | TBD | naga first, manual fixups where needed |
| Atlas ownership | Backend owns texture vs callback | TBD | Callback (`flush_dirty`) is cleaner but backend-owns gives more control |
| DisplayList location | Own crate vs lui-models | TBD | Own crate keeps models free of rendering concepts |
