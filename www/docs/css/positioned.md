---
title: Positioned Layout
---

# Positioned Layout Properties

wgpu-html implements CSS positioned layout with `static`, `relative`, `absolute`, and `fixed` positioning. The layout engine resolves containing blocks, applies offsets, and handles shrink-to-fit sizing for absolutely positioned elements.

## `position`

Controls the positioning scheme:

```css
position: static;      /* normal flow (default) */
position: relative;    /* offset from normal flow position */
position: absolute;    /* removed from flow, positioned relative to containing block */
position: fixed;       /* removed from flow, positioned relative to viewport */
```

All four values are fully supported. `sticky` is parsed but degrades to `relative` — scroll-pinning is not yet implemented.

## Offsets: `top`, `right`, `bottom`, `left`

Offset properties work with positioned elements:

```css
.overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
}
```

Accepted values: `<length>`, `<percentage>`, `auto`.

- `auto` means no offset from that edge (default)
- Percentages resolve against the containing block's size

## Positioning Schemes

### `position: static` (Default)

The element participates in normal flow layout. The `top`, `right`, `bottom`, `left` properties have no effect.

```css
.static-box {
  position: static;   /* default, no offset applied */
  width: 300px;
}
```

### `position: relative`

The element stays in normal flow (occupies its original space), then is visually offset by `top`/`right`/`bottom`/`left`:

```css
.shifted {
  position: relative;
  top: 10px;           /* moves 10px down from normal position */
  left: -5px;          /* moves 5px left from normal position */
}
```

```
Normal flow position:      [box]
Relative (top:10px; left:20px):
                           ┌──────────────────┐
                           │  original space   │
                           │  (still occupied) │
                           └──────────────────┘
                                  ┌───┐
                                  │box│ (rendered here)
                                  └───┘
```

The `apply_relative_position()` function in layout computes the offset and adds it to the element's position after normal flow layout.

### `position: absolute`

The element is removed from normal flow (does not occupy space) and positioned relative to its **containing block**:

```css
.container {
  position: relative;    /* creates containing block */
  width: 400px;
  height: 300px;
}

.absolute-badge {
  position: absolute;
  top: 0;
  right: 0;
  width: 80px;
  height: 24px;
}
```

```
Container (position: relative)
┌──────────────────────────────┐
│                              │
│                    ┌───────┐ │  ← badge positioned
│                    │ badge │ │    at top-right corner
│                    └───────┘ │    of container
│                              │
└──────────────────────────────┘
```

### `position: fixed`

Like `absolute`, but the **containing block is the viewport**:

```css
.fixed-header {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 60px;
  background-color: rgba(0, 0, 0, 0.9);
}
```

The element stays at the same position regardless of scrolling.

## Containing Block Resolution

The containing block is determined by:

1. For `fixed` — the viewport (always)
2. For `absolute` — the nearest positioned ancestor (`position: relative | absolute | fixed | sticky`), or the viewport if none
3. For `relative` — the element's own normal flow position

The layout function `layout_out_of_flow_block()` in `wgpu-html-layout` walks up the tree to find the containing block, then resolves inset offsets and sizes.

## Shrink-to-Fit Sizing for Absolute Elements

When an absolutely positioned element has:
- No explicit `width` (or `width: auto`)
- Both `left` and `right` are `auto`

The element sizes to its **shrink-to-fit** width — the minimum of:
- Available space (containing block width)
- Preferred (max-content) width
- The CSS shrink-to-fit formula: `min(max-content, max(min-content, available))`

This prevents absolutely positioned elements from expanding to fill the containing block when not desired.

## Right/Bottom Anchoring

Setting opposing offsets requires the element to stretch or adjust:

```css
.fill-width {
  position: absolute;
  left: 0;
  right: 0;          /* element stretches to fill between edges */
  height: 50px;
}

.anchored-right {
  position: absolute;
  right: 20px;       /* anchored to right edge */
  bottom: 20px;      /* anchored to bottom edge */
  /* width/height auto → shrink-to-fit */
}
```

When both left and right (or top and bottom) are non-auto, the element's width/height is computed as `containing_block_size - left - right`, unless an explicit width/height is set.

## `z-index`

```css
.overlay {
  z-index: 10;
}
```

The `z-index` property is parsed and stored on the `Style` struct as an `Option<i32>`. However, it is **not consumed** by the layout or paint passes, meaning **all elements are painted in tree DFS order** regardless of `z-index`. This is a known gap.

## Sticky Positioning

```css
.sticky-nav {
  position: sticky;
  top: 0;
}
```

`sticky` is parsed but degrades to `relative`. Scroll-pinning (where the element "sticks" to the viewport edge while its containing block is in view) is not yet implemented.

## Code Examples

### Modal Overlay

```css
.modal-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
}

.modal-dialog {
  position: relative;
  width: 500px;
  max-width: 90vw;
  background-color: white;
  border-radius: 8px;
  padding: 24px;
}
```

### Tooltip

```css
.tooltip-container {
  position: relative;
}

.tooltip {
  position: absolute;
  bottom: 100%;
  left: 50%;
  transform: translateX(-50%);   /* parsed but not consumed */
  padding: 4px 8px;
  background-color: #333;
  color: white;
  border-radius: 4px;
  font-size: 12px;
  white-space: pre;
}
```

### Badge

```css
.badge-wrapper {
  position: relative;
  display: inline-block;
}

.badge {
  position: absolute;
  top: -8px;
  right: -8px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background-color: #e74c3c;
  color: white;
  font-size: 12px;
  display: flex;
  justify-content: center;
  align-items: center;
}
```

### Fixed Bottom Bar

```css
.bottom-bar {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  height: 48px;
  background-color: #2c3e50;
  color: white;
  display: flex;
  align-items: center;
  padding: 0 16px;
}
```

### Anchor + Content Layout

```css
.page {
  position: relative;
  min-height: 100vh;
}

.anchor {
  position: absolute;
  top: 120px;
  left: 24px;
  width: 80px;
  height: 40px;
}

/* Anchor stays at (24px, 120px) relative to .page */
```
