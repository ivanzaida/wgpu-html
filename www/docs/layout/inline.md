---
id: inline
title: Inline Formatting Context
---

# Inline Formatting Context (IFC)

The inline formatting context handles mixed text and inline-block children. When a block-level element's children are all inline-level (`display: inline` or text nodes), the engine builds line boxes.

## Line-Box Layout

For a mixed inline context, the engine:

1. Collects each child into an `InlineLayout` struct (text runs via shape, inline-blocks via recursive layout).
2. Builds a list of `Line` objects, each containing a slice of `InlineLayout` items.
3. Word-wraps each line at the container width, breaking between inline items and at soft-break opportunities within text runs.
4. Sizes each line's height from the maximum ascent + descent across its items.
5. Vertically aligns items within the line baseline.

```rust
struct Line {
    items: Vec<InlineLayout>,
    width: f32,
    ascent: f32,
    descent: f32,
    y: f32,
}
```

## Word Wrapping

Text runs are shaped with an optional `max_width_px`. When wrapping is enabled (non-`pre` white-space), cosmic-text inserts word-boundary breaks. The IFC layer additionally handles wrapping between inline-block elements — when the current line overflows, a new line box starts.

## text-align

`text-align` is applied per line box:

```css
text-align: left | right | center | justify;
```

Each line's items are shifted horizontally within the container width. For single-text-leaf paragraphs, the entire `ShapedRun` is translated — a heuristic since proper per-line alignment requires per-line glyph repositioning.

```rust
let align_dx = horizontal_align_offset(text_align, container_w, line_width);
```

## Inline-Block Elements

`display: inline-block` elements are laid out as block boxes but placed inline within the line flow. Each inline-block:

1. Is laid out recursively at its content size.
2. Contributes its margin box height to the line's ascent/descent.
3. Can be broken across lines only if it is wider than the remaining line space.

## Anonymous Text Runs

Raw text nodes (those not wrapped in an element) produce anonymous text leaves. Each text leaf:

1. Has its `font-family`, `font-size`, `font-weight`, `font-style`, `color`, and `letter-spacing` resolved from the cascaded style (inherited from ancestors).
2. Is shaped via `TextContext::shape_and_pack()`.
3. Is placed as a `BoxKind::Text` with the shaped run in `text_run`.

## Rich-Text Paragraph Shaping

When a block contains multiple inline children (e.g., `<span>bold</span> normal <em>italic</em>`), the engine uses a rich-text path:

1. Flattens the inline subtree into `(text, Attrs)` spans.
2. Feeds them to cosmic-text via `set_rich_text()` — word-boundary breaks land between spans while preserving per-span attributes.
3. Re-expands the result into anonymous block boxes for per-line backgrounds and `BoxKind::Text` leaves for glyphs, with per-glyph color baked in by `shape_paragraph`.

## Example

```html
<div style="width: 400px; text-align: justify; line-height: 1.6;">
    <strong>Bold intro:</strong>
    This paragraph has <em>mixed formatting</em>
    and an inline-block <span style="display:inline-block;
    width:80px; height:30px; background:#ddd;"></span> in the flow.
</div>
```

The result is a justified 400px-wide block with bold, italic, and normal text on the same lines, plus the 80×30 inline-block aligned to the text baseline.
