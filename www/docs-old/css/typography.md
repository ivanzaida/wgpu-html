---
title: Typography & Text Properties
---

# Typography and Text Properties

lui uses [cosmic-text](https://github.com/pop-os/cosmic-text) (backed by HarfBuzz for shaping and rustybuzz for fallback) to shape and render text. The typography system supports font family fallback, weight/style selection, text transformation, decorations, alignment, and more.

## `color`

Sets the foreground color of text. Inherited from parent:

```css
body { color: #333333; }
.highlight { color: #e74c3c; }
.muted { color: rgba(0, 0, 0, 0.5); }
```

Supports all [color formats](./colors-backgrounds): hex, rgb()/rgba(), hsl()/hsla(), named colors, system colors, `transparent`.

Each glyph instance carries its own color tint, resolved from the element's cascaded `color` property at paint time.

## `font-family`

A comma-separated list of font family names. The engine tries each family in order until it finds a registered font:

```css
font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
font-family: Georgia, "Times New Roman", serif;
font-family: "Fira Code", "Cascadia Code", monospace;
font-family: system-ui, sans-serif;
```

### Generic Family Keywords

When no specific font matches, these generic families provide fallback:

| Keyword | Description |
|---|---|
| `sans-serif` | Default sans-serif system font |
| `serif` | Default serif system font |
| `monospace` | Default monospace system font |
| `cursive` | Cursive/handwriting style |
| `fantasy` | Decorative/display font |
| `system-ui` | Platform's default UI font |
| `ui-serif`, `ui-sans-serif`, `ui-monospace`, `ui-rounded` | Platform UI fonts specific to each style |

### Registering Fonts

Fonts are registered on the `Tree` before layout:

```rust
use lui_tree::FontFace;

tree.register_font(FontFace {
    family: "Helvetica Neue".into(),
    path: "fonts/HelveticaNeue.ttf".into(),
    weight: None,
    style: None,
});

// Or register system fonts
lui_winit::register_system_fonts(&mut tree, "Arial");
```

## `font-size`

Sets the size of text. Inherited:

```css
font-size: 16px;
font-size: 1.2em;          /* relative to parent's font-size */
font-size: 1rem;           /* relative to root font-size */
font-size: 2vw;            /* viewport-relative */
font-size: 150%;           /* 150% of parent font-size */
font-size: calc(1em + 2px);
```

The default font size (when not inherited and not set) is 16px.

## `font-weight`

Controls the thickness of glyphs:

```css
font-weight: normal;       /* 400 */
font-weight: bold;         /* 700 */
font-weight: bolder;       /* one step heavier than parent */
font-weight: lighter;      /* one step lighter than parent */
font-weight: 100;          /* thin */
font-weight: 400;          /* normal */
font-weight: 700;          /* bold */
font-weight: 900;          /* black/heavy */
```

Weight is selected from available font faces. If the exact weight isn't available, the closest match is used.

## `font-style`

Controls italic/oblique rendering:

```css
font-style: normal;
font-style: italic;
font-style: oblique;
```

`italic` and `oblique` select the italic/oblique variant of the font face if available. If not available, synthetic italicization is not applied.

## `line-height`

Controls the height of each line box:

```css
line-height: 1.5;          /* unitless — 1.5 × font-size (preferred) */
line-height: 24px;         /* absolute */
line-height: 150%;         /* percentage of font-size */
line-height: 1.5em;        /* em — identical to 150% */
line-height: normal;       /* browser default (~1.25 × font-size) */
```

A unitless number (the preferred method in CSS) multiplies the element's font-size and is inherited as the multiplier, not the computed pixel value. This ensures descendant elements with different font sizes still get proportional line heights.

The default is approximately 1.25× font-size.

## `letter-spacing`

Adds or removes space between glyphs. Applied as a post-shape per-glyph x-offset:

```css
letter-spacing: normal;         /* default spacing */
letter-spacing: 0.5px;          /* add 0.5px between each glyph */
letter-spacing: -1px;           /* tighter spacing */
letter-spacing: 0.1em;          /* relative to font-size */
```

The offset is applied to each `PositionedGlyph` after text shaping.

## `text-transform`

Transforms text case before shaping:

```css
text-transform: none;           /* unchanged */
text-transform: uppercase;      /* ALL CAPS */
text-transform: lowercase;      /* all lowercase */
text-transform: capitalize;     /* First Letter Of Each Word Capitalized */
```

The transformation is applied to the text content **before** it is sent to the shaper, so glyph selection, kerning, and layout all operate on the transformed text.

## `text-align`

Sets horizontal alignment of text within block-level elements:

```css
text-align: left;          /* left-aligned (for LTR) */
text-align: right;         /* right-aligned (for LTR) */
text-align: center;        /* centered */
text-align: start;         /* start edge (left in LTR, right in RTL) */
text-align: end;           /* end edge */
```

Alignment is applied per line box in the inline formatting context. The entire line is shifted within the containing block's content width.

## `text-decoration`

Draws decorative lines on text:

```css
text-decoration: none;           /* no decoration */
text-decoration: underline;      /* line below text */
text-decoration: overline;       /* line above text */
text-decoration: line-through;   /* line through middle of text */
```

Decorations are painted as solid quads after glyph rendering:
- **Underline** — positioned at the font's underline metric
- **Overline** — positioned near the ascent
- **Line-through** — positioned at approximately x-height / 2

Decoration color is the same as the text `color` property.

## `white-space`

Controls whitespace handling:

```css
white-space: normal;       /* collapse whitespace, wrap at boundaries (default) */
white-space: pre;          /* preserve all whitespace, no wrapping */
```

- **`normal`** — sequences of whitespace are collapsed to a single space. Text wraps at the container width.
- **`pre`** — all whitespace (spaces, tabs, newlines) is preserved. Text does not wrap.

Other `white-space` values (`nowrap`, `pre-wrap`, `pre-line`, `break-spaces`) are parsed but may have limited or partial layout support.

## `vertical-align`

Controls vertical alignment of inline-level elements:

```css
vertical-align: baseline;
vertical-align: sub;
vertical-align: super;
vertical-align: text-top;
vertical-align: text-bottom;
vertical-align: middle;
vertical-align: top;
vertical-align: bottom;
vertical-align: 5px;
vertical-align: -2px;
```

> **Note:** `vertical-align` is parsed and its values are recognized, but layout consumption is limited. Only a subset of values (primarily baseline and length offsets) have meaningful layout effects. Superscript/subscript text placement is not yet implemented.

## Complete Typography Example

```css
body {
  font-family: "Inter", system-ui, sans-serif;
  font-size: 16px;
  font-weight: 400;
  font-style: normal;
  line-height: 1.6;
  color: #1a1a2e;
}

h1 {
  font-family: "Playfair Display", Georgia, serif;
  font-size: 2.5rem;
  font-weight: 700;
  line-height: 1.2;
  letter-spacing: -0.5px;
  text-align: center;
  color: #16213e;
}

.lead {
  font-size: 1.25rem;
  line-height: 1.8;
  color: rgba(26, 26, 46, 0.8);
}

code {
  font-family: "Fira Code", "Cascadia Code", monospace;
  font-size: 0.9em;
  color: #e74c3c;
}

a {
  color: #3498db;
  text-decoration: underline;
}

a:hover {
  text-decoration: none;
  color: #2980b9;
}

.muted {
  font-size: 0.875rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #999;
}
```

### Rust: Font Registration

```rust
// Register custom fonts
tree.register_font(FontFace {
    family: "Inter".into(),
    path: "fonts/Inter-Regular.ttf".into(),
    weight: Some(400),
    style: Some(FontStyle::Normal),
});

tree.register_font(FontFace {
    family: "Inter".into(),
    path: "fonts/Inter-Bold.ttf".into(),
    weight: Some(700),
    style: Some(FontStyle::Normal),
});

// Register system fonts for fallback
use lui_winit::register_system_fonts;
register_system_fonts(&mut tree, "Arial");
register_system_fonts(&mut tree, "Segoe UI");
```
