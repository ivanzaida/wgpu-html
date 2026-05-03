---
id: block
title: Block Flow Layout
---

# Block Flow Layout

Block flow is the default formatting context. Every block-level element stacks vertically inside its parent's content box.

## Vertical Stacking

Children are placed one after another in source order. Each child's `margin_rect.y` is positioned at `previous_child.margin_rect.bottom` — margins collapse between adjacent block siblings following CSS 2.1 Section 8.3.1.

## Margin & Padding

All four sides are resolved from CSS shorthand-to-longhand expansion. The parser expands `margin: 10px 20px` into the four longhands, and layout resolves each via the percentage-of-containing-block-width rule.

```css
.box {
    margin: 20px 10px;      /* top/bottom 20, left/right 10 */
    padding: 16px;          /* all sides 16px */
}
```

In `LayoutBox`, margins are encoded in the offset between `margin_rect` and `border_rect`; padding is the offset between `border_rect` and `content_rect`.

## box-sizing

Two values control how `width`/`height` are interpreted:

- **content-box** (default): `width` sets the content area. Border and padding add to the outer size.
- **border-box**: `width` sets the border-box size. Content area shrinks to accommodate border + padding.

```rust
fn effective_content_width(style: &Style, border_box_w: f32, border: Insets, padding: Insets) -> f32 {
    match style.box_sizing.unwrap_or(BoxSizing::ContentBox) {
        BoxSizing::ContentBox => /* style.width defines content width */,
        BoxSizing::BorderBox  => /* style.width defines border-box, subtract border+padding */,
    }
}
```

## Explicit Width/Height with Min/Max Clamping

After resolving `width` / `height`, the result passes through a clamping chain:

```rust
let clamped = resolved
    .min(style.max_height.resolve(container_h))
    .max(style.min_height.resolve(container_h));
```

`min-width` / `max-width` / `min-height` / `max-height` are all resolved relative to the containing block width (percentages) or as pixel values.

## Auto Margin Centering

When a block box has `margin-left: auto` and/or `margin-right: auto` and a definite width, the horizontal margins split the remaining space evenly:

```css
.centered {
    width: 600px;
    margin-left: auto;
    margin-right: auto;  /* centers the box */
}
```

Vertical `auto` margins resolve to `0` in block flow (per CSS spec).

## Border Widths, Colors, Styles

All three border properties are resolved per side and carried into `LayoutBox`:

```rust
pub border: Insets,               // f32 per side (top, right, bottom, left)
pub border_colors: BorderColors,  // Option<Color> per side
pub border_styles: BorderStyles,  // BorderStyle per side
```

Supported styles: `solid`, `dashed`, `dotted`, `none`, `hidden`. `double`/`groove`/`ridge`/`inset`/`outset` render as solid fallback.

## border-radius

Per-corner radii support independent horizontal and vertical components:

```css
.box {
    border-radius: 20px 10px 30px 5px / 10px 20px 5px 15px;
}
```

The CSS-3 corner-overflow clamping algorithm reduces overlapping radii so no corner's radii sum exceeds the edge length on either axis.

## background-clip

`background-clip` drives which rectangle the background fills:

| clip value | background_rect equals |
|---|---|
| `border-box` (default) | `border_rect` |
| `padding-box` | `border_rect` inset by border thickness |
| `content-box` | `content_rect` |

The background corner radii are reduced from the outer `border_radius` by the clip's inset distance — a concentric inner-radius reduction so the painted shape matches the filled area.
