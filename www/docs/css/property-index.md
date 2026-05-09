---
title: CSS Property Index
---

# CSS Property Index

This is the complete index of all supported CSS properties in wgpu-html, formatted as a property table. Columns:

- **Property** — CSS property name (kebab-case)
- **Values** — accepted syntax
- **Initial** — CSS specification initial value
- **Inherited** — whether the property inherits by default
- **Notes** — implementation status (✅ fully implemented, ⚠️ partial, ❌ parsed only)

---

## Layout & Box Model

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `display` | `block` \| `inline` \| `inline-block` \| `flex` \| `grid` \| `none` \| (9 table variants) | `inline` | No | ✅ Block/flex/grid/none; table falls through to block |
| `position` | `static` \| `relative` \| `absolute` \| `fixed` | `static` | No | ✅ Absolute/relative/fixed; sticky degrades to relative |
| `width` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ All units, `calc()`, `min()`/`max()`/`clamp()` |
| `height` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ All units |
| `min-width` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ Clamping during layout |
| `max-width` | `<length>` \| `<percentage>` \| `auto` | `none` | No | ✅ Clamping during layout |
| `min-height` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ Clamping during layout |
| `max-height` | `<length>` \| `<percentage>` \| `auto` | `none` | No | ✅ Clamping during layout |
| `box-sizing` | `content-box` \| `border-box` | `content-box` | No | ✅ Affects width/height interpretation |
| `margin` | `<length>` \| `<percentage>` \| `auto` (1–4 values) | `0` | No | ✅ Shorthand with 1–4 value expansion |
| `margin-top` | `<length>` \| `<percentage>` \| `auto` | `0` | No | ✅ Auto margin horizontal centering |
| `margin-right` | `<length>` \| `<percentage>` \| `auto` | `0` | No | ✅ |
| `margin-bottom` | `<length>` \| `<percentage>` \| `auto` | `0` | No | ✅ |
| `margin-left` | `<length>` \| `<percentage>` \| `auto` | `0` | No | ✅ |
| `padding` | `<length>` \| `<percentage>` (1–4 values) | `0` | No | ✅ Shorthand with 1–4 value expansion |
| `padding-top` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `padding-right` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `padding-bottom` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `padding-left` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `top` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ Offsets for positioned elements |
| `right` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ Right/bottom anchoring |
| `bottom` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ |
| `left` | `<length>` \| `<percentage>` \| `auto` | `auto` | No | ✅ |

## Borders

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `border` | `<width>` `<style>` `<color>` (any order) | `medium none currentColor` | No | ✅ Shorthand fans out to all four sides |
| `border-top` | `<width>` `<style>` `<color>` | see individual | No | ✅ Per-side shorthand |
| `border-right` | `<width>` `<style>` `<color>` | see individual | No | ✅ |
| `border-bottom` | `<width>` `<style>` `<color>` | see individual | No | ✅ |
| `border-left` | `<width>` `<style>` `<color>` | see individual | No | ✅ |
| `border-width` | `<length>` (1–4 values) | `medium` | No | ⚠️ Parsed + applied as length |
| `border-style` | `none` \| `hidden` \| `solid` \| `dashed` \| `dotted` \| `double` \| `groove` \| `ridge` \| `inset` \| `outset` | `none` | No | ⚠️ `double`/`groove`/`ridge`/`inset`/`outset` fall through to solid |
| `border-color` | `<color>` (1–4 values) | `currentColor` | No | ⚠️ Border without explicit color is skipped (no currentColor fallback) |
| `border-top-width` | `<length>` | `medium` | No | ✅ |
| `border-right-width` | `<length>` | `medium` | No | ✅ |
| `border-bottom-width` | `<length>` | `medium` | No | ✅ |
| `border-left-width` | `<length>` | `medium` | No | ✅ |
| `border-top-style` | `<border-style>` | `none` | No | ✅ |
| `border-right-style` | `<border-style>` | `none` | No | ✅ |
| `border-bottom-style` | `<border-style>` | `none` | No | ✅ |
| `border-left-style` | `<border-style>` | `none` | No | ✅ |
| `border-top-color` | `<color>` | `currentColor` | No | ⚠️ Same currentColor limitation |
| `border-right-color` | `<color>` | `currentColor` | No | ⚠️ |
| `border-bottom-color` | `<color>` | `currentColor` | No | ⚠️ |
| `border-left-color` | `<color>` | `currentColor` | No | ⚠️ |
| `border-radius` | `<length> [ / <length> ]?` (1–4 values per side) | `0` | No | ✅ Elliptical syntax with `/`, CSS3 corner-overflow clamping |
| `border-top-left-radius` | `<length> [<length>]?` | `0` | No | ✅ H + V component |
| `border-top-right-radius` | `<length> [<length>]?` | `0` | No | ✅ |
| `border-bottom-right-radius` | `<length> [<length>]?` | `0` | No | ✅ |
| `border-bottom-left-radius` | `<length> [<length>]?` | `0` | No | ✅ |

## Colors & Backgrounds

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `color` | `<color>` | `canvastext` | Yes | ✅ Full color parsing; inherited |
| `accent-color` | `<color>` | UA blue | Yes | ✅ Tints checked checkbox/radio fills, range filled portion |
| `background-color` | `<color>` \| `transparent` | `transparent` | No | ✅ |
| `background-image` | `url(...)` \| `none` | `none` | No | ✅ URL-based image loading |
| `background-repeat` | `repeat` \| `repeat-x` \| `repeat-y` \| `no-repeat` | `repeat` | No | ✅ Tiling with background-clip |
| `background-clip` | `border-box` \| `padding-box` \| `content-box` | `border-box` | No | ✅ Concentric inner-radius reduction for clipping |
| `background-size` | `<string>` (raw) | `auto` | No | ⚠️ Parsed and stored as raw string; limited layout consumption |
| `background-position` | `<string>` (raw) | `0% 0%` | No | ⚠️ Parsed and stored as raw string; limited layout consumption |

## Typography

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `color` | `<color>` | `canvastext` | Yes | ✅ All color formats |
| `font-family` | `<family-name> [, <family-name>]*` | UA-dependent | Yes | ✅ Comma-separated fallback; generic keywords: `sans-serif`, `serif`, `monospace`, `cursive`, `fantasy`, `system-ui`, `ui-*` |
| `font-size` | `<length>` \| `<percentage>` | `medium` | Yes | ✅ All length units |
| `font-weight` | `normal` \| `bold` \| `bolder` \| `lighter` \| `100`–`900` | `normal` | Yes | ✅ Full numeric range |
| `font-style` | `normal` \| `italic` \| `oblique` | `normal` | Yes | ✅ Italic shaping via cosmic-text |
| `line-height` | `normal` \| `<number>` \| `<length>` \| `<percentage>` | `normal` | Yes | ✅ Default ~1.25× font-size; number multiplier, length, percentage |
| `letter-spacing` | `normal` \| `<length>` | `normal` | Yes | ✅ Post-shape per-glyph offset |
| `text-align` | `left` \| `right` \| `center` \| `start` \| `end` | `start` | Yes | ✅ Block-level horizontal alignment; `justify` also supported but not listed in enum |
| `text-transform` | `none` \| `uppercase` \| `lowercase` \| `capitalize` | `none` | Yes | ✅ Applied pre-shape; full uppercase/lowercase/capitalize |
| `text-decoration` | `none` \| `underline` \| `overline` \| `line-through` | `none` | Yes | ✅ Rendered as solid quads at correct vertical offsets |
| `white-space` | `normal` \| `pre` | `normal` | Yes | ✅ Normal: whitespace collapse; Pre: preserve |
| `vertical-align` | `baseline` \| `sub` \| `super` \| `text-top` \| `text-bottom` \| `middle` \| `top` \| `bottom` \| `<length>` | `baseline` | No | ❌ Parsed; limited layout effect |

## Overflow & Visibility

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `overflow` | `visible` \| `hidden` \| `scroll` \| `auto` | `visible` | No | ✅ Sets both axes |
| `overflow-x` | `visible` \| `hidden` \| `scroll` \| `auto` | `visible` | No | ✅ Per-axis clipping |
| `overflow-y` | `visible` \| `hidden` \| `scroll` \| `auto` | `visible` | No | ✅ Per-axis clipping |
| `opacity` | `<number>` (0–1) | `1` | No | ✅ Multiplicative inheritance; baked into color alpha |
| `visibility` | `visible` \| `hidden` | `visible` | Yes | ✅ Inherited |
| `z-index` | `<integer>` \| `auto` | `auto` | No | ❌ Parsed; not consumed in paint ordering |
| `pointer-events` | `auto` \| `none` | `auto` | Yes | ✅ `none` skips hit-testing; children with `auto` remain hittable |
| `user-select` | `auto` \| `none` \| `text` \| `all` | `auto` | Yes | ✅ `none` suppresses selection; `text`/`all` treated as `auto` |
| `cursor` | `<string>` | `auto` | Yes | ⚠️ Parsed; not applied to OS cursor shape |

## Flexbox

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `flex-direction` | `row` \| `row-reverse` \| `column` \| `column-reverse` | `row` | No | ✅ Main axis direction |
| `flex-wrap` | `nowrap` \| `wrap` \| `wrap-reverse` | `nowrap` | No | ✅ Multi-line wrapping |
| `justify-content` | `flex-start` \| `flex-end` \| `center` \| `space-between` \| `space-around` \| `space-evenly` \| `start` \| `end` \| `left` \| `right` | `flex-start` | No | ✅ Full CSS-Flex-1 alignment |
| `align-items` | `flex-start` \| `flex-end` \| `center` \| `stretch` \| `baseline` \| `start` \| `end` | `stretch` | No | ✅ Baseline falls to start |
| `align-content` | `flex-start` \| `flex-end` \| `center` \| `stretch` \| `space-between` \| `space-around` \| `space-evenly` \| `start` \| `end` | `stretch` | No | ✅ Multi-line cross-axis alignment |
| `align-self` | `auto` \| `flex-start` \| `flex-end` \| `center` \| `stretch` \| `baseline` \| `start` \| `end` | `auto` | No | ✅ Per-item override of `align-items` |
| `flex` | `none` \| `<flex-grow> <flex-shrink> <flex-basis>` | `0 1 auto` | No | ✅ Three-value shorthand |
| `flex-grow` | `<number>` | `0` | No | ✅ Two-pass grow distribution |
| `flex-shrink` | `<number>` | `1` | No | ✅ Two-pass shrink distribution; content-based minimum per CSS-Flex-1 §4.5 |
| `flex-basis` | `<length>` \| `auto` | `auto` | No | ✅ Initial main size before grow/shrink |
| `order` | `<integer>` | `0` | No | ✅ Visual reorder without affecting source order (hit-testing preserved) |
| `gap` | `<length>` \| `<percentage>` | `0` | No | ✅ Sets both `row-gap` and `column-gap` |
| `row-gap` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `column-gap` | `<length>` \| `<percentage>` | `0` | No | ✅ |

## CSS Grid

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `grid-template-columns` | `<track-list>` \| `none` | `none` | No | ✅ `px`, `fr`, `auto`, `minmax()`, `repeat()` |
| `grid-template-rows` | `<track-list>` \| `none` | `none` | No | ✅ Same track sizing |
| `grid-auto-columns` | `<track-size>` | `auto` | No | ✅ Implicit column track size |
| `grid-auto-rows` | `<track-size>` | `auto` | No | ✅ Implicit row track size |
| `grid-auto-flow` | `row` \| `column` \| `dense` \| `row dense` \| `column dense` | `row` | No | ✅ Row/column major auto-placement |
| `grid-column-start` | `<integer>` \| `span <integer>` \| `auto` | `auto` | No | ✅ Line-based placement |
| `grid-column-end` | `<integer>` \| `span <integer>` \| `auto` | `auto` | No | ✅ |
| `grid-row-start` | `<integer>` \| `span <integer>` \| `auto` | `auto` | No | ✅ |
| `grid-row-end` | `<integer>` \| `span <integer>` \| `auto` | `auto` | No | ✅ |
| `grid-column` | `<start> / <end>` | `auto / auto` | No | ✅ Shorthand |
| `grid-row` | `<start> / <end>` | `auto / auto` | No | ✅ Shorthand |
| `justify-items` | `start` \| `center` \| `end` \| `stretch` | `stretch` | No | ✅ Inline-axis item alignment |
| `justify-self` | `start` \| `center` \| `end` \| `stretch` | `auto` | No | ✅ Per-item inline-axis override |
| `align-items` | `start` \| `center` \| `end` \| `stretch` | `stretch` | No | ✅ Block-axis item alignment (grid uses same enum) |
| `align-self` | `start` \| `center` \| `end` \| `stretch` | `auto` | No | ✅ Per-item block-axis override |
| `justify-content` | `start` \| `center` \| `end` \| `stretch` \| `space-between` \| `space-around` \| `space-evenly` | `start` | No | ✅ Grid container inline-axis alignment |
| `align-content` | `start` \| `center` \| `end` \| `stretch` \| `space-between` \| `space-around` \| `space-evenly` | `start` | No | ✅ Grid container block-axis alignment |
| `gap` | `<length>` \| `<percentage>` | `0` | No | ✅ Sets both `row-gap` and `column-gap` |
| `row-gap` | `<length>` \| `<percentage>` | `0` | No | ✅ |
| `column-gap` | `<length>` \| `<percentage>` | `0` | No | ✅ |

## Effects & Transforms

| Property | Values | Initial | Inherited | Notes |
|---|---|---|---|---|
| `transform` | `<string>` (raw) | `none` | No | ❌ Parsed as raw string; never consumed |
| `transform-origin` | `<string>` (raw) | `50% 50%` | No | ❌ Parsed as raw string; never consumed |
| `transition` | `<string>` (raw) | `all 0s ease 0s` | No | ❌ Parsed as raw string; never consumed |
| `animation` | `<string>` (raw) | `none` | No | ❌ Parsed as raw string; never consumed |
| `box-shadow` | `<string>` (raw) | `none` | No | ❌ Parsed as raw string; never consumed |

## Form Control Styling

Standard CSS properties that affect form control rendering:

| Property | Applies to | Effect |
|---|---|---|
| `accent-color` | checkbox, radio, range | Tints checked fills and range filled portion. Inherited. |
| `border-color` | checkbox, radio | Unchecked border stroke color |
| `color` | checkbox, radio | Checkmark / radio dot color (auto-contrast from accent by default) |

### `--lui-*` Vendor Properties

Custom properties for styling form control parts that have no standard CSS equivalent. These use the `--lui-` prefix and accept any `<color>` value.

| Property | Default | Effect |
|---|---|---|
| `--lui-track-color` | white | Range slider unfilled track background |
| `--lui-thumb-color` | white | Range slider thumb fill |

```css
input[type="range"] {
    accent-color: #7c3aed;
    --lui-track-color: rgba(124, 58, 237, 0.15);
    --lui-thumb-color: #e0d4fc;
}
```

These work like regular CSS custom properties — they inherit, can use `var()` references, and can be set programmatically via `Node::set_custom_property()`.

## Grid Track Size Units

wgpu-html supports the following track sizing types within `grid-template-columns` and `grid-template-rows`:

| Unit | Description | Example |
|---|---|---|
| `px` / `<length>` | Fixed pixel/unit track | `200px` |
| `fr` | Flex fraction of remaining space | `1fr`, `2fr` |
| `auto` | Intrinsic (content-based) sizing | `auto` |
| `minmax(min, max)` | Clamped track range | `minmax(100px, 1fr)` |
| `repeat(count, track)` | Repeated track pattern | `repeat(3, 1fr)`, `repeat(auto-fill, 200px)` |
