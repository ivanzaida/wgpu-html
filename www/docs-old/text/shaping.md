---
title: Text Shaping
---

# Text Shaping

Text shaping converts a string + CSS font properties into positioned, atlas-packed glyphs. The pipeline lives in `lui-text::shape.rs`.

## Shaping Pipeline

The main entry point is `TextContext::shape_and_pack()`:

```rust
pub fn shape_and_pack(
    &mut self,
    text: &str,
    handle: FontHandle,
    size_px: f32,
    line_height: f32,
    letter_spacing: f32,
    weight: u16,
    style: FontStyleAxis,
    max_width_px: Option<f32>,
    color: [f32; 4],
) -> Option<ShapedRun>
```

Steps:
1. Build cosmic-text `Buffer` with `Attrs` (family, size, weight, style).
2. Set text content via `buffer.set_text(text, attrs, Shaping::Advanced)`.
3. Use cosmic-text's `shape_until_scroll()` to get layout runs.
4. Extract glyph positions, scale to physical pixels, apply letter-spacing.
5. For each glyph: atlas cache lookup → shelf-pack → emit `PositionedGlyph`.
6. Package lines, glyphs, byte boundaries into `ShapedRun`.

## font-family List Fallback

The `font-family` CSS property accepts a comma-separated list. The engine tries each family in order:

```css
code { font-family: "JetBrains Mono", "Consolas", monospace; }
```

`pick_font()` iterates the list and returns the first match. If none match, no font is selected and the text run is empty — visual fallback requires registering appropriate system fonts.

## font-weight and font-style

- **font-weight**: numeric value (100–900) or keyword (`normal` = 400, `bold` = 700). Mapped to the nearest registered weight.
- **font-style**: `normal`, `italic`, or `oblique`. Italic and oblique are interchangeable during matching (preferred order: exact → the other → normal).

## font-size Resolution

Keywords (`small`, `medium`, `large`, etc.) are resolved against the UA default of 16px. Units (`px`, `em`, `rem`, `pt`, `%`) are resolved during cascade.

## letter-spacing

Applied as a post-shape per-glyph horizontal offset:

```rust
let letter_spacing = letter_spacing_px(style, size_css) * ctx.scale;
```

Each glyph's x-position is incremented by `letter_spacing × glyph_index`. This keeps shaping (kerning, ligatures) intact while adding the spacing.

## text-transform

Applied pre-shape so font features (ligatures, contextual alternates) operate on the transformed text:

```rust
let transformed = apply_text_transform(&normalized, style.text_transform.as_ref());
```

| Value | Effect |
|---|---|
| `uppercase` | All characters upper-cased |
| `lowercase` | All characters lower-cased |
| `capitalize` | First character of each word upper-cased |
| `none` (default) | No transformation |

## white-space

| Value | Behavior |
|---|---|
| `normal` | Collapse whitespace, wrap at line edges |
| `pre` | Preserve whitespace, no wrapping |
| `nowrap` | Collapse whitespace, no wrapping |
| `pre-wrap` | Preserve whitespace, wrap at line edges |
| `pre-line` | Collapse whitespace, preserve line breaks, wrap |

Whitespace collapse turns runs of spaces/tabs/newlines into a single space before shaping.

## Core Types

```rust
pub struct ShapedRun {
    pub glyphs: Vec<PositionedGlyph>,
    pub glyph_chars: Vec<usize>,      // glyph→char index mapping
    pub lines: Vec<ShapedLine>,       // per-line metrics
    pub text: String,                 // visible text after transforms
    pub byte_boundaries: Vec<usize>,  // UTF-8 char boundaries
    pub width: f32,
    pub height: f32,
    pub ascent: f32,
}

pub struct ShapedLine {
    pub top: f32,
    pub height: f32,
    pub glyph_range: (usize, usize),  // half-open slice into ShapedRun::glyphs
}

pub struct PositionedGlyph {
    pub x: f32, pub y: f32,
    pub w: f32, pub h: f32,
    pub uv_min: [f32; 2], pub uv_max: [f32; 2],
    pub color: [f32; 4],
}
```

## Line Wrapping in IFC

In the Inline Formatting Context, `ShapedRun::lines` provides the break points. Each `ShapedLine::glyph_range` identifies which glyphs belong to which line. The IFC layer uses these to:

1. Position each line at the correct y-offset.
2. Apply `text-align` per line (offsetting glyphs horizontally).
3. Size the containing block's height to the sum of line heights.

For single-text-leaf paragraphs with `text-align: right` or `center`, the entire `ShapedRun` is translated rather than per-line — a heuristic sufficient for most cases.
