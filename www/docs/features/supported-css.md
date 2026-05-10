---
sidebar_position: 2
---

# Supported CSS

## Layout

| Property | Support | Notes |
|---|---|---|
| `display` | ✅ | block, inline, inline-block, flex, inline-flex, grid, inline-grid, table/table-row/table-cell/table-caption/table-row-group/table-header-group/table-footer-group/table-column/table-column-group, none, contents (partial), ruby/ruby-text |
| `position` | ✅ | static, relative, absolute, fixed; sticky degraded to relative |
| `top`, `right`, `bottom`, `left` | ✅ | Consumed by positioned layout |
| `float` | ❌ | Not implemented |
| `z-index` | ⚠️ | Sibling sort done; no independent stacking contexts |

## Box Model

| Property | Support | Notes |
|---|---|---|
| `width`, `height` | ✅ | px, %, em, rem, vw, vh, vmin, vmax, calc(), auto |
| `min-width`, `max-width`, `min-height`, `max-height` | ✅ | Clamping applied in layout |
| `margin` (all sides + shorthand) | ✅ | 1–4 value expansion, px/%/auto |
| `padding` (all sides + shorthand) | ✅ | 1–4 value expansion |
| `box-sizing` | ✅ | content-box, border-box |
| `aspect-ratio` | ❌ | Not implemented |

## Borders

| Property | Support | Notes |
|---|---|---|
| `border` (shorthand) | ✅ | width, style, color |
| `border-{side}-width` | ✅ | Per-side widths |
| `border-{side}-style` | ⚠️ | solid, dashed, dotted, none, hidden; double/groove/ridge/inset/outset render as solid |
| `border-{side}-color` | ✅ | Per-side colors |
| `border-radius` | ✅ | 1–4 corner expansion + elliptical syntax (`/`) |
| `border-image` | ❌ | Deferred only |

## Backgrounds

| Property | Support | Notes |
|---|---|---|
| `background-color` | ✅ | Solid colors with rounded corners |
| `background-image` | ✅ | URL images + CSS gradients |
| `background-size` | ✅ | cover, contain, px, % |
| `background-position` | ✅ | Keywords + px/% |
| `background-repeat` | ✅ | repeat, no-repeat, repeat-x, repeat-y |
| `background-clip` | ✅ | border-box, padding-box, content-box |
| `linear-gradient()` | ✅ | With angle + color stops |
| `radial-gradient()` | ✅ | Circle/ellipse + color stops |
| `conic-gradient()` | ✅ | With angle + color stops |
| `repeating-*` gradients | ✅ | All three gradient types |
| `background-attachment` | ❌ | Deferred |
| `background-origin` | ❌ | Deferred |

## Typography

| Property | Support | Notes |
|---|---|---|
| `color` | ✅ | All color formats |
| `font-family` | ✅ | Registered fonts + generic fallback |
| `font-size` | ✅ | px, em, rem, %, keywords |
| `font-weight` | ✅ | 100–900, normal/bold/bolder/lighter |
| `font-style` | ✅ | normal, italic, oblique |
| `line-height` | ✅ | unitless, px, em, % |
| `letter-spacing` | ✅ | px, em |
| `text-align` | ✅ | left, right, center, justify |
| `text-transform` | ✅ | uppercase, lowercase, capitalize |
| `text-decoration` | ✅ | underline, line-through, overline |
| `white-space` | ✅ | normal, pre, nowrap, pre-wrap, pre-line |
| `word-break` | ✅ | normal, break-all, keep-all |
| `vertical-align` | ✅ | baseline, sub, super, top, middle, bottom, text-top, text-bottom, length |
| `text-indent` | ❌ | Deferred |
| `word-spacing` | ❌ | Deferred |
| `@font-face` | ❌ | Not implemented |

## Flexbox

All CSS Flexbox Level 1 properties are fully supported:

| Property | Support |
|---|---|
| `display: flex` / `inline-flex` | ✅ |
| `flex-direction` (row, row-reverse, column, column-reverse) | ✅ |
| `flex-wrap` (nowrap, wrap, wrap-reverse) | ✅ |
| `justify-content` (all values) | ✅ |
| `align-items`, `align-self` | ✅ |
| `align-content` (multi-line) | ✅ |
| `flex-grow`, `flex-shrink`, `flex-basis`, `flex` | ✅ |
| `order` | ✅ |
| `gap`, `row-gap`, `column-gap` | ✅ |
| `margin: auto` (absorbs free space) | ✅ |

## Grid

CSS Grid Level 1 is mostly supported:

| Property | Support |
|---|---|
| `display: grid` / `inline-grid` | ✅ |
| `grid-template-columns`, `grid-template-rows` | ✅ (px, fr, auto, repeat(), minmax()) |
| `grid-auto-columns`, `grid-auto-rows` | ✅ |
| `grid-column-start/end`, `grid-row-start/end` | ✅ (line + span) |
| `grid-column`, `grid-row` (shorthands) | ✅ |
| `justify-items`, `align-items` | ✅ |
| `justify-self`, `align-self` | ✅ |
| `justify-content`, `align-content` | ✅ |
| `gap`, `row-gap`, `column-gap` | ✅ |
| `grid-auto-flow` | ✅ |
| `grid-template-areas` | ❌ | Deferred |
| `grid-area` (shorthand) | ⚠️ | Expands to line-based placement only |
| `repeat(auto-fill/auto-fit)` | ❌ | Not implemented |
| `dense` packing | ❌ | Not implemented |
| Named lines, subgrid | ❌ | Not implemented |

## Effects

| Property | Support | Notes |
|---|---|---|
| `opacity` | ✅ | Multiplied through subtree |
| `visibility` | ✅ | visible, hidden, collapse |
| `box-shadow` | ⚠️ | Parsed but stored raw — no paint |
| `transform` | ⚠️ | Parsed but stored raw — no paint |
| `filter` | ❌ | Not implemented |

## Interactivity

| Property | Support | Notes |
|---|---|---|
| `cursor` | ✅ | CSS cursor values resolved |
| `pointer-events` | ✅ | none skips hit-testing, auto works; inherited |
| `user-select` | ✅ | none prevents text selection; inherited |
| `resize` | ✅ | Both/none/horizontal/vertical (textarea) |
| `accent-color` | ✅ | Checkbox/radio/range/color controls |

## Scrolling & Overflow

| Property | Support | Notes |
|---|---|---|
| `overflow`, `overflow-x`, `overflow-y` | ✅ | visible, hidden, scroll, auto |
| `text-overflow` | ✅ | clip, ellipsis |
| `scrollbar-color` | ✅ | Track/thumb colors |
| `scrollbar-width` | ✅ | Thin/normal |
| `scroll-behavior` | ❌ | Not implemented |

## Other

| Property | Support | Notes |
|---|---|---|
| `var()` / custom properties `--*` | ✅ | Inherited, recursive substitution, cycle detection |
| `calc()`, `min()`, `max()`, `clamp()` | ✅ | Full math expression engine (also sin/cos/tan/sqrt/pow/etc.) |
| `@media` | ✅ | width, height, orientation, min/max-, `not` |
| `!important` | ✅ | Full cascade band handling |
| CSS-wide keywords (inherit/initial/unset) | ✅ | Per-property resolution |
| `::before`, `::after` | ✅ | Content rendering |
| `::first-line`, `::first-letter` | ✅ | Color only |
| `::placeholder` | ✅ | Input/textarea placeholder styling |
| `::selection` | ✅ | Selected text color + background |
| `::file-selector-button` | ✅ | File input button styling |
| `::lui-*` pseudo-elements | ✅ | Popup/picker internal styling |
| SVG presentation attributes | ✅ | fill, stroke, stroke-width, stroke-linecap/linejoin, stroke-dasharray/dashoffset, fill-opacity, stroke-opacity, fill-rule |
