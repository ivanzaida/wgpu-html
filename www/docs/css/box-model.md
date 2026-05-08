---
title: Box Model
---

# CSS Box Model

wgpu-html implements the standard CSS box model with content, padding, border, and margin boxes. The box model affects
layout geometry (spacing, sizing) and paint (background clipping, border rendering).

## Box Model Structure

```
┌─────────────────────────────────────────────┐
│                  margin                     │
│  ┌───────────────────────────────────────┐  │
│  │               border                  │  │
│  │  ┌─────────────────────────────────┐  │  │
│  │  │            padding              │  │  │
│  │  │  ┌───────────────────────────┐  │  │  │
│  │  │  │        content            │  │  │  │
│  │  │  │                           │  │  │  │
│  │  │  └───────────────────────────┘  │  │  │
│  │  └─────────────────────────────────┘  │  │
│  └───────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

The three canonical rectangles used throughout the layout and paint systems:

| Rectangle      | Definition                  | Use                                   |
|----------------|-----------------------------|---------------------------------------|
| `content_rect` | Content area inside padding | Child layout, inline text flow        |
| `border_rect`  | Content + padding + border  | Paint box, background positioning     |
| `margin_rect`  | Border + margin             | Flow spacing, parent's content sizing |

Geometry assertions in tests use these three rectangles explicitly.

## `width` and `height`

The `width` and `height` properties set the content-box dimensions by default:

```css
.box {
    width: 300px;
    height: 200px;
}
```

Accepted values:

- `<length>`: `px`, `em`, `rem`, `vw`, `vh`, `vmin`, `vmax`, `%`
- `auto`: fills available container width, or sum of children for height
- `calc()`: computed lengths
- `min()`, `max()`, `clamp()`: math function lengths

### `min-width`, `max-width`, `min-height`, `max-height`

These clamp the computed width/height:

```css
.responsive {
    width: 100%;
    max-width: 800px;
    min-width: 320px;
}

.card {
    height: auto;
    min-height: 200px;
    max-height: 500px;
}
```

Clamping is applied in the layout pass after resolving `width`/`height`.

## `box-sizing`

Controls whether `width`/`height` include padding and border:

```css
/* Default: width = content width */
.box-content {
    box-sizing: content-box;
    width: 200px; /* content is 200px wide */
    padding: 10px; /* border_rect is 220px wide */
    border: 1px; /* border_rect is 222px wide */
}

/* Border box: width = content + padding + border */
.box-border {
    box-sizing: border-box;
    width: 200px; /* border_rect is 200px wide */
    padding: 10px; /* content is 178px wide */
    border: 1px; /* content is 176px wide */
}
```

The layout engine subtracts padding and border from the specified width/height when `box-sizing: border-box`.

## Margin Properties

### `margin` Shorthand

The `margin` shorthand accepts 1–4 values:

```css
margin:

10
px

; /* all four sides */
margin:

10
px

20
px

; /* top/bottom, left/right */
margin:

10
px

20
px

30
px

; /* top, left/right, bottom */
margin:

10
px

20
px

30
px

40
px

; /* top, right, bottom, left */
```

### Per-Side Longhands

```css
margin-top:

10
px

;
margin-right:

20
px

;
margin-bottom:

30
px

;
margin-left:

40
px

;
```

### `auto` Margins

`auto` margins enable horizontal centering in block layout:

```css
.centered {
    width: 600px;
    margin-left: auto;
    margin-right: auto;
}
```

When both left and right margins are `auto`, the remaining horizontal space is split equally, centering the element.

## Padding Properties

### `padding` Shorthand

Same 1–4 value expansion as `margin`:

```css
padding:

10
px

;
padding:

10
px

20
px

;
padding:

10
px

20
px

30
px

;
padding:

10
px

20
px

30
px

40
px

;
```

Percent values are relative to the containing block's width.

### Per-Side Longhands

```css
padding-top:

8
px

;
padding-right:

16
px

;
padding-bottom:

8
px

;
padding-left:

16
px

;
```

## Border Properties

### `border` Shorthand

The `border` shorthand sets width, style, and color for all four sides simultaneously. Values can appear in any order:

```css
border:

1
px solid red

;
border:

2
px dashed #333

;
border:

3
px dotted

rgba
(
0
,
0
,
255
,
0.5
)
;
```

The shorthand fans out to all four per-side longhands during parsing. Missing components leave existing values intact (
cascade order: `border: 2px solid red; border-top: 4px dashed blue;` works correctly).

### Per-Side Border Shorthands

```css
border-top:

2
px solid blue

;
border-right:

1
px dashed gray

;
border-bottom:

3
px double green

;
border-left:

1
px solid transparent

;
```

### Border Longhands

Each side has three sub-properties:

```css
border-top-width:

2
px

;
border-top-style: solid

;
border-top-color: red

;
```

Available border styles:

| Style    | Description                           | Rendering                     |
|----------|---------------------------------------|-------------------------------|
| `none`   | No border                             | Skipped                       |
| `hidden` | No border (table conflict resolution) | Skipped                       |
| `solid`  | Single solid line                     | Quad / SDF ring               |
| `dashed` | Dashed line segments                  | Segment loop / patterned ring |
| `dotted` | Dotted line                           | Segment loop / patterned ring |
| `double` | Two solid lines                       | Falls through to solid        |
| `groove` | 3D grooved effect                     | Falls through to solid        |
| `ridge`  | 3D ridged effect                      | Falls through to solid        |
| `inset`  | 3D inset effect                       | Falls through to solid        |
| `outset` | 3D outset effect                      | Falls through to solid        |

> **Note:** `double`, `groove`, `ridge`, `inset`, and `outset` are parsed but rendered as plain solid. Dashed/dotted on
> rounded boxes follow the curve only when all four corners are uniform-circular; otherwise corners stay bare.

## `border-radius`

wgpu-html implements the full `border-radius` specification with elliptical (horizontal/vertical) radii and per-corner
expansion.

### Uniform Radius

```css
border-radius:

5
px

; /* all corners */
border-radius:

5
px

10
px

; /* top-left+bottom-right, top-right+bottom-left */
border-radius:

5
px

10
px

15
px

; /* top-left, top-right+bottom-left, bottom-right */
border-radius:

5
px

10
px

15
px

20
px

; /* top-left, top-right, bottom-right, bottom-left */
```

### Elliptical Radius

The `/` separator specifies different horizontal and vertical radii:

```css
border-radius:

10
px

/
5
px

; /* h=10px, v=5px all corners */
border-radius:

10
px

20
px

/
5
px

10
px

; /* different h/v per pair */
```

### Per-Corner Longhands

Each corner has separate horizontal (H) and vertical (V) components:

```css
border-top-left-radius:

10
px

5
px

; /* H=10px, V=5px */
border-top-right-radius:

20
px

;
border-bottom-right-radius:

15
px

10
px

;
border-bottom-left-radius:

5
px

;
```

### Rendering

Rounded corners are rendered through the SDF (Signed Distance Field) quad pipeline. The shader evaluates per-pixel
distance from the rounded rectangle shape for smooth anti-aliased corners. CSS3 corner-overflow clamping is applied:
when the sum of two adjacent radii exceeds the box's dimension, both radii are scaled down proportionally.

## Code Examples

### Complete Box Model Example

```css
.card {
    box-sizing: border-box;
    width: 300px;

    margin: 16px auto; /* vertical 16px, horizontal centering */
    padding: 20px 24px; /* vertical 20px, horizontal 24px */
    border: 1px solid #ddd;

    border-radius: 8px;

    background-color: white;
    background-clip: padding-box;
}
```

### Layout Effect Diagram

```
┌────────── margin (16px) ──────────────┐
│ ┌────── border (1px) ───────────────┐ │
│ │ ┌── padding (20px/24px) ────────┐ │ │
│ │ │                                │ │ │
│ │ │     content: 300px - 2×1       │ │ │
│ │ │            - 2×24              │ │ │
│ │ │          = 250px               │ │ │
│ │ │                                │ │ │
│ │ └────────────────────────────────┘ │ │
│ └────────────────────────────────────┘ │
└────────────────────────────────────────┘

margin_rect width:  300 + 32 = 332px
border_rect width:  300px
content_rect width: 300 - 2 - 48 = 250px
```

### Border Style Combinations

```css
.panel {
    border-top: 3px solid #2196F3;
    border-right: 1px solid #ddd;
    border-bottom: 1px solid #ddd;
    border-left: 1px solid #ddd;
    border-radius: 4px;
}
```

### Shorthand vs Longhand Cascade

```css
.card {
    border: 2px solid red; /* sets all four sides */
    border-left: 4px dashed blue; /* overrides left side only */
    border-top-color: green; /* overrides top color, width/style from shorthand */
}
```
