---
title: Flexbox Layout
---

# Flexbox CSS Properties

lui implements **complete CSS Flexible Box Layout Module Level 1** with all major properties. The flex layout engine handles axis direction, wrapping, grow/shrink distribution, alignment, gap, and ordering.

## `display: flex`

Activates flex layout on an element and its direct children:

```css
.container {
  display: flex;
}
```

The container becomes a flex container. Its children become flex items. Only direct children participate — nested descendants are laid out normally within their own formatting contexts.

Flex containers can also use `display: inline-flex` (parsed but currently equivalent to `flex` in layout).

## `flex-direction`

Sets the main axis direction — the primary axis along which flex items are placed:

```css
flex-direction: row;              /* left to right (default) */
flex-direction: row-reverse;      /* right to left */
flex-direction: column;           /* top to bottom */
flex-direction: column-reverse;   /* bottom to top */
```

```
row:             [A] [B] [C] →
row-reverse:  ← [C] [B] [A]
column:          [A]
                 [B] ↓
                 [C]
column-reverse:  [C]
                 [B] ↑
                 [A]
```

The main axis direction also determines which properties are "main" vs "cross":
- **Row flex** — main axis = horizontal, cross axis = vertical
- **Column flex** — main axis = vertical, cross axis = horizontal

## `flex-wrap`

Controls whether flex items wrap to multiple lines when they overflow the container:

```css
flex-wrap: nowrap;        /* single line, items may shrink (default) */
flex-wrap: wrap;          /* wrap to next line */
flex-wrap: wrap-reverse;  /* wrap to previous line (reverse cross order) */
```

```
nowrap:        [A] [B] [C] [D] [E] → (overflow or shrink)
wrap:          [A] [B] [C]
               [D] [E]
wrap-reverse:  [D] [E]
               [A] [B] [C]
```

Multi-line flex containers enable `align-content` for distributing space between lines.

## `justify-content`

Aligns flex items along the **main axis**. Distributing extra space:

```css
justify-content: flex-start;       /* items at start (default) */
justify-content: flex-end;         /* items at end */
justify-content: center;           /* items centered */
justify-content: space-between;    /* first at start, last at end, even gaps */
justify-content: space-around;     /* equal space around each item */
justify-content: space-evenly;     /* equal space between items and edges */
```

```
flex-start:     [A][B][C]..........
flex-end:       ..........[A][B][C]
center:         ....[A][B][C]......
space-between:  [A]......[B]......[C]
space-around:   ..[A]....[B]....[C]..
space-evenly:   ...[A]...[B]...[C]...
```

Also supported: `start`, `end`, `left`, `right` (logical direction variants).

## `align-items`

Aligns flex items along the **cross axis** (perpendicular to main axis). Applied to the container, affects all items:

```css
align-items: stretch;        /* fill cross axis (default) */
align-items: flex-start;     /* align to cross-start */
align-items: flex-end;       /* align to cross-end */
align-items: center;         /* center in cross axis */
align-items: baseline;       /* align text baselines */
```

```
stretch:       ┌────┐┌────┐┌────┐
               │ A  ││ B  ││ C  │
               │    ││    ││    │
               └────┘└────┘└────┘

flex-start:    ┌──┐ ┌──┐ ┌──┐
               │A │ │B │ │C │
               │  │ │  │ │  │
               └──┘ └──┘ └──┘
               ...................

center:        ................
               ┌──┐ ┌──┐ ┌──┐
               │A │ │B │ │C │
               └──┘ └──┘ └──┘
```

> **Note:** `baseline` falls back to `flex-start` alignment. Full baseline alignment with mixed font sizes is not yet implemented.

## `align-self`

Overrides `align-items` for a specific flex item:

```css
.item-special {
  align-self: flex-end;   /* this item aligns to cross-end */
}
.item-centered {
  align-self: center;     /* this item centers in cross axis */
}
```

Values: `auto` (defer to parent's `align-items`), `flex-start`, `flex-end`, `center`, `stretch`, `baseline`.

## `align-content`

Distributes space **between lines** in a multi-line flex container (when `flex-wrap: wrap`):

```css
.container {
  display: flex;
  flex-wrap: wrap;
  height: 400px;
  align-content: space-between;
}
```

Values: `flex-start`, `flex-end`, `center`, `stretch`, `space-between`, `space-around`, `space-evenly`.

Only has an effect when there are multiple lines and extra cross-axis space.

## Flex Sizing: `flex-grow`, `flex-shrink`, `flex-basis`

### Individual Properties

```css
.item {
  flex-grow: 1;         /* grow factor (default: 0) */
  flex-shrink: 1;       /* shrink factor (default: 1) */
  flex-basis: 200px;    /* initial main size (default: auto) */
}
```

### `flex` Shorthand

```css
.item { flex: 1; }              /* 1 1 0% */
.item { flex: 1 0 auto; }       /* grow=1, shrink=0, basis=auto */
.item { flex: 2 1 300px; }      /* grow=2, shrink=1, basis=300px */
.item { flex: none; }           /* 0 0 auto — inflexible */
```

### How Grow/Shrink Works

The engine uses a **two-pass distribution algorithm**:

1. **Grow pass (if items underfill)**: remaining space is distributed proportionally to `flex-grow` values. Each item's contribution = `remaining_space * flex_grow / sum(flex_grow)`.

2. **Shrink pass (if items overflow)**: overflow is distributed proportionally to `flex_shrink * flex_basis` values. Items with `flex-shrink: 0` do not shrink. Items with `overflow: visible` respect content-based minimum size per CSS-Flex-1 §4.5 (they cannot shrink below their content width).

**Two-pass** means the engine first calculates ideal sizes, then re-evaluates if clamping (min/max constraints) caused redistribution needs.

## `order`

Changes the visual order of flex items without affecting source order (important for hit-testing):

```css
.item-first { order: -1; }
.item-last { order: 1; }
```

```html
<div style="display: flex">
  <div style="order: 3">A</div>     <!-- rendered third -->
  <div style="order: 1">B</div>     <!-- rendered first -->
  <div style="order: 2">C</div>     <!-- rendered second -->
</div>
<!-- Visual: [B] [C] [A] -->
<!-- Source (hit-testing): A, B, C -->
```

Default `order` is `0`. Items with equal `order` maintain their source order.

## `gap`, `row-gap`, `column-gap`

Sets spacing between flex items (replaces margin-based spacing, the modern approach):

```css
.container {
  display: flex;
  gap: 16px;                     /* both row and column gap */
  row-gap: 24px;                 /* vertical gap in row flex */
  column-gap: 16px;              /* horizontal gap in row flex */
}
```

```
gap:    [A] <16px> [B] <16px> [C]
        <24px>
        [D] <16px> [E] <16px> [F]
```

Percentage values are relative to the container's content box size.

## Complete Examples

### Horizontal Navigation Bar

```css
.nav {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 24px;
  padding: 12px 24px;
  background-color: #2c3e50;
  color: white;
}

.nav-brand {
  font-size: 1.25rem;
  font-weight: 700;
}

.nav-links {
  display: flex;
  gap: 16px;
}

.nav-links a {
  color: rgba(255, 255, 255, 0.8);
  text-decoration: none;
  padding: 4px 8px;
}

.nav-links a:hover {
  color: white;
}
```

### Card Grid with Flex Wrap

```css
.card-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  justify-content: center;
}

.card {
  flex: 0 1 300px;       /* don't grow, shrink if needed, 300px basis */
  min-width: 250px;       /* don't shrink below 250px */
  padding: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
}
```

### Sticky Footer Layout

```css
.page {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.header {
  padding: 16px;
  background-color: #f8f9fa;
}

.content {
  flex: 1;               /* grow to fill remaining space */
  padding: 24px;
}

.footer {
  padding: 16px;
  background-color: #2c3e50;
  color: white;
}
```

### Centered Content

```css
.centered {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
}
```
