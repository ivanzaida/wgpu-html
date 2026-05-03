---
id: grid
title: CSS Grid Layout
---

# CSS Grid Layout

The grid formatting context implements CSS Grid Layout Level 1 (§6–§11). `display: grid` dispatches into `layout_grid_children()`.

## Track Definitions

### `grid-template-columns` / `grid-template-rows`

Track lists support the full CSS Grid sizing vocabulary:

```css
.grid {
    display: grid;
    grid-template-columns: 200px 1fr 2fr auto minmax(100px, 1fr);
    grid-template-rows: 80px auto;
}
```

| Track type | Meaning |
|---|---|
| `<length>` (px, em, rem, %) | Fixed-size track |
| `fr` (flex fraction) | Distributes remaining space proportionally |
| `auto` | Sizes to the largest content in the track |
| `minmax(min, max)` | Clamped track with lower and upper bounds |
| `repeat(n, pattern)` | Repeats a track pattern *n* times |

The `repeat()` function is expanded inline during parsing. `fr` distribution happens after all fixed and auto tracks are resolved, consuming the remaining container space.

### Track Resolution Order

1. Resolve all definite `<length>` tracks.
2. Size `auto` tracks to their maximum content contributions.
3. Distribute remaining space to all `fr` tracks proportionally.
4. Apply `minmax()` clamping — tracks with `minmax()` that land outside bounds are frozen at the bound, and space reflows to unfrozen tracks.

## Implicit Tracks

```css
.grid {
    grid-auto-rows: 100px;
    grid-auto-columns: 150px;
}
```

When items overflow the explicit grid, implicit tracks are created at the `grid-auto-rows`/`grid-auto-columns` size.

## Grid Auto Flow

```css
grid-auto-flow: row | column | dense;
```

- **row** (default): Auto-placed items fill row-by-row, left to right.
- **column**: Fill column-by-column, top to bottom.
- **dense**: Backfill smaller items into earlier gaps (accepted, lays out same as non-dense for now).

## Item Placement

Items are placed by line number or can span multiple tracks:

```css
.item-a {
    grid-column-start: 1;
    grid-column-end: span 2;
    grid-row-start: 2;
    grid-row-end: 4;
}

/* Shorthand */
.item-b {
    grid-column: 1 / 3;
    grid-row: 2 / span 2;
}
```

Line numbers are 1-indexed (CSS convention). Items with no explicit placement are auto-placed in source order.

## Auto-Placement Algorithm

For `grid-auto-flow: row`:

1. Iterate items in source order.
2. For each item without explicit placement, scan the grid by increasing row, then column, looking for the first unfilled cell.
3. If the item spans multiple tracks, check all cells in the span are empty.
4. Place the item at the first valid position.

For `grid-auto-flow: column`, the scan order is column-major (column increasing, then row).

## Alignment

```css
.grid {
    justify-items: stretch;    /* per-cell inline axis */
    align-items: stretch;      /* per-cell block axis */
    justify-content: center;   /* track block distribution */
    align-content: center;     /* track block distribution */
}

.item {
    justify-self: start;
    align-self: end;
}
```

`justify-items` / `align-items` position items within their grid cells. `justify-content` / `align-content` distribute the grid tracks within the container when the container is larger than the track sum.

## Gap

```css
gap: 16px 24px;    /* row-gap column-gap */
row-gap: 16px;
column-gap: 24px;
```

## Complete Example

```css
.page-layout {
    display: grid;
    grid-template-columns: 250px 1fr 1fr;
    grid-template-rows: auto 1fr auto;
    gap: 16px;
    min-height: 100vh;
}

.header  { grid-column: 1 / -1; }
.sidebar { grid-row: 2; }
.content { grid-column: 2 / -1; grid-row: 2; }
.footer  { grid-column: 1 / -1; }
```

This creates a classic holy-grail layout: header spanning all columns, sidebar + two-column content area, footer at the bottom.
