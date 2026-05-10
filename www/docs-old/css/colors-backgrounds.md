---
title: Colors & Backgrounds
---

# Colors and Backgrounds

wgpu-html supports a broad range of CSS color formats and background properties. Colors are parsed into a typed `CssColor` enum and resolved to `[r, g, b, a]` linear sRGB at paint time for GPU consumption.

## Color Formats

### Hexadecimal

All four CSS hex formats are supported:

```css
color: #f00;           /* #rgb → #ff0000 */
color: #f00f;          /* #rgba → #ff0000ff */
color: #ff0000;        /* #rrggbb */
color: #ff000080;      /* #rrggbbaa (50% alpha) */
```

### RGB / RGBA Functions

```css
color: rgb(255, 0, 0);
color: rgb(100%, 0%, 0%);
color: rgba(255, 0, 0, 0.5);
color: rgba(100%, 0%, 0%, 50%);
```

Components accept integers (0–255), percentages (0%–100%), and bare floats (0.0–255.0). Alpha accepts floats (0.0–1.0) or percentages.

### HSL / HSLA Functions

```css
color: hsl(0, 100%, 50%);              /* red */
color: hsl(120deg, 100%, 25%);         /* green (dark) */
color: hsla(240, 100%, 50%, 0.7);      /* semi-transparent blue */
```

Hue accepts degrees (bare number or `deg` suffix). Saturation and lightness are percentages.

### Named Colors

```
black, white, red, green, blue, yellow, cyan, aqua,
magenta, fuchsia, gray, grey, lightgray, lightgrey,
darkgray, darkgrey, silver, maroon, olive, lime,
teal, navy, purple, orange, pink
```

```css
color: orange;
background-color: navy;
border-color: silver;
```

### CSS Color Module Level 4 System Colors

System colors are used primarily in the UA stylesheet for form controls. They are resolved to specific sRGB values at paint time:

```css
background-color: canvas;
color: canvastext;
border-color: buttonborder;
background-color: field;
color: fieldtext;
background-color: buttonface;
color: buttontext;
```

Full system color list: `canvas`, `canvastext`, `linktext`, `visitedtext`, `activetext`, `buttonface`, `buttontext`, `buttonborder`, `field`, `fieldtext`, `highlight`, `highlighttext`, `selecteditem`, `selecteditemtext`, `mark`, `marktext`, `graytext`, `accentcolor`, `accentcolortext`.

### Special Color Keywords

```css
color: transparent;      /* rgba(0, 0, 0, 0) */
color: currentColor;     /* resolves to None — no fallback */
```

> **Note:** `currentColor` is parsed as `CssColor::CurrentColor` but resolves to `None` in the paint pass. Borders without an explicit color are skipped (no fallback to `color` or `currentColor`).

## `background-color`

Sets a solid color fill for the element's background:

```css
.box {
  background-color: #f0f0f0;
  background-color: rgb(240, 240, 240);
  background-color: hsl(0, 0%, 94%);
  background-color: lightgray;
  background-color: transparent;     /* no fill */
}
```

The background color is painted as a quad (or SDF-rounded quad when `border-radius` is set) clipped to the `background-clip` box.

## `background-clip`

Controls which box the background extends to:

```css
background-clip: border-box;    /* background fills to outer border edge (default) */
background-clip: padding-box;   /* background stops at inner border edge (outside padding) */
background-clip: content-box;   /* background stops at content edge */
```

```
border-box:  ┌────────────────────┐
             │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  border
             │▓┌──────────────────┐│
             │▓│  padding +       ││
             │▓│  content         ││
             │▓└──────────────────┘│
             └────────────────────┘

padding-box: ┌──border─────────────┐
             │  ┌──────────────────┐│
             │  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓││
             │  │▓padding +       ││
             │  │▓content         ││
             │  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓││
             │  └──────────────────┘│
             └──────────────────────┘

content-box: ┌──border─────────────┐
             │  ┌──padding─────────┐│
             │  │  ┌──────────────┐││
             │  │  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓│││
             │  │  │▓content     ▓│││
             │  │  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓│││
             │  │  └──────────────┘││
             │  └──────────────────┘│
             └──────────────────────┘
```

When combined with `border-radius`, the background clip uses concentric inner radii: `padding-box` radius = `max(0, border_radius - border_width)`, `content-box` radius = `max(0, padding_radius - padding)`.

## `background-image`

Loads an image from a URL as the element's background:

```css
.hero {
  background-image: url("hero.jpg");
  background-image: url("https://example.com/bg.png");
  background-image: none;       /* remove background image */
}
```

The URL can be:
- A relative or absolute file path
- An HTTP(S) URL
- A `data:` URI
- Quoted (`url("path")`) or unquoted (`url(path)`)

Images are loaded asynchronously by the layout crate, cached with TTL and byte-budget eviction, and rendered through the image pipeline. Supported formats include PNG, JPEG, GIF (including animated), and WebP.

## CSS Gradients

All three CSS gradient types are supported as `background-image` values, including their `repeating-*` variants. Gradients are rasterized to RGBA pixel buffers at layout time and rendered through the existing image pipeline.

### `linear-gradient()`

```css
/* Direction keywords */
background: linear-gradient(to right, red, blue);
background: linear-gradient(to bottom right, #ff6b6b, #feca57);

/* Angle */
background: linear-gradient(45deg, #a29bfe, #fd79a8);
background: linear-gradient(0.5turn, white, black);

/* Multiple color stops with positions */
background: linear-gradient(to right, #00b894 0%, #0984e3 50%, #6c5ce7 100%);

/* Default direction is "to bottom" */
background: linear-gradient(red, blue);
```

Supported direction formats:
- `to top`, `to bottom`, `to left`, `to right` and diagonal combinations (`to top right`, etc.)
- Angles: `deg`, `rad`, `turn`, `grad`

### `radial-gradient()`

```css
/* Default: ellipse farthest-corner at center */
background: radial-gradient(white, black);

/* Circle */
background: radial-gradient(circle, #fdcb6e, #e17055);

/* Custom center position */
background: radial-gradient(circle at 30% 70%, #74b9ff, #0984e3);

/* Size keywords */
background: radial-gradient(circle closest-side, #55efc4, #00b894);
background: radial-gradient(ellipse farthest-corner at 70% 30%, #fab1a0, #e17055);

/* Explicit size */
background: radial-gradient(circle 100px, red, blue);
```

Shape keywords: `circle`, `ellipse` (default).
Size keywords: `closest-side`, `farthest-side`, `closest-corner`, `farthest-corner` (default).

### `conic-gradient()`

```css
/* Rainbow wheel */
background: conic-gradient(red, yellow, lime, aqua, blue, magenta, red);

/* Start angle */
background: conic-gradient(from 45deg, #ff6348, #ffa502, #2ed573, #1e90ff, #ff6348);

/* Custom center */
background: conic-gradient(at 30% 70%, #e17055, #fdcb6e, #55efc4, #e17055);
```

### `repeating-*` variants

```css
/* Diagonal stripes */
background: repeating-linear-gradient(45deg, #e17055 0px, #e17055 10px, #fdcb6e 10px, #fdcb6e 20px);

/* Concentric rings */
background: repeating-radial-gradient(circle, #2d3436 0px, #2d3436 10px, #636e72 10px, #636e72 20px);

/* Pinwheel */
background: repeating-conic-gradient(#e17055 0deg, #fdcb6e 30deg, #55efc4 60deg);
```

### Color stops

Color stops support all CSS color formats (named, hex, `rgb()`, `rgba()`, `hsl()`, `hsla()`, `transparent`) with optional position in `%` or `px`:

```css
background: linear-gradient(to right, red 0%, green 50%, blue 100%);
background: repeating-linear-gradient(90deg, red 0px, blue 20px);
```

Stops without explicit positions are distributed evenly between their neighbors.

### Gradients with other background properties

Gradients interact with `background-size`, `background-position`, `background-repeat`, and `background-clip`:

```css
.tiled-gradient {
  background-image: linear-gradient(red, blue);
  background-size: 50px 50px;
  background-repeat: repeat;
}

.clipped-gradient {
  background: radial-gradient(circle, white, navy);
  background-clip: padding-box;
  border-radius: 12px;
  border: 4px solid transparent;
}
```

## `background-repeat`

Controls tiling of the background image:

```css
background-repeat: repeat;       /* tile both axes (default) */
background-repeat: repeat-x;     /* tile horizontally only */
background-repeat: repeat-y;     /* tile vertically only */
background-repeat: no-repeat;    /* single image, no tiling */
```

## `background-size` and `background-position`

Both properties are parsed from CSS and stored as raw strings:

```css
background-size: cover;
background-size: contain;
background-size: 100% auto;
background-size: 200px 150px;

background-position: center;
background-position: top left;
background-position: 50% 50%;
background-position: 10px 20px;
```

These are consumed by the layout engine from their raw string form. Full structuring for all values is not yet implemented.

## sRGB → Linear Color Conversion

The quad pipeline converts sRGB colors to linear space for correct GPU blending. The WGSL shader performs the conversion:

```
// sRGB to linear (approximate gamma 2.2)
linear = pow(srgb, 2.2);
```

This ensures that color blending, opacity compositing, and anti-aliasing produce visually correct results on gamma-correct surfaces.

## Code Examples

### Background with Rounded Clip

```css
.card {
  background-color: #ffffff;
  border: 1px solid #e0e0e0;
  border-radius: 12px;
  background-clip: padding-box;
  padding: 24px;
}
```

### Tiled Background Pattern

```css
.pattern-bg {
  background-color: #f5f5f5;
  background-image: url("pattern.png");
  background-repeat: repeat;
}
```

### Gradient Card

```css
.card {
  background: linear-gradient(145deg, #2d3436, #636e72);
  border-radius: 12px;
  padding: 24px;
  color: white;
}
```

### Gradient Button

```css
.btn {
  background: linear-gradient(to right, #f093fb, #f5576c);
  border-radius: 24px;
  padding: 12px 32px;
  color: white;
}
```

### Hero Section

```css
.hero {
  background: linear-gradient(135deg, #667eea, #764ba2);
  color: white;
  padding: 80px 24px;
}
```

### System Color Form

```css
input[type="text"] {
  background-color: field;
  color: fieldtext;
  border: 1px solid buttonborder;
  padding: 6px 12px;
  border-radius: 4px;
}

input[type="text"]:focus {
  border-color: highlight;
  background-color: field;
}
```

### Rust: Resolving Colors Programmatically

```rust
use wgpu_html_models::CssColor;

// Parse a color string
let color = parse_css_color("hsla(200, 80%, 50%, 0.8)");

// Resolve to [r, g, b, a] in linear space
fn resolve_color(c: &CssColor) -> [f32; 4] {
    match c {
        CssColor::Named(name) => resolve_named_color(name),
        CssColor::Hex(h) => resolve_hex_color(h),
        CssColor::Rgb(r, g, b) => srgb_to_linear([*r, *g, *b, 255]),
        CssColor::Rgba(r, g, b, a) => srgb_to_linear([*r, *g, *b, (*a * 255.0) as u8]),
        CssColor::Hsl(h, s, l) => hsl_to_srgb(*h, *s, *l),
        CssColor::Hsla(h, s, l, a) => {
            let [r, g, b] = hsl_to_srgb(*h, *s, *l);
            [r, g, b, *a]
        }
        CssColor::Transparent => [0.0, 0.0, 0.0, 0.0],
        CssColor::CurrentColor => panic!("unresolved currentColor"),
        CssColor::Function(_) => [0.0, 0.0, 0.0, 1.0],
    }
}
```
