---
title: CSS Grid Layout
---

# CSS Grid Properties

lui implements a substantial subset of CSS Grid Layout Module Level 1. The grid system supports explicit track definitions with flexible `fr` units, implicit tracks, named line placement, spanning, auto-placement, and full alignment.

## `display: grid`

Activates grid layout on an element:

```css
.container {
  display: grid;
}
```

All direct children become grid items. Grid items are placed into cells defined by explicit or implicit tracks.

## Grid Template Columns and Rows

### `grid-template-columns` and `grid-template-rows`

Define the explicit grid tracks:

```css
.grid {
  display: grid;
  grid-template-columns: 200px 1fr 1fr;
  grid-template-rows: auto 1fr auto;
}
```

This creates a 3-column, 3-row grid:

```
┌──────────┬───────────┬───────────┐
│  200px   │    1fr    │    1fr    │
├──────────┼───────────┼───────────┤
│  auto    │           │           │
├──────────┼───────────┼───────────┤
│  auto    │           │           │
└──────────┴───────────┴───────────┘
```

#### Track Sizing Units

| Unit | Description | Example |
|---|---|---|
| `<length>` | Fixed size in `px`, `%`, `em`, etc. | `200px`, `50%` |
| `fr` | Fraction of remaining free space | `1fr`, `2fr` |
| `auto` | Intrinsic sizing based on content | `auto` |
| `minmax(min, max)` | Clamped track between min and max | `minmax(100px, 1fr)` |
| `repeat(count, track)` | Repeated track patterns | `repeat(3, 1fr)` |

#### `fr` Unit Distribution

`fr` units distribute remaining space after fixed tracks, gaps, and `auto` tracks are resolved:

```css
grid-template-columns: 200px 1fr 2fr;
/* 200px fixed, then remaining × (1/3) and remaining × (2/3) */
```

#### `minmax()`

Clamp a track between a minimum and maximum size:

```css
grid-template-columns: minmax(200px, 1fr) 1fr 1fr;
/* first column is at least 200px, at most 1fr of remaining */
```

#### `repeat()`

Repeat track definitions:

```css
grid-template-columns: repeat(3, 1fr);                    /* three equal columns */
grid-template-columns: repeat(2, 100px 1fr);              /* 100px 1fr 100px 1fr */
grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); /* responsive */
```

`auto-fill` creates as many tracks as will fit in the container.

## Implicit Tracks: `grid-auto-columns` and `grid-auto-rows`

When grid items are placed outside the explicitly defined grid, implicit tracks are created:

```css
.grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  grid-auto-rows: 150px;         /* implicit rows are 150px tall */
}
```

Items placed in row 5 (when only 3 explicit rows are defined) will create implicit rows sized by `grid-auto-rows`.

## `grid-auto-flow`

Controls the auto-placement algorithm direction:

```css
grid-auto-flow: row;           /* fill rows first (default) */
grid-auto-flow: column;        /* fill columns first */
grid-auto-flow: row dense;     /* row-major, dense packing */
grid-auto-flow: column dense;  /* column-major, dense packing */
```

```
Row-major (grid-auto-flow: row):
  1  2  3
  4  5  6
  7  8  .

Column-major (grid-auto-flow: column):
  1  4  7
  2  5  8
  3  6  .

Dense:    backfills gaps left by
          explicitly placed items
```

> **Note:** `dense` is parsed and accepted but currently lays out identically to non-dense.

## Item Placement

Place grid items on specific lines or spans:

### `grid-column-start` / `grid-column-end` / `grid-row-start` / `grid-row-end`

```css
.item {
  grid-column-start: 1;
  grid-column-end: 3;        /* spans columns 1-2 */
  grid-row-start: 2;
  grid-row-end: span 2;      /* spans 2 rows starting from row 2 */
}
```

```
      col1   col2   col3
row1  ┌──────┬──────┬──────┐
row2  │      │ .item spanning   │
row3  │      │ columns 1-2,     │
row4  │      │ rows 2-4         │
      └──────┴──────┴──────┘
```

### `grid-column` / `grid-row` (Shorthands)

```css
.item {
  grid-column: 1 / 3;       /* start=1, end=3 */
  grid-row: 2 / span 2;     /* start=2, span=2 */
}
```

Placement values:
- **`<integer>`** — 1-based line number (positive counts from start, negative from end)
- **`span <integer>`** — spans N tracks from the auto-placed or opposite edge
- **`auto`** — auto-placement decides the value

Items without explicit placement are handled by the auto-placement algorithm, which places items in order (source order) into the next available cell, respecting `grid-auto-flow` direction.

## Alignment

### Item-Level: `justify-items` / `justify-self` (inline axis), `align-items` / `align-self` (block axis)

```css
.grid {
  display: grid;
  justify-items: center;     /* center items horizontally in their cells */
  align-items: stretch;      /* stretch items vertically (default) */
}

.item-special {
  justify-self: start;        /* override — align left in cell */
  align-self: center;         /* override — center vertically in cell */
}
```

Values: `start`, `center`, `end`, `stretch`.

### Container-Level: `justify-content` / `align-content`

When the grid is smaller than the container, these distribute extra space:

```css
.grid {
  display: grid;
  grid-template-columns: 300px 300px;
  width: 1000px;             /* extra 400px of space */
  justify-content: center;   /* center the grid columns */
}
```

Values: `start`, `center`, `end`, `stretch`, `space-between`, `space-around`, `space-evenly`.

## Gap

```css
.grid {
  display: grid;
  gap: 16px;                 /* both row and column gap */
  row-gap: 24px;             /* vertical gap */
  column-gap: 16px;          /* horizontal gap */
}
```

Gaps only appear between tracks, not at the container edges.

## Complete Examples

### Holy Grail Layout

```css
.page {
  display: grid;
  grid-template-columns: 250px 1fr 250px;
  grid-template-rows: auto 1fr auto;
  min-height: 100vh;
  gap: 0;
}

.header {
  grid-column: 1 / -1;       /* span all columns */
  padding: 16px;
  background-color: #2c3e50;
  color: white;
}

.sidebar-left {
  padding: 24px;
  background-color: #f8f9fa;
}

.sidebar-right {
  padding: 24px;
  background-color: #f8f9fa;
}

.content {
  padding: 24px;
}

.footer {
  grid-column: 1 / -1;       /* span all columns */
  padding: 16px;
  background-color: #2c3e50;
  color: rgba(255, 255, 255, 0.8);
}
```

### Responsive Card Grid

```css
.card-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 24px;
  padding: 24px;
}

.card {
  padding: 20px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background-color: white;
}
```

### Dashboard Layout

```css
.dashboard {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  grid-template-rows: auto;
  gap: 16px;
}

.stats-card {
  padding: 20px;
  background-color: white;
  border-radius: 8px;
}

.stats-card.wide {
  grid-column: span 2;      /* take 2 columns */
}

.chart-area {
  grid-column: 1 / -1;      /* full width */
  grid-row: span 3;          /* 3 rows tall */
  min-height: 300px;
}
```

### Named Line Placement

```css
.timeline {
  display: grid;
  grid-template-columns: 100px 1fr;
  grid-template-rows: auto;
  gap: 8px 24px;
}

.timeline-date {
  grid-column: 1;
  text-align: right;
  font-weight: 700;
  color: #666;
}

.timeline-event {
  grid-column: 2;
  padding-bottom: 24px;
}
```
