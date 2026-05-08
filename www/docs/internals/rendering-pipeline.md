---
title: Rendering Pipeline
---

# Rendering Pipeline

How a `LayoutBox` tree becomes pixels on screen.

## Paint Stage

The paint stage walks the layout tree and emits drawing primitives into a `DisplayList`.

**Entry point:** `crates/wgpu-html/src/paint.rs`

| Function | Line | Purpose |
|---|---|---|
| `paint_layout_full()` | 148 | Entry: builds clip stack, walks layout tree with interaction state |
| `paint_box_in_clip()` | 247 | Core recursive painter for one box + children |

### Paint Order (per box)

`paint_box_in_clip()` processes each box in this order:

1. **Background color** (line ~274) -- solid fill with `background-clip` rect and corner radii
2. **Background image** (line ~294) -- pre-tiled image quads, clipped to rounded corners
3. **Borders** (line ~323) -- uniform ring or per-side edges depending on color uniformity
4. **`<img>` content** (line ~335) -- textured quad in content rect
5. **Text glyphs** (line ~354) -- shaped glyph quads with decorations, selection, caret
6. **Overflow clip push** (line ~504) -- if `overflow: hidden`, push rounded-corner clip frame
7. **Children** (line ~540) -- recursively paint in z-index order
8. **Resize handle** (line ~568) -- three-line indicator for `resize` property
9. **Scrollbars** (line ~590) -- track + thumb for overflow axes

### Z-Index Sorting

Children are sorted into three layers before painting (line ~542):

```
z < 0   -> layer -1 (painted first, behind everything)
z = auto -> layer  0 (document order)
z > 0   -> layer  1 (painted last, on top)
```

Sort key function at `z_index_sort_key()` (line 743). Within each layer, stable sort preserves document order.

### Clipping

The display list maintains a clip stack. Each `overflow: hidden` element pushes a `ClipFrame`:

- **Rectangular scissor** -- intersection of all ancestor clips
- **Rounded SDF discard** -- fragment shader discards pixels outside rounded corners using `clip_radii_h`/`clip_radii_v`

Clip push at line ~504, pop at the end of children iteration.

## DisplayList

**File:** `crates/wgpu-html-renderer/src/paint.rs`

```rust
pub struct DisplayList {
    pub quads: Vec<Quad>,              // solid color + stroke quads
    pub images: Vec<ImageQuad>,        // textured image quads
    pub glyphs: Vec<GlyphQuad>,        // glyph quads (atlas sampling)
    pub clips: Vec<ClipRange>,         // scissor ranges
    pub commands: Vec<DisplayCommand>, // interleaved paint order
    pub canvas_color: Option<[f32; 4]>, // CSS canvas background (from root/body)
}
```

Each `DisplayCommand` references one item (quad/image/glyph) and its clip range:

```rust
pub struct DisplayCommand {
    pub kind: DisplayCommandKind, // Quad | Image | Glyph
    pub index: u32,               // index into quads/images/glyphs
    pub clip_index: u32,          // which ClipRange
}
```

Key methods:

| Method | Line | Purpose |
|---|---|---|
| `push_clip()` | 231 | Open new scissor range |
| `pop_clip()` | 250 | Close range, return to parent |
| `finalize()` | 281 | Remap indices, drop empty clips |
| `push_quad()` | 346 | Solid filled quad |
| `push_quad_stroke_ellipse()` | 428 | Stroked ring with elliptical corners |
| `push_glyph()` | ~451 | Glyph quad with atlas UVs |
| `push_image_with_opacity()` | ~451 | Image quad with opacity |

`finalize()` (line 281) is critical -- it extends open clip ranges, drops empty ones, and remaps command clip indices.

## GPU Renderer

**File:** `crates/wgpu-html-renderer/src/lib.rs`

### Initialization

| Function | Line | Purpose |
|---|---|---|
| `Renderer::new()` | 50 | Create renderer bound to window surface |
| `Renderer::headless()` | 141 | Offscreen renderer (no surface) |

Creates wgpu instance, adapter, device, queue, surface config, and three pipelines.

### Three GPU Pipelines

| Pipeline | File | Vertex | Per-Instance Data |
|---|---|---|---|
| **QuadPipeline** | `quad_pipeline.rs` | Unit quad (instanced) | pos, size, color, radii_h/v, stroke, pattern |
| **ImagePipeline** | `image_pipeline.rs` | Unit quad (instanced) | pos, size, opacity + bound texture |
| **GlyphPipeline** | `glyph_pipeline.rs` | Unit quad (instanced) | pos, size, color, uv_min/max |

**QuadPipeline** handles:
- Solid filled rectangles (stroke = 0)
- Stroked rings / borders (stroke > 0 on any side)
- Dashed/dotted patterns via `pattern` field (kind + dash/gap lengths)
- SDF rounded corners in fragment shader

**ImagePipeline** handles:
- One GPU texture per unique image (cached in `HashMap<image_id, CachedImage>`)
- Linear bilinear filtering, clamp-to-edge
- Per-instance opacity

**GlyphPipeline** handles:
- R8 atlas texture (2048x2048 default) shared across all glyphs
- Per-glyph UV coordinates into atlas
- Alpha blend in display space (non-sRGB view) for correct anti-aliasing

### Canvas Background (CSS 2.2 section 14.2)

Per the CSS spec, the root element's background (or the body's, if the root has none) propagates to fill the entire viewport canvas. This is implemented via `DisplayList.canvas_color`:

1. `paint_layout_full()` extracts the root's `background` color. If the root (`<html>`) has no background, it falls back to the first child (`<body>`).
2. The color is stored in `list.canvas_color`.
3. `Renderer::render()` reads `canvas_color` and sets it as the wgpu `clear_color` before the render passes, so the body background fills the entire surface.

### Render Pass

| Function | Line | Purpose |
|---|---|---|
| `Renderer::render()` | 413 | Acquire surface, prepare pipelines, encode + submit |
| `record_ordered_commands()` | 509 | Walk `DisplayList.commands` in order, draw per clip/kind |
| `record_legacy_batches()` | 602 | Fallback: all quads, then images, then glyphs |

`render()` creates two texture views from the surface:
- **sRGB view** -- for quads and images (linear color space blending)
- **Non-sRGB view** -- for glyphs (alpha blending in gamma-encoded space)

### Offscreen Rendering

| Function | Line | Purpose |
|---|---|---|
| `capture_to()` | 227 | Render to texture, save as PNG |
| `render_to_rgba()` | 327 | Render to texture, return RGBA8 bytes |

## Border Rendering

**File:** `crates/wgpu-html/src/paint.rs`

| Case | Function | Approach |
|---|---|---|
| All sides same color + solid | `uniform_border_color()` (line 323) | Single ring quad with SDF stroke |
| Mixed color/style + rounded | `paint_rounded_per_side_borders()` (line 950) | One ring per side, SDF restricts to that side |
| Mixed color/style + sharp | `paint_border_edges()` (line 1130) | Four edge quads |
| Dashed/dotted uniform curve | Pattern shader (line 1002) | SDF pattern on uniform-circular borders only |
| Dashed/dotted sharp/elliptical | `paint_segments()` (line 1283) | Segment quads along edge |

Detection logic at `uniform_border_color()` checks if all non-zero sides share the same color. If yes, one ring quad. Otherwise, per-side fallback.

## Text Rendering

Shaped text arrives as `ShapedRun` from `wgpu-html-text`. During paint (lines 354-502):

1. Resolve text color (per-leaf `text_color` * opacity)
2. Emit decoration quads (underline/overline/line-through, thickness = ascent/12)
3. Emit selection background quads (per-glyph range)
4. For each glyph: clip UVs if bleeding outside text box, select color (normal/selection/edit), push glyph quad
5. Emit edit caret (thin vertical bar at cursor position, 500ms blink cycle)

## Image & Gradient Pipeline

Background images are pre-tiled during layout into `BackgroundImagePaint`:

```rust
pub struct BackgroundImagePaint {
    pub image_id: u64,
    pub data: Arc<Vec<u8>>,   // decoded RGBA8
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Rect>,     // pre-positioned in physical pixels
}
```

At paint time, the painter iterates tiles and emits one image quad per tile. Gradients are CPU-rasterized into images during layout and flow through the same image pipeline.
