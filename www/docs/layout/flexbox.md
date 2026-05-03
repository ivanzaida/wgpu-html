---
id: flexbox
title: Flexbox Layout
---

# Flexbox Layout

The flex formatting context implements CSS Flexbox Level 1 (§9) at a high level of fidelity. `display: flex` (or `inline-flex`) dispatches into `layout_flex_children()`.

## Direction and Wrapping

```css
.container {
    display: flex;
    flex-direction: row;           /* row | row-reverse | column | column-reverse */
    flex-wrap: wrap;               /* nowrap | wrap | wrap-reverse */
}
```

Multi-line wrapping produces a list of *flex lines*. Each line is sized independently; `align-content` distributes leftover cross-axis space among lines.

## Alignment

### Main axis — `justify-content`

```css
justify-content: flex-start | flex-end | center | space-between | space-around | space-evenly;
```

- **space-between**: first item at start, last at end, equal gaps between
- **space-around**: half-gap at edges, full gaps between items
- **space-evenly**: equal spacing everywhere, including edges

### Cross axis — `align-items`, `align-self`

```css
align-items: stretch | flex-start | flex-end | center | baseline;
```

`align-self` on individual items overrides the container's `align-items`.

### Multi-line — `align-content`

```css
align-content: flex-start | flex-end | center | space-between | space-around | stretch;
```

Active only in multi-line flex containers (i.e. `flex-wrap: wrap`).

## Flex Factors (grow / shrink / basis)

```css
.item {
    flex-grow: 1;
    flex-shrink: 0;
    flex-basis: 200px;
}
```

The `flex` shorthand is expanded by the parser into the three longhands. The layout algorithm implements the canonical iterative freeze loop (CSS-Flex-1 §9.7):

1. Compute initial flex base sizes from `flex-basis` or auto content size.
2. Distribute remaining free space proportionally to `flex-grow` values.
3. If overflow: shrink items proportionally to `flex-shrink × flex-base-size`.
4. Clamp each item to its `min-width`/`max-width` — frozen items stop participating.
5. Repeat steps 2-4 until no items change (typically 1-2 iterations).

## Gap

```css
gap: 10px 20px;          /* row-gap column-gap */
row-gap: 10px;
column-gap: 20px;
```

## order

```css
.item-first  { order: -1; }
.item-last   { order: 1;  }
```

Items are stable-sorted by `order` then source index. Layout reorders coordinates but the `children` vector stays in source order — hit-testing ignores visual order, matching CSS behavior.

## Auto Margins on Flex Items

`margin: auto` on a flex item absorbs free space on the main axis. If two items both have `margin-left: auto`, they split the remaining space (centering their adjacent outer edges). On the cross axis, a single `margin: auto` on the perpendicular edge pushes the item against the opposite side — e.g. `margin-top: auto` aligns to the bottom.

## Content-Based Minimum Size

Per CSS-Flex-1 §4.5, `min-width: auto` on a flex item resolves to its content-based minimum size rather than zero. This prevents items from shrinking below their natural content width. The layout engine computes this by measuring the item at its `flex-basis` and clamping shrinkage.

## Complete Example

```css
.card-row {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    padding: 20px;
}

.card {
    flex: 1 1 250px;
    max-width: 350px;
    padding: 24px;
    border: 2px solid #e0e0e0;
    border-radius: 12px;
}

.card.featured {
    flex-grow: 2;
    order: -1;
}
```

This produces a wrapped row of cards. The featured card appears first (order -1), takes twice the share (`flex-grow: 2`), and all cards wrap when the container narrows below `250px × count`.
