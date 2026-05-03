---
id: css/syntax
title: CSS Syntax & Data Types
---

# CSS Syntax and Basic Data Types

wgpu-html parses CSS values into strongly-typed Rust enums and structs. This page documents the supported syntax for each value type.

## Length Units

Length values are parsed into the `CssLength` enum:

```rust
pub enum CssLength {
  Px(f32),
  Percent(f32),
  Em(f32),
  Rem(f32),
  Vw(f32),
  Vh(f32),
  Vmin(f32),
  Vmax(f32),
  Auto,
  Zero,
  Calc(Box<CssMathExpr>),
  Min(Vec<CssLength>),
  Max(Vec<CssLength>),
  Clamp { min: Box<CssLength>, preferred: Box<CssLength>, max: Box<CssLength> },
  Raw(String),
}
```

### Absolute Units

| Unit | Description | Example | Rust |
|---|---|---|---|
| `px` | CSS pixels | `16px` | `CssLength::Px(16.0)` |
| `0` | Zero (unitless) | `0` | `CssLength::Zero` |

The bare value `0` is treated as a special zero that does not require a unit, matching CSS specification behaviour.

### Relative Units

| Unit | Description | Example |
|---|---|---|
| `%` | Percentage of containing block | `50%` |
| `em` | Relative to element's font-size | `1.5em` |
| `rem` | Relative to root font-size | `1.2rem` |
| `vw` | 1% of viewport width | `100vw` |
| `vh` | 1% of viewport height | `50vh` |
| `vmin` | 1% of viewport's smaller dimension | `50vmin` |
| `vmax` | 1% of viewport's larger dimension | `50vmax` |

Note: `em` and `rem` use a hard-coded 16px fallback when no font-size is inherited.

### Auto

The `auto` keyword resolves to `CssLength::Auto` and is used for automatic sizing in width, height, margins, and positioned offsets.

### Math Functions as Lengths

Length values can be produced by CSS math functions:

```css
width: calc(100% - 20px);
height: min(50vh, 400px);
font-size: max(12px, 2vw);
padding: clamp(8px, 2vw, 32px);
```

These parse into `CssLength::Calc(Box<CssMathExpr>)`, `CssLength::Min(Vec<CssLength>)`, `CssLength::Max(Vec<CssLength>)`, and `CssLength::Clamp { ... }` respectively. See the [Math Functions](./math-functions) page for full details.

## Colors

Colors are parsed into the `CssColor` enum:

```rust
pub enum CssColor {
  Named(String),
  Hex(String),
  Rgb(u8, u8, u8),
  Rgba(u8, u8, u8, f32),
  Hsl(f32, f32, f32),
  Hsla(f32, f32, f32, f32),
  Transparent,
  CurrentColor,
  Function(String),
}
```

### Hexadecimal Notation

```css
color: #f00;          /* #rgb → CssColor::Hex("#f00") */
color: #ff0000ff;     /* #rrggbbaa → CssColor::Hex("#ff0000ff") */
color: #ff0000;       /* #rrggbb → CssColor::Hex("#ff0000") */
color: #f00f;         /* #rgba → CssColor::Hex("#f00f") */
```

All four hex formats are supported. `#rgb` expands to `#rrggbb` and `#rgba` expands to `#rrggbbaa` during conversion to `[r, g, b, a]` at resolution time.

### RGB / RGBA Functions

```css
color: rgb(255, 0, 0);         /* → CssColor::Rgb(255, 0, 0) */
color: rgb(100%, 0%, 0%);      /* percentage values */
color: rgba(255, 0, 0, 0.5);   /* → CssColor::Rgba(255, 0, 0, 0.5) */
color: rgba(100%, 0%, 0%, 50%);/* alpha as percentage */
```

### HSL / HSLA Functions

```css
color: hsl(0, 100%, 50%);              /* → CssColor::Hsl(0.0, 100.0, 50.0) */
color: hsla(240, 100%, 50%, 0.7);      /* → CssColor::Hsla(240.0, 100.0, 50.0, 0.7) */
color: hsl(120deg, 80%, 60%);          /* hue with deg suffix */
```

### Named Colors

20+ named colors are supported:

```
black, white, red, green, blue, yellow, cyan/aqua, magenta/fuchsia,
gray/grey, lightgray/lightgrey, darkgray/darkgrey, silver,
maroon, olive, lime, teal, navy, purple, orange, pink
```

### CSS Color Module Level 4 System Colors

System colors from CSS Color Level 4 are supported, primarily used in the UA stylesheet for form control styling:

```
canvas, canvastext, linktext, visitedtext, activetext,
buttonface, buttontext, buttonborder,
field, fieldtext,
highlight, highlighttext,
selecteditem, selecteditemtext,
mark, marktext, graytext,
accentcolor, accentcolortext
```

```css
background-color: buttonface;
color: buttontext;
```

### Special Color Keywords

| Keyword | Description |
|---|---|
| `transparent` | `rgba(0, 0, 0, 0)` |
| `currentColor` | References the element's `color` property; resolves to `None` in wgpu-html |

## Number Values

Bare numbers are used for several properties:

```css
flex-grow: 1;      /* → parsed as f32 */
flex-shrink: 0;    /* → parsed as f32 */
order: -1;         /* → i32 */
z-index: 10;       /* → i32 */
opacity: 0.5;      /* → f32, clamped to [0, 1] */
```

The `flex` shorthand accepts `flex-grow`, optional `flex-shrink`, and optional `flex-basis`:

```css
flex: 1;               /* 1 1 0% */
flex: 1 0 auto;        /* grow=1, shrink=0, basis=auto */
flex: none;             /* 0 0 auto */
```

## String Values

Some properties accept arbitrary string values:

- `font-family: "Helvetica Neue", Arial, sans-serif`
- `cursor: pointer`
- `transform: rotate(45deg) scale(1.2)` (raw, not consumed)
- `background-image: url("image.png")`

## Shorthand vs Longhand Properties

wgpu-html handles shorthand expansion during parsing:

```css
/* These are equivalent: */
margin: 10px 20px;
/* Expands to: */
margin-top: 10px;
margin-right: 20px;
margin-bottom: 10px;
margin-left: 20px;
```

Shorthand expansion follows CSS rules:
- **1 value** — applies to all four sides
- **2 values** — first is top/bottom, second is left/right
- **3 values** — first is top, second is left/right, third is bottom
- **4 values** — top, right, bottom, left (clockwise)

When a shorthand is applied after longhands, it marks the longhand fields as "reset" so that earlier longhand values don't leak through:

```css
margin-left: 5px;
margin: 10px;     /* margin-left reset to 10px, not 5px */
```

## Keyword Values

Many properties accept predefined keyword values. These are parsed into Rust enums:

```rust
// Parsing display: flex
pub enum Display {
  None, Block, Inline, InlineBlock, ListItem,
  Flex, InlineFlex, Grid, InlineGrid,
  Table, TableCaption, // + more (fall through to block)
}

// Parsing position: absolute
pub enum Position {
  Static, Relative, Absolute, Fixed, Sticky,
}
```

Keywords are case-insensitive in CSS but stored as CamelCase in Rust.

## How Values Are Parsed into Rust Types

The core parsing function is `apply_css_property(style: &mut Style, property: &str, value: &str)` in `wgpu-html-parser/src/css_parser.rs`. It dispatches on the property name:

```rust
pub fn apply_css_property(style: &mut Style, property: &str, value: &str) {
  // Custom properties (--*)
  if property.starts_with("--") {
    style.custom_properties.insert(property.to_owned(), value.to_owned());
    return;
  }
  // Values containing var() are deferred
  if value_contains_var(value) {
    style.var_properties.insert(property.to_owned(), value.to_owned());
    return;
  }
  match property {
    "display" => style.display = parse_display(value),
    "width" => style.width = parse_css_length(value),
    "color" => style.color = parse_css_color(value),
    // ... ~80 properties
  }
}
```

CSS-wide keywords (`inherit`, `initial`, `unset`) are intercepted before property-specific parsers and stored in side-car keyword maps for the cascade to resolve against the parent's computed style.
