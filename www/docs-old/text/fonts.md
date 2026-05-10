---
title: Font System
---

# Font System

Fonts are per-document assets registered on `Tree::fonts` (`FontRegistry`). The `lui-text` crate consumes `&FontRegistry` to build its shaping-side database.

## Font Registration

```rust
use std::sync::Arc;
use lui_tree::{FontFace, FontStyleAxis, Tree};

let mut tree = Tree::new(root_node);

tree.register_font(FontFace {
    family: "Inter".into(),
    weight: 400,
    style: FontStyleAxis::Normal,
    data: Arc::from(include_bytes!("Inter-Regular.otf") as &[u8]),
});

tree.register_font(FontFace {
    family: "Inter".into(),
    weight: 700,
    style: FontStyleAxis::Normal,
    data: Arc::from(include_bytes!("Inter-Bold.otf") as &[u8]),
});

tree.register_font(FontFace {
    family: "Inter".into(),
    weight: 400,
    style: FontStyleAxis::Italic,
    data: Arc::from(include_bytes!("Inter-Italic.otf") as &[u8]),
});
```

Supports `.ttf`, `.otf`, and `.ttc` file formats. `data` is `Arc<[u8]>` so the same bytes can be registered under multiple family aliases without copying.

## Family + Weight + Style Matching

CSS `font-family`, `font-weight`, and `font-style` cascade down to each node. The layout engine resolves the font:

```rust
let families = parse_family_list(style.font_family.as_deref());
let family_refs: Vec<&str> = families.iter().map(String::as_str).collect();
let weight = font_weight_value(style.font_weight.as_ref());
let axis = font_style_axis(style.font_style.as_ref());

let handle = ctx.text.ctx.pick_font(&family_refs, weight, axis);
```

The matching algorithm:
1. Try each family in the `font-family` list (first match wins).
2. Within a family, score faces by weight distance (closest to target).
3. If target is italic/oblique: prefer italic faces, fall back to normal.
4. If no match in any family, fall back to the first registered system font.
5. If nothing registered at all, text is not shaped (empty run).

## System Font Discovery

System fonts can be enumerated and registered automatically:

```rust
use lui_tree::system_font_variants;

let variants: Vec<SystemFontVariant> = system_font_variants();
for v in &variants {
    tree.register_font(FontFace {
        family: v.family.clone(),
        weight: v.weight,
        style: v.style,
        data: v.data.clone(),
    });
}
```

The `system_font_variants()` function works on:
| Platform | Source |
|---|---|
| Windows | `%WINDIR%\Fonts` directory |
| Linux | `fontconfig` (when enabled) or `/usr/share/fonts` |
| macOS | System font directories |

A convenience helper registers all system fonts for a specific family:

```rust
use lui_tree::register_system_fonts;

register_system_fonts(&mut tree, "Arial");
```

## Font Metrics and Sizing

`font-size` resolves to physical pixels via the CSS pixel → physical pixel scale factor:

```rust
let size_css = font_size_px(style).unwrap_or(16.0);  // CSS px
let size_px = size_css * ctx.scale;                    // physical px
```

`line-height` can be a plain number (multiplier), length, or percentage:

```rust
let line_h_css = line_height_px_for_font(style, size_css, &ctx.text.ctx, handle);
let line_height = line_h_css * ctx.scale;
```

## FontFace Struct

```rust
pub struct FontFace {
    pub family: String,
    pub weight: u16,           // 100-900 (100 = Thin, 400 = Normal, 700 = Bold, 900 = Black)
    pub style: FontStyleAxis,  // Normal | Italic | Oblique
    pub data: Arc<[u8]>,       // TTF/OTF/WOFF bytes
}

pub enum FontStyleAxis {
    Normal,
    Italic,
    Oblique,
}
```

## Re-registration

Re-registering a face with the same `(family, weight, style)` overrides the previous one. Later registration wins on ties during matching — this lets hosts reload fonts at runtime.

## Generation Tracking

`FontRegistry::generation()` returns a monotonically increasing counter. `TextContext::sync_fonts()` compares this against its last-seen value to skip redundant reloading.
