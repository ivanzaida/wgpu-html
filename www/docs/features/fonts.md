---
sidebar_position: 6
---

# Fonts

## Font Registration

wgpu-html does not ship fonts. You must register them at startup:

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Regular.ttf".into(),
    weight: 400,
    style: FontStyle::Normal,
});
```

Supported formats: `.ttf`, `.otf`, `.ttc` (TrueType Collection).

## Font Matching

Fonts are matched by `family` → `weight` → `style`:
- Family: exact match on the registered `family` string
- Weight: closest available weight (100–900 range)
- Style: `Normal` or `Italic`/`Oblique`

If no exact family match is found, falls back to the first registered font with a matching generic family (sans-serif, serif, monospace).

## Multiple Weights and Styles

Register multiple font files for the same family to enable weight and style variations:

```rust
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Regular.ttf".into(),
    weight: 400,
    ..Default::default()
});
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Bold.ttf".into(),
    weight: 700,
    ..Default::default()
});
tree.register_font(FontFace {
    family: "sans-serif".into(),
    file: "fonts/Roboto-Italic.ttf".into(),
    weight: 400,
    style: FontStyle::Italic,
});
```

## System Font Discovery

The winit driver includes a helper for automatic system font registration:

```rust
tree.register_system_fonts("sans-serif");
```

This scans standard OS font directories:
- **Windows**: `C:\Windows\Fonts\`
- **Linux**: `/usr/share/fonts/`, `~/.local/share/fonts/`
- **macOS**: `/System/Library/Fonts/`, `/Library/Fonts/`, `~/Library/Fonts/`

```rust
// Discover available font variants without registering
let variants = system_font_variants("sans-serif");
```

## Glyph Atlas

Fonts are rasterized on the CPU and stored in a shared **2048×2048 RGBA glyph atlas** texture on the GPU. The atlas uses a shelf-packing allocator to place glyph bitmaps efficiently.

## Current Limitations

- No `@font-face` support for web fonts
- No variable font axis support (`font-variation-settings`)
- No `font-kerning`, `font-language-override`, `font-optical-sizing`
- `em`/`rem` use a hard-coded 16px when no font-size is inherited
