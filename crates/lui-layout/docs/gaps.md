# lui-layout — Implementation Gaps

Status of every CSS layout feature relative to the spec. Updated 2026-05-13.

## Legend

- **done** — implemented and tested
- **partial** — basic cases work, edge cases missing
- **stub** — BoxKind exists, dispatches to block/no-op
- **missing** — not implemented at all

---

## Block Layout

| Feature | Status | Notes |
|---------|--------|-------|
| Vertical stacking | done | |
| Explicit width/height | done | |
| min/max-width/height clamping | done | |
| Percentage width/height | done | Against containing block |
| `margin: 0 auto` centering | done | Both-auto centers, left-auto pushes right |
| `box-sizing: border-box` | done | Width/height subtract padding+border |
| `box-sizing: content-box` | done | Default behavior |
| `float: left/right` | missing | CSS2 float layout not implemented |
| `clear: left/right/both` | missing | Depends on float |
| Margin collapsing — siblings | done | Positive, negative, mixed |
| Margin collapsing — parent/first-child | done | Collapses when no border/padding separates them |
| Margin collapsing — parent/last-child | done | Same for bottom |
| Margin collapsing — prevented by border/padding | done | Border or padding on parent prevents collapse |
| Margin collapsing — prevented by BFC | done | overflow:hidden, flex, grid, inline-block, flow-root |
| Margin collapsing — through empty blocks | missing | Empty block's top+bottom margins collapse with adjacent |
| `overflow: hidden/scroll/auto` | partial | Prevents margin collapse (BFC), but no visual clipping or scroll containers |
| `overflow: visible` (default) | done | Content overflows naturally |

## Flexbox

| Feature | Status | Notes |
|---------|--------|-------|
| flex-direction (all 4) | done | row, column, row-reverse, column-reverse |
| flex-wrap (all 3) | done | nowrap, wrap, wrap-reverse |
| flex-grow | done | |
| flex-shrink (weighted by base size) | done | CSS-Flex-1 §9.7 freeze loop |
| flex-basis (px, auto fallback) | done | |
| flex-basis: auto with content measurement | partial | Falls back to 0 for non-text elements instead of measuring intrinsic content |
| justify-content (all 6) | done | flex-start/end, center, space-between/around/evenly |
| align-items (all 4) | done | flex-start/end, center, stretch |
| align-self override | done | |
| align-content (multi-line) | done | All 7 values |
| order (stable sort) | done | |
| gap / row-gap / column-gap | done | Shorthand expanded by cascade |
| margin: auto on main axis | done | Absorbs free space before justify-content |
| margin: auto on cross axis | done | Per-item distribution |
| min/max clamping on items | done | Iterative freeze loop |
| Stretch re-layout | done | Items without explicit cross size re-laid at line cross size |
| Percentage cross size on indefinite container | done | Correctly prevents stretch |
| Nested flex containers | done | |
| Intrinsic sizing (max-content/min-content) | missing | flex-basis:auto doesn't recursively measure content for non-text elements |
| Baseline alignment | missing | Falls back to flex-start |
| `visibility: collapse` on flex items | missing | |

## Grid

| Feature | Status | Notes |
|---------|--------|-------|
| grid-template-columns/rows (px, fr, auto) | done | |
| Percentage tracks | done | |
| `repeat(N, tracks)` | done | Including mixed patterns like `repeat(2, 100px 1fr)` |
| `minmax(min, max)` | done | px min, fr/px/auto max |
| grid-column/row-start/end (line numbers) | done | |
| `span N` on column/row end | done | Multi-cell items |
| gap / row-gap / column-gap | done | |
| Auto-placement (row-major) | done | |
| grid-auto-rows (px) | done | |
| grid-auto-columns | partial | Parsed but not fully used in column-flow |
| grid-auto-flow: column | partial | Code path exists, untested |
| grid-auto-flow: dense | partial | Code path exists, untested |
| align-items / justify-items | done | center, end, start |
| align-self / justify-self per item | done | Overrides container alignment |
| `auto-fill` in repeat() | missing | Responsive grids — compute track count from available space |
| `auto-fit` in repeat() | missing | Like auto-fill but collapses empty tracks |
| Named grid lines | missing | `[name] 1fr [name]` syntax |
| `grid-template-areas` | missing | Named area placement |
| `grid-template` shorthand | missing | Combined rows/columns/areas |
| `grid-area` shorthand | missing | Combined row/column start/end |
| Subgrid (CSS Grid Level 2) | missing | Child grid inherits parent track sizing |
| Masonry layout (CSS Grid Level 3) | missing | Experimental spec |

## Positioned Layout

| Feature | Status | Notes |
|---------|--------|-------|
| `position: static` | done | Default, stays in flow |
| `position: relative` with top/left | done | Offset from normal position |
| `position: relative` with right/bottom | done | Negative offset |
| `position: absolute` with top/left | done | Positioned against containing block |
| `position: absolute` with right/bottom | done | |
| `position: absolute` left+right auto-width | done | Width computed from insets |
| `position: absolute` no insets (static position) | done | Falls back to where element would be in flow |
| `position: fixed` | done | Positioned against viewport |
| `position: sticky` | missing | Scrolls with content until threshold, then sticks |
| `z-index` stacking order | missing | No z-index sorting; paint order = DOM order |
| Containing block from transforms | missing | `transform` on an ancestor should establish containing block for absolute descendants |
| Percentage insets | done | Against containing block dimensions |

## Inline Layout

| Feature | Status | Notes |
|---------|--------|-------|
| Text shaping (system fonts) | done | Via lui-glyph / cosmic-text |
| Line breaking (word wrap) | done | `break_into_lines` from lui-glyph |
| Inline container flow | done | Children flow horizontally with wrapping |
| Inline-block | done | Block inside, inline outside |
| `text-align: left` | done | Default behavior |
| `text-align: center/right/justify` | missing | No horizontal alignment of line content |
| `vertical-align` | missing | No baseline/top/middle/bottom alignment |
| `white-space: normal` | done | Default wrapping |
| `white-space: nowrap` | missing | |
| `white-space: pre/pre-wrap/pre-line` | missing | |
| `word-break: break-all` | missing | |
| `overflow-wrap: break-word` | missing | |
| `text-overflow: ellipsis` | missing | |
| `text-indent` | missing | |
| `letter-spacing` / `word-spacing` | missing | Properties exist on ComputedStyle but not used in layout |
| `text-decoration` | missing | Not a layout concern (paint only) but affects ink bounds |
| `text-transform` | missing | uppercase/lowercase — affects shaping |
| `direction: rtl` | missing | |
| `writing-mode` | missing | |
| `line-height` in inline context | partial | Used for text shaping; not for inline box vertical alignment |
| Inline-block shrink-to-fit | missing | Should shrink to content width when no explicit width set |

## Box Model

| Feature | Status | Notes |
|---------|--------|-------|
| Content-box sizing | done | Default |
| `box-sizing: border-box` | done | Width/height include padding+border |
| Padding (px) | done | |
| Border width (px) | done | |
| Margin (px, auto detection) | done | |
| Margin auto centering | done | Both-auto, left-auto, right-auto |
| Negative margins | done | |
| Percentage padding/margin | partial | Percentage resolves against containing width; not tested for vertical |
| `outline` | missing | Not a layout concern (doesn't affect box size) |

## Units

| Feature | Status | Notes |
|---------|--------|-------|
| px | done | |
| % | done | Against containing block |
| em | partial | Resolved at cascade time with default 16px; doesn't inherit computed font-size properly |
| rem | done | Against root 16px |
| pt, cm, mm, in, pc | done | Converted to px in resolve_length |
| vw, vh, vmin, vmax | missing | Viewport units not resolved |
| ch, ex | missing | Font-metric units |
| fr (grid/flex) | done | |
| calc() | missing | Parsed as Function but not evaluated during layout |

## Display Types

| Feature | Status | Notes |
|---------|--------|-------|
| `display: block` | done | |
| `display: inline` | done | |
| `display: inline-block` | done | |
| `display: flex` | done | |
| `display: grid` | done | |
| `display: none` | done | Skipped in box generation |
| `display: table/table-row/table-cell` | stub | BoxKind exists, dispatched as no-op |
| `display: inline-flex` | missing | Should be inline outside, flex inside |
| `display: inline-grid` | missing | Should be inline outside, grid inside |
| `display: contents` | missing | Element's box not generated; children promoted to parent |
| `display: list-item` | stub | BoxKind::ListItem exists, no marker generation |
| `display: flow-root` | partial | Recognized as BFC root for margin collapsing; no dedicated dispatch |

## Visual / Paint Properties (not layout but noted)

| Feature | Status | Notes |
|---------|--------|-------|
| `opacity` | missing | Doesn't affect layout but affects stacking context |
| `visibility: hidden` | missing | Should reserve space but not paint |
| `visibility: collapse` | missing | Table/flex-specific behavior |
| `transform` | missing | Doesn't affect layout but establishes containing block |
| `clip-path` | missing | Paint only |
| `filter` | missing | Paint only |
| `mix-blend-mode` | missing | Paint only |

## Table Layout

| Feature | Status | Notes |
|---------|--------|-------|
| `<table>` / `display: table` | stub | BoxKind exists, no layout logic |
| `<tr>` / `display: table-row` | stub | |
| `<td>` / `display: table-cell` | stub | |
| `<th>` | stub | |
| `<thead>` / `<tbody>` / `<tfoot>` | missing | |
| `<caption>` | missing | |
| `<colgroup>` / `<col>` | missing | |
| Auto table layout algorithm | missing | |
| Fixed table layout | missing | |
| `border-collapse` / `border-spacing` | missing | |
| Cell spanning (colspan/rowspan) | missing | |

## Architecture Limitations

| Issue | Impact | Notes |
|-------|--------|-------|
| No stacking context tree | Medium | z-index has no effect; paint order = DOM order |
| No scrollable overflow | High | No scroll containers; content clips to viewport |
| No intrinsic sizing pass | Medium | Flex/grid items can't measure content for auto sizing |
| em units resolved at cascade with default 16px | Medium | Nested font-size inheritance doesn't propagate to em resolution |
| Single layout pass | Low | Some features (auto table, intrinsic flex) need two passes |
| No anonymous table wrappers | Low | Table elements outside `<table>` not wrapped automatically |

---

## Priority Recommendations

### Tier 1 — Affects nearly every page
1. ~~`box-sizing: border-box`~~ done
2. `text-align: center/right`
3. ~~`overflow: hidden`~~ partial (BFC, no visual clipping)
4. ~~`margin: 0 auto` block centering~~ done
5. `white-space: nowrap`
6. Viewport units (vw, vh)

### Tier 2 — Common layouts
7. `float` / `clear`
8. `vertical-align` (inline)
9. `z-index` stacking
10. Intrinsic sizing for flex-basis:auto
11. ~~Parent-child margin collapsing~~ done
12. `display: inline-flex` / `inline-grid`

### Tier 3 — Advanced
13. `position: sticky`
14. `calc()`
15. Grid `auto-fill` / `auto-fit`
16. Grid named areas
17. Table layout
18. `text-overflow: ellipsis`
19. `direction: rtl`

### Tier 4 — Rare / Future
20. Subgrid
21. `display: contents`
22. Masonry
23. Container queries
24. `writing-mode`
