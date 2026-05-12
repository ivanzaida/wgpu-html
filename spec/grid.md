# lui — Grid Layout Spec

The plan and current state of `display: grid`, as implemented in
`crates/lui-layout-old/src/grid.rs`. Companion to `roadmap.md`
(M10 — grid) and `status.md`.

Status: shipped at the same level of fidelity as flex — every test
in `crates/lui-layout-old/src/tests.rs` (81 total, 14 dedicated to
grid) passes, the demo page at `crates/lui-demo/html/grid.html`
exercises the holy-grail layout, a fr-based photo gallery, and
row/column auto-flow. The known gaps are spelled out in §6.

---

## 1. Goals

- `display: grid` and `display: inline-grid` containers honouring
  the standard CSS-Grid-Layout-1 sizing + placement algorithm at a
  fidelity adequate for typical web layouts.
- `grid-template-columns` / `grid-template-rows` with `<length>`,
  `<percent>`, `auto`, `<flex>` (`fr`), and `repeat(<integer>,
  <list>)` expansion.
- `grid-auto-rows` / `grid-auto-columns` for implicit tracks.
- Explicit placement (`grid-column`, `grid-row` shorthands and the
  longhands), span shorthand (`grid-column: span N`), and source-
  order auto-placement for unplaced items.
- `grid-auto-flow: row | column` (the `dense` variants accept the
  keyword for cascade fidelity but lay out the same as their
  non-dense counterparts).
- `gap` / `row-gap` / `column-gap` separating tracks.
- Default cell anchoring (`justify-items` / `align-items`,
  defaulting to `stretch`) and per-item override (`justify-self` /
  `align-self`).
- Track-block distribution via `justify-content` / `align-content`
  when the explicit tracks don't fill the container.
- Clean separation from block layout: the grid layer drives item
  layout via `BlockOverrides` so the block walker stays simple.

## 2. Non-goals (current scope)

- No `grid-template-areas` (named areas) and `grid-area` shorthand.
  Named placement is deferred — see §6 for the path forward.
- No `minmax(<min>, <max>)` two-bound clamping. The parser accepts
  the syntax but uses the `<max>` value as the track size (so
  `minmax(100px, 1fr)` lays out as `1fr`).
- No `min-content` / `max-content` / `fit-content()` track sizes.
  These need an intrinsic-size pre-pass on every item, the same
  hole flex has.
- No `repeat(auto-fill, …)` / `repeat(auto-fit, …)`. The parser
  accepts the syntax and produces a single `Auto` track for the
  inner list — track-count resolution depending on container size
  is deferred.
- No `dense` packing. `grid-auto-flow: row dense` is recognized in
  the cascade but the placement loop doesn't backfill earlier holes.
- No named grid lines, no negative line numbers (`grid-column: -1`
  doesn't anchor to the inline-end edge).
- No baseline alignment for non-text items (we don't track
  per-block baselines yet — same gap as flex).
- No subgrid (`display: subgrid` / `grid-template-columns: subgrid`).
- No masonry layout.

## 3. Architecture

```
                              ┌──────────────────────┐
            CascadedTree   →  │ layout::layout_block │  →  LayoutBox
                              └──────────┬───────────┘
                                         │ display == grid / inline-grid?
                                         ▼
                              ┌──────────────────────┐
                              │  grid::layout_grid   │
                              │      _children       │
                              └──────────┬───────────┘
                                         │ for each item:
                                         ▼
                              ┌──────────────────────┐
                              │ layout_block_at_with │
                              │  (BlockOverrides)    │
                              └──────────────────────┘
```

The grid layer is an 8-phase pipeline:

1. **Build raw items.** Iterate `parent.children` (drop
   `display: none`). For each item, snapshot its placement
   directives (`grid-row-start/end`, `grid-column-start/end`),
   per-item alignment overrides (`justify-self`, `align-self`),
   and whether the item carries an explicit inline / block size
   (`width` / `height`).

2. **Explicit grid templates.** Take `grid-template-columns` and
   `grid-template-rows` (already typed `Vec<GridTrackSize>` thanks
   to the parser) verbatim. They start as the explicit grid; they
   can grow during phase 3.

3. **Auto-place items.** First pass: items whose placement is
   fully definite (line-numbered on both axes) pin themselves
   into an occupancy grid. Second pass: every remaining item is
   placed using a row-major (or column-major if
   `grid-auto-flow: column`) cursor — sweep until a free
   `(col_span × row_span)` rectangle is found. Implicit tracks
   beyond the explicit grid are appended on demand and pick up
   `grid-auto-rows` / `grid-auto-columns` for sizing. After this
   step every item knows its `(col_start, col_end, row_start,
   row_end)` half-open ranges.

4. **Resolve column widths.**
    - Pass 1: fixed-length columns resolve via the shared
      `length::resolve` helper.
    - Pass 2: `auto` columns absorb the max-content width of items
      whose primary span includes the column. Items spanning
      multiple auto tracks distribute their measurement
      proportionally to current track sizes (uniformly when all
      are still 0).
    - Pass 3: `<flex>` (`fr`) tracks divide whatever remains of
      `inner_width − fixed_total − gaps` proportionally to their fr
      factors. With no fr tracks, surplus stays at the container
      end (handed to `justify-content` in phase 8).

5. **Lay out items at assigned column-span widths.** For each
   item, the column-span width is the sum of spanned track widths
   plus intervening column-gaps. We call `layout_block_at_with`
   with `BlockOverrides { width: Some(span_w), height: None }` for
   items that will stretch on the inline axis, and with default
   overrides for items that have an explicit `width` or a non-
   stretch `justify-self`. The block-axis size that emerges is
   stored as `measured_h` for phase 6.

6. **Resolve row heights.** Same algorithm as columns, with item
   `measured_h` feeding the row-axis auto tracks. Block-axis fr
   tracks get their share of `inner_height_explicit` when present.

7. **Place items into cells.** Compute each cell's
   `(cell_x, cell_y, cell_w, cell_h)` from cumulative track
   positions plus gaps. Resolve effective `justify-self` and
   `align-self` (falling through to `justify-items` /
   `align-items`, then to `stretch`). For items requiring a
   stretch on either axis with no explicit size, re-lay via
   `BlockOverrides { width: Some(cell_w) | None, height:
   Some(cell_h) | None }`. Otherwise reuse the box from phase 5
   and translate it inside the cell to the resolved start /
   center / end position.

8. **Container-axis distribution.** When the sum of resolved
   tracks is below the container's inner extent on an axis,
   distribute the remainder via `justify-content` (inline) or
   `align-content` (block): `start` / `end` / `center` /
   `space-between` / `space-around` / `space-evenly`. Implemented
   in `track_distribution` and `align_content_distribution` in
   `grid.rs`.

The container's used inline / block size is returned to the block
walker, which folds it back into the parent's `content_rect`.
Source order is preserved on the returned `LayoutBox` children so
hit-testing matches the DOM.

## 4. Boundary with block layout

The grid module talks to the rest of the layout engine through:

- `crate::layout_block_at_with` + `BlockOverrides { width, height
  }` (in `crates/lui-layout-old/src/lib.rs`) — drives recursive
  block layout to a precomputed extent without mutating the
  cascaded style. Same primitive flex uses.
- `crate::translate_box_x_in_place` /
  `crate::translate_box_y_in_place` — recursive subtree shifting
  to position items inside cells.
- `crate::length::resolve` — fixed-length track sizes, gap
  resolution, `min/max-*` clamps on the grid container.

Intrinsic-width measurement for `auto` tracks calls
`layout_block_at_with` once with default overrides at the current
viewport extents and reads the resulting margin-box. This is a
single-pass approximation (no separate `min-content` /
`max-content` measurement pre-pass), which keeps the implementation
small at the cost of less faithful auto-track sizing for items
whose intrinsic min and max differ greatly.

## 5. Property coverage

| Property | Supported | Notes |
|---|---|---|
| `display: grid` / `inline-grid` | ✅ | Both dispatch to the same layout pass. |
| `grid-template-columns` | ✅ | `<length>`, `<percent>`, `auto`, `<flex>`, `repeat(<int>, …)`. |
| `grid-template-rows` | ✅ | Same coverage. |
| `grid-template-areas` | ❌ | Named areas not parsed; `grid-area` shorthand ignored. |
| `grid-auto-rows` | ✅ | Used for implicit-track sizing. |
| `grid-auto-columns` | ✅ | Same. |
| `grid-auto-flow` | ⚠️ | `row` / `column` honoured. `dense` keyword accepted but lays out non-dense. |
| `grid-column` shorthand | ✅ | Line numbers and `span <n>`. |
| `grid-row` shorthand | ✅ | Same. |
| `grid-column-start` / `-end` | ✅ | `auto`, line number, `span <n>`. Negative numbers fall through to `auto`. |
| `grid-row-start` / `-end` | ✅ | Same. |
| `gap` / `row-gap` / `column-gap` | ✅ | Per-axis longhands win over the shorthand. |
| `justify-items` | ✅ | Default cell inline anchoring. |
| `justify-self` | ✅ | Per-item override. |
| `align-items` | ✅ | Default cell block anchoring. Reuses the flex enum. |
| `align-self` | ✅ | Per-item override. Same. |
| `justify-content` | ✅ | Distributes inline-axis free space (start / end / center / space-between / -around / -evenly). |
| `align-content` | ✅ | Same on the block axis. |
| `minmax(<min>, <max>)` | ⚠️ | Parsed, but track is sized as `<max>`. |
| `min-content` / `max-content` / `fit-content()` | ❌ | Track sizing keywords not recognized. |
| `repeat(auto-fill, …)` / `repeat(auto-fit, …)` | ⚠️ | Parses to a single `Auto` track. |
| `subgrid` | ❌ | Not modelled. |
| `place-items` / `place-self` / `place-content` | ❌ | Two-axis shorthands not parsed; use the longhands. |
| Named grid lines | ❌ | No line-name registry. |

## 6. Known gaps (deferred work)

In rough order of value-per-effort:

1. **`grid-template-areas` + `grid-area`.** Parse the multi-string
   syntax (rows separated by quotes, cells inside each row
   separated by whitespace, `.` as anonymous), build a
   name → `(row_start..row_end, col_start..col_end)` map, then
   teach `parse_grid_axis_shorthand` and `grid-area` to consult
   it. Most-requested pattern after the basics; biggest
   ergonomic win.
2. **Intrinsic content sizing.** A measurement pre-pass that
   captures both `min-content` and `max-content` per item,
   replacing the current single-pass approximation. Once that
   exists, the `min-content` / `max-content` / `fit-content()`
   track size keywords slot in as new `GridTrackSize` variants
   pointing at the matching pre-pass output.
3. **`minmax()` two-bound clamping.** Add a `MinMax(min: Box<…>,
   max: Box<…>)` variant to `GridTrackSize` and run a "growth
   limit" pass: each track gets sized to its `<min>` initially
   and then grown toward `<max>` while distributing fr-style
   surplus. The fr distribution path already exists; this is a
   refinement.
4. **`repeat(auto-fill | auto-fit, …)`.** Once `minmax()` is in
   place, count how many copies of the inner list fit between
   the container's min and max widths.
5. **`dense` packing.** Currently `grid-auto-flow: dense`
   produces the same output as the non-dense variant. Switching
   to dense placement just means restarting the cursor at the
   start of the next row / column on every wrap, instead of
   advancing.
6. **Negative line numbers.** `grid-column: -1` should anchor at
   the inline-end edge of the explicit grid. Needs the explicit
   grid extent at parse time, or — easier — a placement
   normalization pass that runs after the explicit-grid sizing
   finishes.
7. **Named grid lines.** Tied to area-name support; both share a
   small registry.
8. **Baseline alignment.** Same hole flex has — needs
   per-block-baseline propagation through the layout output.
9. **Subgrid.** Big architectural change: the inner grid would
   adopt the outer grid's lines for the axes flagged
   `subgrid`. Probably the last grid feature we'd add.
10. **Per-axis `place-*` shorthands.** Trivially trivial as a
    parser-side fan-out; deferred only because the longhands
    cover the typical authoring patterns.

## 7. Tests

`crates/lui-layout-old/src/tests.rs` ships 14 dedicated grid /
sizing assertions on top of the existing 67 layout tests:

- `grid_two_by_two_fixed_columns`
- `grid_fr_distributes_remaining_width`
- `grid_repeat_expands_track_list`
- `grid_explicit_placement_via_grid_column`
- `grid_span_shorthand`
- `grid_auto_flow_row_packs_in_source_order`
- `grid_auto_flow_column_packs_vertically`
- `grid_implicit_rows_use_grid_auto_rows`
- `grid_row_gap_and_column_gap_separate_cells`
- `grid_align_self_end_anchors_item_to_cell_bottom`
- `grid_justify_self_center_horizontally`
- `grid_align_items_stretch_default_fills_cell_vertically`
- `grid_justify_content_center_centers_track_block`

Plus the parser-side track / line / shorthand parsers are
covered through the existing CSS declaration test suite.

## 8. Demo

`crates/lui-demo/html/grid.html` exercises in order:

1. **Holy-grail page layout.** `grid-template-columns: 200px 1fr
   200px;` `grid-template-rows: 60px 200px 40px;` with a header
   spanning all three columns (`grid-column: 1 / 4`), a
   sidebar/main/aside row, and a footer spanning all three
   columns. Demonstrates fixed-column + fr fluidity + explicit
   placement.
2. **Photo gallery.** `grid-template-columns: repeat(4, 1fr);`
   with `gap: 12px;` and 8 cards laid out in two rows via
   row-major auto-flow.
3. **Auto-flow showcase.** A row-major grid with one
   `grid-column: span 2` cell and one `grid-row: span 2` cell
   showing how spans interact with the cursor, then a sibling
   block with `grid-auto-flow: column;` producing column-major
   stacking.
