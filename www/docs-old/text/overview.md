---
title: Text Rendering Overview
---

# Text Rendering Overview

Text rendering in lui is handled by the `lui-text` crate, which owns the shaping pipeline, font database, and CPU-side glyph atlas.

## Font Database

Fonts are registered on the `Tree` via `FontRegistry`. Each font face is a `(family, weight, style)` tuple backed by raw TTF/OTF bytes:

```rust
tree.register_font(FontFace {
    family: "Inter".into(),
    weight: 400,
    style: FontStyleAxis::Normal,
    data: Arc::from(include_bytes!("Inter-Regular.otf") as &[u8]),
});
```

The registry lives on the document (`Tree::fonts`), not globally. Font matching is a simplified CSS-Fonts-3 algorithm: exact match preferred, then fallback through family list, then system fonts.

## Cosmic-Text Shaping Pipeline

The `TextContext` wraps `FontDb` (cosmic-text bridge) and `SwashCache` for glyph rasterisation:

```rust
pub struct TextContext {
    // Owns FontDb (cosmic-text fonts), Atlas (CPU R8 atlas), caches
}
```

Shaping flow per text run:
1. Resolve `font-family` list, `font-weight`, `font-style` to a `FontHandle`.
2. Apply `text-transform` (uppercase/lowercase/capitalize) pre-shape.
3. Normalize whitespace per `white-space` value.
4. Feed text + font handle + size + letter-spacing to cosmic-text.
5. For each shaped glyph, look up `(font, glyph_id, subpx_offset, size)` in the atlas cache.
6. On miss: rasterize via SwashCache, shelf-pack into the atlas.
7. Emit `PositionedGlyph` with run-relative pixel rect and atlas UVs.

## Glyph Atlas

The glyph atlas is a 2048×2048 `R8Unorm` texture shared between CPU and GPU:

```rust
pub const GLYPH_ATLAS_SIZE: u32 = 2048;
```

- **CPU side** (`lui-text::Atlas`): shelf-packing allocator. Each glyph gets a rectangular region. Dirty rects accumulate and are flushed to the GPU each frame.
- **GPU side** (`lui-renderer::GlyphPipeline`): samples the atlas texture with UV coordinates, multiplies coverage by per-glyph color.

## Per-Glyph Text Color

Each `PositionedGlyph` carries its own linear RGBA color:

```rust
pub struct PositionedGlyph {
    pub x: f32, pub y: f32, pub w: f32, pub h: f32,
    pub uv_min: [f32; 2], pub uv_max: [f32; 2],
    pub color: [f32; 4],
}
```

Color is resolved from the cascaded `color` property at the span level. Rich-text paragraphs (multiple inline styles) produce glyphs with per-span colors.

## Text Decorations

Three decoration lines are supported, painted as solid quads:

```rust
pub enum TextDecorationLine {
    Underline,
    LineThrough,
    Overline,
}
```

Each text leaf's `LayoutBox::text_decorations` carries the set of active lines. The paint pass emits one thin quad per decoration line at the appropriate vertical offset.

## Sub-Pages

- [Font System](./fonts) — registration, matching, system fonts
- [Text Shaping](./shaping) — shaping pipeline, CSS properties, IFC wrapping
