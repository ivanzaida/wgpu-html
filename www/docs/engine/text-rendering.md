---
sidebar_position: 7
---

# Text Rendering

Text rendering spans two crates: `wgpu-html-text` for font management and shaping, and the glyph pipeline in `wgpu-html-renderer` for GPU rendering.

## Font Database (`wgpu-html-text`)

### Registering Fonts

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "path/to/font.ttf".into(),
    weight: 400,
    style: FontStyle::Normal,
});
```

Fonts are matched by `family` → `weight` → `style`, with a configurable fallback to the first registered font. Multiple fonts can be registered for the same family with different `weight` and `style` values. Supported formats: `.ttf`, `.otf`, `.ttc`.

### System Font Discovery

The winit driver includes helpers for automatic system font registration:

```rust
tree.register_system_fonts("sans-serif");
```

This scans common OS font directories (Windows Fonts, /usr/share/fonts, /System/Library/Fonts).

## Text Shaping

Powered by **cosmic-text** (which wraps HarfBuzz), text shaping produces positioned glyphs:

1. **Span collection** — the layout engine collects inline text spans with their resolved styles
2. **Rich text layout** — `shape_paragraph()` sends all spans as a paragraph with inline style changes
3. **Line breaking** — word wrapping, soft hyphens, and forced breaks (`<br>`, `<wbr>`)
4. **Glyph positions** — per-glyph `(x, y)` positions and atlas UV coordinates

## Glyph Atlas

Glyphs are rasterized on the CPU and uploaded to a GPU texture atlas:

- **2048×2048 RGBA atlas** — shelf-packing allocator for glyph placement
- **CPU rasterization** — cosmic-text rasterizes glyphs at the requested size
- **GPU upload** — atlas texture updates are batched and uploaded to the glyph pipeline each frame

## Text Properties

| Property | Status | Notes |
|---|---|---|
| `font-family` | ✅ | Registered fonts + generic fallback; no `@font-face` |
| `font-size` | ✅ | px, em, rem, % |
| `font-weight` | ✅ | 100–900 with font matching |
| `font-style` | ✅ | normal, italic, oblique |
| `line-height` | ✅ | Affects line box height |
| `letter-spacing` | ✅ | Extra advance between glyphs |
| `text-align` | ✅ | left, right, center, justify |
| `text-transform` | ✅ | uppercase, lowercase, capitalize |
| `white-space` | ✅ | normal, pre, nowrap, pre-wrap, pre-line |
| `text-decoration` | ✅ | underline, line-through, overline (rendered at correct vertical offsets) |
| `text-overflow` | ❌ | Not implemented |
| `word-break` | ❌ | Not implemented |
| `vertical-align` | ❌ | Not implemented |

## Glyph Pipeline (GPU)

The `GlyphPipeline` in `wgpu-html-renderer` handles GPU text rendering:

- **Instanced rendering** — one instance per glyph with position, UV, and color
- **Alpha blending** — glyphs blend in display space (non-sRGB view)
- **Per-glyph color** — resolved from the `color` CSS property through the cascade, plus selection highlighting
- **Clip respect** — glyphs honor the active clip range (scissor + rounded SDF mask)
- **Linear filtering** — smooth edges on atlas sampling

## Text Editing

The text editing system supports:
- Insertion and deletion with caret navigation (arrow keys, Home, End)
- Shift-selection for range selection
- Word and line selection (double-click, triple-click)
- Clipboard copy (Ctrl+C) and paste (Ctrl+V)
- Multiline editing in `<textarea>`
- Password masking (U+2022 bullets) for `<input type="password">`
