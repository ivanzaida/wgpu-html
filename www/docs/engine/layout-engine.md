---
sidebar_position: 5
---

# Layout Engine

The layout engine converts the cascaded style tree into positioned boxes with pixel coordinates. It lives in `crates/lui-layout/src/lib.rs`.

## Entry Point

```rust
pub fn layout_with_text(
    tree: &CascadedTree,
    viewport_width: f32,
) -> LayoutBox
```

Receives a `&CascadedTree` (already style-resolved) and produces a `LayoutBox` tree. All coordinates are absolute pixels from the viewport origin (top-left).

## LayoutBox

The output node carries the CSS box model geometry plus all resolved visual properties:

| Field | Description |
|---|---|
| `margin_rect` | Outer edge including margin |
| `border_rect` | Edge of border area (margin→border) |
| `content_rect` | Inner content area (border→padding→content) |
| `background` | Resolved background color (RGBA) |
| `background_rect` | Paint rect for background (respects `background-clip`) |
| `border` | Per-side border widths |
| `border_colors` | Per-side border colors |
| `border_styles` | Per-side border styles (solid/dashed/dotted) |
| `border_radius` | Corner radii (supports elliptical) |
| `text_run` | Shaped text glyphs + positions |
| `text_color` | Resolved text color |
| `image` | Decoded image data for `<img>` elements |
| `background_image` | Background image tile positions |
| `children` | Child layout boxes |
| `kind` | `Block` or `Text` |

## Block Layout

`layout_block()` is the core recursive function:

1. **`display: none`** → early exit, empty box
2. **Text leaf** → shape text via `TextContext`, return text box
3. **Replaced element** → `<img>` intrinsic sizing
4. **Box model** → resolve margin/border/padding, apply `box-sizing`, min/max clamping
5. **Children dispatch**:
   - `display: flex` → `layout_flex_children()`
   - `display: grid` → `layout_grid_children()`
   - All children inline → inline formatting context (IFC)
   - Otherwise → recursive `layout_block()` per child
6. **Height finalization** → explicit height or content-based, clamped by min/max

## Flexbox

Implements CSS Flexbox Level 1 (`crates/lui-layout/src/flex.rs`):

- Flex direction (row/column, normal/reverse)
- Flex wrap (nowrap/wrap/wrap-reverse)
- `flex-grow` / `flex-shrink` / `flex-basis` with iterative freeze loop
- `justify-content` (all values)
- `align-items` / `align-self`
- `align-content` for multi-line containers
- `gap` / `row-gap` / `column-gap`
- `order` sorting
- `margin: auto` absorption

## CSS Grid

Implements CSS Grid Layout Level 1 (`crates/lui-layout/src/grid.rs`):

- `grid-template-columns` / `grid-template-rows` with `px`, `fr`, `auto`, `repeat()`
- `grid-auto-rows` / `grid-auto-columns` for implicit tracks
- Explicit placement via `grid-column-start/end`, `grid-row-start/end`
- Auto-placement with `grid-auto-flow: row | column`
- `justify-items` / `align-items` per cell
- `justify-content` / `align-content` for track distribution
- `gap` between tracks

## Inline Formatting Context (IFC)

Detected automatically when a block's children are all inline-level. Uses cosmic-text paragraph shaping for line breaking, word wrap, and `text-align`.

## Positioning

| Position | Behavior |
|---|---|
| `static` | Normal flow |
| `relative` | Offset from static position without affecting siblings |
| `absolute` | Removed from flow; containing block = nearest positioned ancestor |
| `fixed` | Removed from flow; containing block = viewport |
| `sticky` | Degraded to `relative` (sticky scroll not yet implemented) |

## Incremental Layout

When DOM mutations occur, `layout_incremental()` updates a cached LayoutBox tree in-place rather than rebuilding from scratch. Clean subtrees are reused. Siblings after a height change are shifted by `dy` without re-layout. Flex/grid containers with any dirty child are fully re-laid-out due to cross-item dependencies.

## Length Resolution

CSS lengths (`px`, `%`, `em`, `rem`, `vw`, `vh`, `vmin`, `vmax`, `calc()`, `min()`, `max()`, `clamp()`) are resolved to pixels via `crates/lui-layout/src/length.rs`. `auto` returns `None` and the caller decides the behavior.
