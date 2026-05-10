---
title: Layout Engine
---

# Layout Engine

How the cascaded style tree becomes positioned boxes with pixel coordinates.

## Entry Points

**File:** `crates/lui-layout/src/lib.rs`

| Function | Line | Purpose |
|---|---|---|
| `layout_with_text()` | 992 | Full layout with text shaping via `TextContext` |
| `layout_with_text_profiled()` | 1006 | Layout with optional timing/profiling |
| `layout_block()` | 1186 | Core recursive block layout |

Layout receives a `&CascadedTree` (already style-resolved) and produces a `LayoutBox` tree. All coordinates are absolute pixels.

## LayoutBox

The output node type (line 431):

```rust
pub struct LayoutBox {
    // CSS box model rects (absolute pixels)
    pub margin_rect: Rect,
    pub border_rect: Rect,
    pub content_rect: Rect,

    // Visual properties (resolved from Style)
    pub background: Option<[f32; 4]>,
    pub background_rect: Rect,
    pub border: [f32; 4],           // top/right/bottom/left widths
    pub border_colors: [Option<[f32; 4]>; 4],
    pub border_styles: [BorderStyle; 4],
    pub border_radius: BorderRadius,

    // Text
    pub text_run: Option<ShapedRun>,
    pub text_color: Option<[f32; 4]>,
    pub text_decorations: TextDecorations,

    // Images
    pub image: Option<ImageData>,
    pub background_image: Option<BackgroundImagePaint>,

    // Interaction
    pub opacity: f32,
    pub z_index: Option<i32>,
    pub cursor: Option<ArcStr>,
    pub pointer_events: PointerEvents,
    pub user_select: UserSelect,

    // Overflow
    pub overflow: Option<OverflowAxes>,
    pub resize: Resize,

    pub children: Vec<LayoutBox>,
    pub kind: BoxKind,              // Block | Text
}
```

## Block Layout Algorithm

`layout_block()` (line 1186) is the core recursive function:

1. **`display: none`** (line 1202) -- early exit, return empty box
2. **Text leaf** (line 1210) -- shape text via `TextContext`, return text box
3. **Replaced element** (line 1219) -- `<img>` intrinsic sizing from decoded dimensions or HTML attrs
4. **Box model resolution** (lines 1246-1330):
   - Resolve margin/border/padding via `resolve_insets_margin()` (line 3841) and `resolve_insets_padding()` (line 3850)
   - Compute width with min/max clamping via `clamp_axis()` (line 3890)
   - Handle `box-sizing: border-box` vs `content-box`
   - Auto-margin centering (line 1316)
5. **Children layout dispatch** (lines 1390-1464):
   - `display: flex` -> `layout_flex_children()` (flex.rs)
   - `display: grid` -> `layout_grid_children()` (grid.rs)
   - All children inline -> `layout_inline_block_children()` (IFC)
   - Otherwise -> recursive `layout_block()` per child
   - Out-of-flow children -> `layout_out_of_flow_block()` (line 1898)
6. **Height finalization** (lines 1467-1495) -- explicit height or content-based, clamped by min/max

### Box Model

- `resolve_insets_margin()` (line 3841) -- resolve each margin side from `CssLength`
- `resolve_insets_padding()` (line 3850) -- resolve each padding side
- `clamp_axis()` (line 3890) -- apply min/max bounds; `min` wins ties per CSS-Sizing-3 section 5.2
- `compute_background_box()` (line 3787) -- resolve `background-clip` to border-box/padding-box/content-box rect

### Length Resolution

**File:** `crates/lui-layout/src/length.rs` line 14

`resolve()` converts `CssLength` to pixels:

| Unit | Resolution |
|---|---|
| `px` | Direct |
| `%` | Fraction of containing block |
| `em` | Multiple of current font-size |
| `rem` | Multiple of root font-size |
| `vw`/`vh`/`vmin`/`vmax` | Fraction of viewport |
| `calc()`/`min()`/`max()`/`clamp()` | Recursive AST evaluation |
| `auto` | Returns `None` (caller decides) |

## Inline Formatting Context (IFC)

**File:** `crates/lui-layout/src/lib.rs`

| Function | Line | Purpose |
|---|---|---|
| `layout_inline_block_children()` | 2977 | Wrapper for block with all-inline children |
| `layout_inline_paragraph()` | 3424 | Main inline paragraph shaping |

Paragraph layout path:

1. **Span collection** (line 3438) -- flatten inline subtree into spans via `collect_paragraph_spans()`
2. **cosmic-text shaping** (line 3476) -- `ctx.text.ctx.shape_paragraph()` for rich text layout
3. **Per-line text-align** (line 3490) -- compute offset for left/right/center/justify
4. **Inline element backgrounds** (lines 3500-3522) -- anonymous block per wrapped span segment
5. **Text decorations** (lines 3524-3557) -- underline/overline/line-through bars
6. **Glyph positioning** (line 3561) -- apply line-alignment offsets to positioned glyphs

## Flexbox

**File:** `crates/lui-layout/src/flex.rs`

**Entry:** `layout_flex_children()` (line 45)

Implements CSS-Flexbox-1 section 9:

1. **Build flex items** (lines 100-116) -- filter `display: none`, collect as `FlexItem` with hypothetical sizes, sort by `order`
2. **Line collection** (lines 118-150) -- `nowrap` = single line; `wrap`/`wrap-reverse` = greedy-fill with breaking; gaps applied between items
3. **Main axis sizing** -- `flex-grow`/`flex-shrink`/`flex-basis` with clamping
4. **Cross-axis alignment** -- `align-items`/`align-self` per item
5. **Justification** -- `justify-content` distribution (flex-start/center/space-between/space-around/space-evenly)
6. **Multi-line** -- `align-content` for cross-axis line distribution
7. **Auto margins** -- absorb free space on main and cross axes

## Grid

**File:** `crates/lui-layout/src/grid.rs`

**Entry:** `layout_grid_children()` (line 46)

Implements CSS-Grid-Layout-1 sections 6-11:

1. **Build grid items** (lines 80-95) -- collect children, skip `display: none`
2. **Explicit templates** (line 98+) -- parse `grid-template-columns`/`rows` with `px`/`fr`/`auto`/`repeat()`
3. **Implicit grid** -- default tracks via `grid-auto-columns`/`grid-auto-rows`
4. **Placement** -- explicit via `grid-column-start/end`, `grid-row-start/end`; auto-placement per `grid-auto-flow`
5. **Track sizing** -- fr resolution, content-based sizing
6. **Alignment** -- `justify-items`/`align-items` (cell), `justify-content`/`align-content` (track distribution), per-item overrides via `justify-self`/`align-self`
7. **Gaps** -- `row-gap`/`column-gap` between tracks

## Positioned Layout

**File:** `crates/lui-layout/src/lib.rs`

| Function | Line | Purpose |
|---|---|---|
| `layout_out_of_flow_block()` | 1898 | Absolute/fixed positioning |
| `positioned_overrides()` | 1942 | Compute width/height from left+right / top+bottom |
| `apply_relative_position()` | 2014 | Offset box in-place for `position: relative` |
| `shrink_to_fit_content_width()` | 1986 | Auto width for positioned blocks |

Positioning rules:
- `position: absolute` -- containing block is nearest positioned ancestor
- `position: fixed` -- containing block is viewport
- `position: relative` -- offset from static position, does not affect siblings
- Both `left` and `right` set -> width auto-computed
- Both `top` and `bottom` set -> height auto-computed

## Intrinsic Sizing

- **Replaced elements** (line 1219) -- HTML `width`/`height` attributes preferred, then decoded image dimensions
- **Shrink-to-fit** (`shrink_to_fit_content_width()`, line 1986) -- for positioned blocks: if all children inline, layout inline and use resulting width; otherwise max of block children's margin boxes
- **Text measurement** (`measure_text_leaf()`, line 2472) -- calls text shaping, returns `(width, height)` in pixels

## Background Images

Pre-computed during layout into `BackgroundImagePaint` (line 554):

```rust
pub struct BackgroundImagePaint {
    pub image_id: u64,
    pub data: Arc<Vec<u8>>,   // decoded RGBA8
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Rect>,     // pre-positioned in physical pixels
}
```

Tile positions computed by `compute_bg_tiles()` (line 181) considering `background-size`, `background-position`, and `background-repeat`. Tiles are filtered to those overlapping `background_rect`.

## Overflow & Scrolling

- `effective_overflow()` (line 2029) -- resolve overflow shorthand + axis interaction
- `OverflowAxes` struct (line 515) -- per-axis clipping flags + scrollbar config
- Border radius clamping: `clamp_corner_radii()` (line 3915) -- normalize when sum exceeds edge length

## Incremental Layout

**File:** `crates/lui-layout/src/lib.rs`

| Function | Purpose |
|---|---|
| `layout_incremental()` | Entry point — updates a cached LayoutBox in-place for dirty subtrees |
| `relayout_children()` | Core algorithm — walks children, skips clean subtrees, shifts siblings |
| `translate_box_y_in_place()` | Shifts a subtree's Y coordinates without re-layout |
| `patch_form_controls()` | Patches form control visual state without any geometry work |

When the DOM changes, the pipeline tracks which node paths are dirty (`Tree.dirty_paths`). Instead of rebuilding the entire LayoutBox tree, `layout_incremental` walks the cached tree and CascadedTree in parallel:

1. **Clean nodes** (not dirty, not ancestor of dirty): skipped entirely — cached LayoutBox reused as-is
2. **Dirty nodes**: re-laid-out from scratch via `layout_block()`. Height delta tracked.
3. **Ancestors of dirty nodes**: recurse into children but don't re-layout the ancestor itself
4. **Siblings after a height change**: shifted via `translate_box_y_in_place(dy)` — no re-layout, just Y-coordinate adjustment
5. **Auto-height parents**: grow/shrink their rects by the accumulated height delta
6. **Explicit-height parents**: absorb the delta (siblings outside don't shift)

### Flex/Grid Fallback

Flex and grid containers have cross-item dependencies (free-space redistribution, track sizing). When any child of a flex/grid container is dirty, the entire container is re-laid-out from scratch. Clean siblings *of* the flex/grid container still benefit from skipping.

### Pipeline Integration

`classify_frame()` returns `PipelineAction::LayoutOnly` when `tree.generation` changed. The pipeline tries incremental layout first when `dirty_paths` is non-empty, falling back to full layout otherwise.

For form control visual state (checkbox checked, range value), a separate `PatchFormControls` action patches `FormControlInfo` fields in-place with zero geometry work.

### What Flows Where

| Direction | Data | Implication |
|---|---|---|
| Down | Container width, overrides | Parent constrains child — clean children are valid if parent width unchanged |
| Up | Computed height, baseline | Child height change propagates to parent and shifts younger siblings |
| Sideways | Flex free-space, grid track sizes | One item change affects all items on same line/track |

## Profiling

Optional profiler at `layout_profile` module (line 1045). Counts block/flex/grid/inline calls and text shaping calls. Dumped to stderr when enabled. Zero overhead when disabled.
