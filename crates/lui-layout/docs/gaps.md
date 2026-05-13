# lui-layout — Implementation Gaps

Status of every CSS layout feature relative to the spec. Updated 2026-05-13.

## Legend

- **done** — implemented and tested
- **partial** — basic cases work, edge cases missing
- **stub** — BoxKind exists, dispatches to block/no-op
- **missing** — not implemented at all
- **wontfix** — known issue, won't be fixed (e.g. due to spec changes or low impact)

---

## Block Layout

| Feature                                         | Status | Notes                                                                  |
|-------------------------------------------------|--------|------------------------------------------------------------------------|
| Vertical stacking                               | done   |                                                                        |
| Explicit width/height                           | done   |                                                                        |
| min/max-width/height clamping                   | done   |                                                                        |
| Percentage width/height                         | done   | Against containing block                                               |
| `margin: 0 auto` centering                      | done   | Both-auto centers, left-auto pushes right                              |
| `box-sizing: border-box`                        | done   | Width/height subtract padding+border                                   |
| `box-sizing: content-box`                       | done   | Default behavior                                                       |
| `float: left/right`                             | done   | Positioned at edges, multiple floats stack, in-flow content narrows    |
| `clear: left/right/both`                        | done   | Moves cursor below relevant floats                                     |
| Float containment (BFC)                         | done   | overflow:hidden/flex/grid/float elements contain their floats          |
| Margin collapsing — siblings                    | done   | Positive, negative, mixed                                              |
| Margin collapsing — parent/first-child          | done   | Collapses when no border/padding separates them                        |
| Margin collapsing — parent/last-child           | done   | Same for bottom                                                        |
| Margin collapsing — prevented by border/padding | done   | Border or padding on parent prevents collapse                          |
| Margin collapsing — prevented by BFC            | done   | overflow:hidden, flex, grid, inline-block, flow-root                   |
| Margin collapsing — through empty blocks        | done   | Empty block self-collapses, then merges with adjacent siblings         |
| `overflow: hidden`                              | done   | Clips at padding box, BFC root, scroll extent tracked                  |
| `overflow: scroll`                              | done   | Always reserves scrollbar space (default 15px), clip rect, scroll info |
| `overflow: auto`                                | done   | Reserves scrollbar only when content overflows                         |
| `overflow: clip`                                | done   | Like hidden but no scroll container                                    |
| `overflow: visible` (default)                   | done   | Content overflows naturally                                            |
| `scrollbar-width: auto/thin/none`               | done   | Controls scrollbar space reservation (15px/8px/0px)                    |
| Scroll position API                             | done   | `set_scroll()`, `scroll_by()`, clamped to valid range                  |
| `scroll_to_reveal()`                            | done   | Computes scroll offset to make a target rect visible                   |
| `child_visible_rect()`                          | done   | Applies scroll offset to get viewport-space position                   |
| Hit testing (scroll-aware)                      | done   | `hit_test()` accounts for scroll offsets and clip rects                |

## Flexbox

| Feature                                       | Status | Notes                                                                                            |
|-----------------------------------------------|--------|--------------------------------------------------------------------------------------------------|
| flex-direction (all 4)                        | done   | row, column, row-reverse, column-reverse                                                         |
| flex-wrap (all 3)                             | done   | nowrap, wrap, wrap-reverse                                                                       |
| flex-grow                                     | done   |                                                                                                  |
| flex-shrink (weighted by base size)           | done   | CSS-Flex-1 §9.7 freeze loop                                                                      |
| flex-basis (px, auto fallback)                | done   |                                                                                                  |
| flex-basis: auto with content measurement     | done   | Measures max-content width via text shaping + recursive child measurement                        |
| justify-content (all 6)                       | done   | flex-start/end, center, space-between/around/evenly                                              |
| align-items (all 4)                           | done   | flex-start/end, center, stretch                                                                  |
| align-self override                           | done   |                                                                                                  |
| align-content (multi-line)                    | done   | All 7 values                                                                                     |
| order (stable sort)                           | done   |                                                                                                  |
| gap / row-gap / column-gap                    | done   | Shorthand expanded by cascade                                                                    |
| margin: auto on main axis                     | done   | Absorbs free space before justify-content                                                        |
| margin: auto on cross axis                    | done   | Per-item distribution                                                                            |
| min/max clamping on items                     | done   | Iterative freeze loop                                                                            |
| Stretch re-layout                             | done   | Items without explicit cross size re-laid at line cross size                                     |
| Percentage cross size on indefinite container | done   | Correctly prevents stretch                                                                       |
| Nested flex containers                        | done   |                                                                                                  |
| Intrinsic sizing (max-content/min-content)    | done   | Max-content and min-content for both axes; auto minimum size per CSS-Flex-1 §4.5                 |
| Baseline alignment                            | done   | First baseline via text ascent; falls back to flex-start for column direction                    |
| `visibility: collapse` on flex items          | done   | Zero main-axis, preserves cross-axis contribution                                                |
| Absolutely-positioned flex children           | done   | Filtered from flex items, laid out against flex container's padding box                          |
| `flex` shorthand keywords                     | done   | `flex:1` → `1 1 0`, `flex:auto` → `1 1 auto`, `flex:none` → `0 0 auto`                           |
| `flex-basis: content/max-content/min-content` | done   | Keywords dispatch to intrinsic measurement functions                                             |
| Percentage margins/padding on flex items      | done   | Resolved against containing block width via `resolve_margin_against` / `resolve_padding_against` |

## Grid

| Feature                                   | Status  | Notes                                                                       |
|-------------------------------------------|---------|-----------------------------------------------------------------------------|
| grid-template-columns/rows (px, fr, auto) | done    |                                                                             |
| Percentage tracks                         | done    |                                                                             |
| `repeat(N, tracks)`                       | done    | Including mixed patterns like `repeat(2, 100px 1fr)`                        |
| `minmax(min, max)`                        | done    | px min, fr/px/auto max                                                      |
| grid-column/row-start/end (line numbers)  | done    |                                                                             |
| `span N` on column/row end                | done    | Multi-cell items                                                            |
| gap / row-gap / column-gap                | done    |                                                                             |
| Auto-placement (row-major)                | done    |                                                                             |
| grid-auto-rows (px)                       | done    |                                                                             |
| grid-auto-columns                         | done    | Used for implicit columns in column-flow                                    |
| grid-auto-flow: column                    | done    | Items placed column-first, implicit columns grow with auto-columns size     |
| grid-auto-flow: dense                     | done    | Resets cursor to (0,0) for each auto-placed item                            |
| align-items / justify-items               | done    | center, end, start                                                          |
| align-self / justify-self per item        | done    | Overrides container alignment                                               |
| `auto-fill` in repeat()                   | done    | Computes track count from available space / track size                      |
| `auto-fit` in repeat()                    | done    | Same as auto-fill; empty tracks remain at 0 (items don't stretch into them) |
| Named grid lines                          | done    | `[name] 1fr [name]` parsed and resolved in placement                        |
| `grid-template-areas`                     | done    | Parses area strings, generates implicit named lines, area-based placement   |
| `grid-template` shorthand                 | done    | `rows / columns` syntax; areas form not yet supported                       |
| `grid-area` shorthand                     | done    | 1-4 values with `/` splitting; area name lookup                             |
| Subgrid (CSS Grid Level 2)                | wontfix | Child grid inherits parent track sizing                                     |
| Masonry layout (CSS Grid Level 3)         | wontfix | Experimental spec                                                           |

## Positioned Layout

| Feature                                          | Status | Notes                                                                         |
|--------------------------------------------------|--------|-------------------------------------------------------------------------------|
| `position: static`                               | done   | Default, stays in flow                                                        |
| `position: relative` with top/left               | done   | Offset from normal position                                                   |
| `position: relative` with right/bottom           | done   | Negative offset                                                               |
| `position: absolute` with top/left               | done   | Positioned against containing block                                           |
| `position: absolute` with right/bottom           | done   |                                                                               |
| `position: absolute` left+right auto-width       | done   | Width computed from insets                                                    |
| `position: absolute` no insets (static position) | done   | Falls back to where element would be in flow                                  |
| `position: fixed`                                | done   | Positioned against viewport                                                   |
| `position: sticky`                               | done   | Laid out in-flow; StickyInsets stored for renderer to apply during scroll     |
| `z-index` stacking order                         | done   | z_index stored on LayoutBox; renderer uses it for paint ordering              |
| Containing block from transforms                 | done   | `is_positioned` checks for transform; abs children use it as containing block |
| Percentage insets                                | done   | Against containing block dimensions                                           |

## Inline Layout

| Feature                           | Status | Notes                                                                       |
|-----------------------------------|--------|-----------------------------------------------------------------------------|
| Text shaping (system fonts)       | done   | Via lui-glyph / cosmic-text                                                 |
| Line breaking (word wrap)         | done   | `break_into_lines` from lui-glyph                                           |
| Inline container flow             | done   | Children flow horizontally with wrapping                                    |
| Inline-block                      | done   | Block inside, inline outside                                                |
| `text-align: left`                | done   | Default behavior                                                            |
| `text-align: center/right`        | done   | Shifts anonymous inline blocks within block containers                      |
| `text-align: justify`             | done   | Stretches anonymous block to container width; renderer handles word spacing |
| `vertical-align`                  | done   | top/middle/bottom alignment within inline line boxes                        |
| `white-space: normal`             | done   | Default wrapping                                                            |
| `white-space: nowrap`             | done   | Prevents line breaking in text nodes and inline containers                  |
| `white-space: pre`                | done   | Prevents line breaking (pre-wrap/pre-line need newline handling)            |
| `white-space: pre-wrap/pre-line`  | done   | Splits on newlines; pre-wrap preserves spaces, pre-line collapses           |
| `word-break: break-all`           | done   | Inserts break opportunities between characters in long words                |
| `overflow-wrap: break-word`       | done   | Same as break-all — breaks long words that overflow                         |
| `text-overflow: ellipsis`         | done   | Sets text_overflow_ellipsis flag on overflowing children                    |
| `text-indent`                     | done   | Offsets first line cursor_x in inline containers                            |
| `letter-spacing` / `word-spacing` | done   | Added to TextStyle; width adjusted after shaping                            |
| `text-decoration`                 | done   | text_decoration field stored on LayoutBox for renderer                      |
| `text-transform`                  | done   | uppercase/lowercase/capitalize applied before shaping                       |
| `direction: rtl`                  | done   | Mirrors inline child x positions in inline containers                       |
| `writing-mode`                    | done   | Stored on LayoutBox for renderer; full axis swap not yet implemented        |
| `line-height` in inline context   | done   | CSS line-height sets minimum line box height in inline containers           |
| Inline-block shrink-to-fit        | done   | Shrinks content.width to max child right edge when no explicit width        |

## Box Model

| Feature                     | Status | Notes                                                                           |
|-----------------------------|--------|---------------------------------------------------------------------------------|
| Content-box sizing          | done   | Default                                                                         |
| `box-sizing: border-box`    | done   | Width/height include padding+border                                             |
| Padding (px)                | done   |                                                                                 |
| Border width (px)           | done   |                                                                                 |
| Margin (px, auto detection) | done   |                                                                                 |
| Margin auto centering       | done   | Both-auto, left-auto, right-auto                                                |
| Negative margins            | done   |                                                                                 |
| Percentage padding/margin   | done   | Resolved against containing block width (both horizontal and vertical per spec) |
|

## Units

| Feature            | Status | Notes                                                                                   |
|--------------------|--------|-----------------------------------------------------------------------------------------|
| px                 | done   |                                                                                         |
| %                  | done   | Against containing block                                                                |
| em                 | done   | Cascade resolves font-size first, then uses element's font-size for other em properties |
| rem                | done   | Against root 16px                                                                       |
| pt, cm, mm, in, pc | done   | Converted to px in resolve_length                                                       |
| vw, vh, vmin, vmax | done   | Cascade resolves against MediaContext viewport; layout fallback in resolve_length_full  |
| ch, ex             | done   | Resolved by cascade; ch/ex ≈ 0.5em (standard approximation)                             |
| fr (grid/flex)     | done   |                                                                                         |
| calc()             | done   | Resolved by cascade via lui-resolve; layout sees plain px values                        |

## Display Types

| Feature                               | Status | Notes                                                   |
|---------------------------------------|--------|---------------------------------------------------------|
| `display: block`                      | done   |                                                         |
| `display: inline`                     | done   |                                                         |
| `display: inline-block`               | done   |                                                         |
| `display: flex`                       | done   |                                                         |
| `display: grid`                       | done   |                                                         |
| `display: none`                       | done   | Skipped in box generation                               |
| `display: table/table-row/table-cell` | done   | Auto/fixed layout, border-spacing/collapse, colspan/rowspan |
| `display: inline-flex`                | done   | Inline outside, flex inside; shrink-to-fit              |
| `display: inline-grid`                | done   | Inline outside, grid inside; shrink-to-fit              |
| `display: contents`                   | done   | Element's box skipped; children promoted to parent      |
| `display: list-item`                  | done   | list_marker stored on LayoutBox; dispatches to block    |
| `display: flow-root`                  | done   | BFC root; laid out as block; prevents margin collapsing |

## Visual / Paint Properties (not layout but noted)

| Feature                | Status  | Notes                                                  |
|------------------------|---------|--------------------------------------------------------|
| `opacity`              | missing | Doesn't affect layout but affects stacking context     |
| `visibility: hidden`   | missing | Should reserve space but not paint                     |
| `visibility: collapse` | missing | Table/flex-specific behavior                           |
| `transform`            | missing | Doesn't affect layout but establishes containing block |
| `clip-path`            | missing | Paint only                                             |
| `filter`               | missing | Paint only                                             |
| `mix-blend-mode`       | missing | Paint only                                             |

## Table Layout

| Feature                              | Status  | Notes                                                              |
|--------------------------------------|---------|--------------------------------------------------------------------|
| `<table>` / `display: table`         | done    | Full table layout with auto/fixed column sizing                    |
| `<tr>` / `display: table-row`        | done    | Row height = tallest cell; rows stack vertically                   |
| `<td>` / `display: table-cell`       | done    | Cells positioned by column widths, colspan/rowspan aware           |
| `<th>`                               | done    | Same as td (styling from UA stylesheet)                            |
| `<thead>` / `<tbody>` / `<tfoot>`    | done    | TableRowGroup kind; rows inside laid out as part of table grid     |
| `<caption>`                          | done    | Laid out above (default) or below (`caption-side: bottom`) table   |
| `<colgroup>` / `<col>`               | missing |                                                                    |
| Auto table layout algorithm          | done    | Explicit cell widths respected; remaining space distributed equally |
| Fixed table layout                   | done    | First row cell widths determine columns; remaining split equally   |
| `border-collapse` / `border-spacing` | done    | Separate model with h/v spacing; collapse zeroes spacing           |
| Cell spanning (colspan/rowspan)      | done    | Occupancy grid; spanned cell widths/heights computed correctly     |

## Architecture Limitations

| Issue                                            | Impact     | Notes                                                              |
|--------------------------------------------------|------------|--------------------------------------------------------------------|
| No stacking context tree                         | Medium     | z-index has no effect; paint order = DOM order                     |
| ~~No intrinsic sizing pass~~                     | ~~Medium~~ | Fixed — max-content and min-content measurement implemented        |
| Percentage margins/padding silently zero         | High       | `sides.rs` only resolves px/auto; percentage values dropped        |
| ~~Abs-positioned children in flex not filtered~~ | ~~High~~   | Fixed — filtered and laid out against flex container's padding box |
| em units resolved at cascade with default 16px   | Medium     | Nested font-size inheritance doesn't propagate to em resolution    |
| ~~Single layout pass~~                            | ~~Low~~    | Table uses multi-pass (height estimation → row positioning → cell layout) |
| No anonymous table wrappers                      | Low        | Table elements outside `<table>` not wrapped automatically         |

---

## Priority Recommendations

### Tier 1 — Affects nearly every page

1. ~~`box-sizing: border-box`~~ done
2. ~~`flex` shorthand keywords~~ done
3. ~~Abs-positioned flex children~~ done
4. ~~Percentage margins/padding~~ done
5. ~~text-align: center/right~~ done
6. ~~`overflow: hidden/scroll/auto`~~ done
7. ~~`margin: 0 auto` block centering~~ done
8. `white-space: nowrap`
9. Viewport units (vw, vh)

### Tier 2 — Common layouts

10. ~~`float` / `clear`~~ done
11. `vertical-align` (inline)
12. `z-index` stacking
13. ~~Intrinsic sizing for flex-basis:auto~~ done
14. ~~Parent-child margin collapsing~~ done
15. `display: inline-flex` / `inline-grid`
16. `flex-basis: content/max-content/min-content` keywords

### Tier 3 — Advanced

13. `position: sticky`
14. `calc()`
15. Grid `auto-fill` / `auto-fit`
16. Grid named areas
17. ~~Table layout~~ done
18. `text-overflow: ellipsis`
19. `direction: rtl`

### Tier 4 — Rare / Future

20. Subgrid
21. `display: contents`
22. Masonry
23. Container queries
24. `writing-mode`
