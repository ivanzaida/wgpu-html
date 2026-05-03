---
id: pipelines
title: GPU Pipelines
---

# GPU Pipelines

wgpu-html renders through three custom GPU pipelines, each with its own WGSL shader, instancing strategy, and bind group layout.

## Quad Pipeline (`quad.wgsl`)

The quad pipeline draws filled and stroked rectangles with rounded corners via Signed Distance Field (SDF) math.

### Per-Quad Instance Data

```rust
struct QuadInstance {
    pos: [f32; 2],       // top-left corner in physical pixels
    size: [f32; 2],      // width, height
    color: [f32; 4],     // linear RGBA
    radii_h: [f32; 4],   // horizontal corner radii (TL, TR, BR, BL)
    radii_v: [f32; 4],   // vertical corner radii (TL, TR, BR, BL)
    stroke: [f32; 4],    // per-side ring thickness (top, right, bottom, left)
    pattern: [f32; 4],   // (kind, dash_len, gap_len, _pad)
}
```

### Two Modes

- **Filled**: `stroke == [0, 0, 0, 0]`. The shader fills the rounded box with `color`, discarding fragments outside the rounded SDF.
- **Stroked ring**: At least one stroke component > 0. The shader paints only the ring between the outer and inner rounded boxes. Inner box is inset by the stroke width on each side.

### Dashed/Dotted Patterns

```rust
pattern: [f32; 4]  // (kind, dash, gap, _)
// kind: 0 = solid, 1 = dashed, 2 = dotted
```

Pattern-based stroking is honored for one-sided rings (exactly one positive stroke component). The fragment shader computes arc-length along the ring's centerline and toggles visibility based on dash/gap intervals.

### Per-Clip Rounded Discard

Each clip slot carries a uniform block:

```rust
struct Globals {
    viewport: [f32; 4],
    clip_rect: [f32; 4],
    clip_radii_h: [f32; 4],
    clip_radii_v: [f32; 4],
    clip_active: [f32; 4],  // 1.0 = run SDF discard
}
```

When `clip_active.x == 1.0`, the fragment shader runs an additional SDF test against the clip rect and discards fragments outside. This provides rounded clipping for `overflow: hidden` + `border-radius` combos.

### Instancing

The pipeline is drawn via `draw_indexed(unit_quad_indices, instances)` — one instance per quad. The vertex shader expands a unit quad to the instance's rect using pos + size.

## Glyph Pipeline (`glyph.wgsl`)

The glyph pipeline draws alpha-tested glyphs from the R8Unorm glyph atlas.

### Per-Glyph Instance Data

```rust
struct GlyphInstance {
    pos: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],      // per-glyph tint (resolved text color)
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}
```

### Atlas Sampling

The pipeline binds a single `R8Unorm` 2048×2048 atlas texture and a linear sampler. The fragment shader samples the atlas at the UV coordinates and multiplies the coverage value by `color`:

```wgsl
let coverage = textureSample(atlas, atlas_sampler, uv).r;
let output = vec4(color.rgb * color.a * coverage, color.a * coverage);
```

### Gamma-Correct Blending

The glyph pass renders to a **non-sRGB view** of the surface texture. This means the GPU treats the texture as raw linear values — blending happens in display space (gamma-correct for anti-aliased edges). If the surface format is already non-sRGB, the glyph view is the same view.

### Per-Clip Discard

Same `Globals` uniform + SDF discard logic as the quad pipeline. Shared clip slot buffer, separate bind group.

## Image Pipeline

The image pipeline draws textured quads for `<img>` elements and `background-image`.

### Per-Image Instance

```rust
struct ImageQuad {
    rect: Rect,
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    image_id: u64,       // keys the GPU texture cache
    tint: [f32; 4],      // opacity multiplier
}
```

### Texture Cache

The renderer maintains a GPU texture cache keyed by `image_id`. When a new image ID appears in the display list:
1. Upload the RGBA8 bytes to a new GPU texture.
2. Store in the cache (LRU eviction when full).
3. Bind for the current frame.

Animated images have per-frame `image_id` values — the layout crate writes the current frame's ID, and the renderer uploads new frames on change.

## WGSL Shader Files

| Pipeline | Shader |
|---|---|
| Quad | `quad.wgsl` — SDF rounded rect, ring stroke, dash/dot patterns, clip discard |
| Glyph | `glyph.wgsl` — R8 atlas sample, alpha-tinted output, clip discard |
| Image | `image.wgsl` — RGBA8 texture sample, tint multiply |

All shaders are compiled at pipeline creation time (no runtime compilation). Vertex shaders transform instance data into NDC space using the viewport dimensions from the globals uniform.
