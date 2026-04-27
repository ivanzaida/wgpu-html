# wgpu-html — Flex Layout Spec

The plan and current state of `display: flex`, as implemented in
`crates/wgpu-html-layout/src/flex.rs`. Companion to `roadmap.md`
(M9 — flexbox) and `status.md`.

Status: shipped. All 16 dedicated flex unit tests pass plus the 52
pre-existing block-layout tests. The implementation follows
[CSS-Flexbox-1] §9 ("Flex Layout Algorithm") at a level of fidelity
adequate for the demo pages and most everyday UI flex patterns; the
known gaps are spelled out in §6.

[CSS-Flexbox-1]: https://www.w3.org/TR/css-flexbox-1/

---

## 1. Goals

- `display: flex` and `display: inline-flex` containers laying out
  their children along a main + cross axis with the standard CSS
  "freeze loop" for distributing free space.
- Honour the parser-side longhands populated from the `flex` shorthand
  (`flex-grow`, `flex-shrink`, `flex-basis`).
- Cover the alignment grid (`justify-content`, `align-items`,
  `align-content`, `align-self`) and the visual reordering knob
  (`order`).
- Honour `gap` / `row-gap` / `column-gap` per axis.
- Honour `min-*` / `max-*` clamping on items, including during the
  iterative grow / shrink resolution.
- Honour `margin: auto` on flex items: absorbs free space on the
  main axis and consumes leftover line cross space on the cross axis.
- Cleanly separate flex from block layout: the recursive block
  walker accepts `BlockOverrides { width, height }` so the flex
  layer can drive an item to a precomputed main / cross extent
  without mutating its cascaded style.

## 2. Non-goals (current scope)

- No baseline alignment for non-text-bearing items. We don't track
  per-block baselines — only the inline formatting context plumbs
  ascent/descent. `align-items: baseline` and
  `align-self: baseline` degrade to `flex-start`.
- No intrinsic content measurement for `flex-basis: content` /
  `flex-basis: auto` with no main-axis size. The base size resolves
  to `0` in that case (which is what `flex: 1 1 0` already wants —
  the most common pattern).
- No `min-content` / `max-content` / `fit-content` length keywords.
  The `CssLength` enum doesn't have those variants; values using
  them parse as `Raw(_)` and resolve to `auto` (i.e. ignored).
- `display: inline-flex` is currently treated as `display: flex`.
  There's no inline-level wrapping context for flex containers.
- No anonymous flex item wrapping for stray text nodes between
  element children. The parser already collapses
  whitespace-only siblings out of element-only parents, so this
  hasn't bitten yet.
- No `position: absolute` flex children — the engine has no
  out-of-flow positioning at all (M4 explicitly removed it).
- No `aspect-ratio` property.
- No `visibility: collapse` on flex items (treated as `visible`).
- No paged / fragmented flex layout.

## 3. Architecture

```
                              ┌──────────────────────┐
            CascadedTree   →  │ layout::layout_block │  →  LayoutBox
                              └──────────┬───────────┘
                                         │ display == flex / inline-flex?
                                         ▼
                              ┌──────────────────────┐
                              │   flex::layout_flex  │
                              │       _children      │
                              └──────────┬───────────┘
                                         │ for each item:
                                         ▼
                              ┌──────────────────────┐
                              │ layout_block_at_with │
                              │  (BlockOverrides)    │
                              └──────────────────────┘
```

The flex layer is a 9-phase pipeline (`flex.rs`):

1. **Item generation.** Iterate `parent.children`, drop
   `display: none` subtrees, collect each into a `FlexItem`
   carrying its style, frame insets, hypothetical sizes, and
   auto-margin flags. Items are then stable-sorted by
   `(order, source_index)` so rendering order matches CSS
   `order` while the resulting `LayoutBox` children are restored
   to source order at the end (hit-testing relies on that).

2. **Line breaking.** `flex-wrap: nowrap` produces a single line.
   `wrap` / `wrap-reverse` greedy-fill by hypothetical outer
   main + gap, breaking when the next item would push the running
   total past the container's main extent.

3. **Flex factor resolution per line** (CSS-Flex-1 §9.7). The
   iterative freeze loop:
    - Each item starts unfrozen at its hypothetical main size
      (base size clamped by `min-*` / `max-*`).
    - Each iteration recomputes free space (= container main −
      sum of frozen items' outer main − sum of unfrozen items'
      *base* outer main − gaps), distributes proportionally to
      `flex-grow` (growing) or `flex-shrink × base_size`
      (shrinking), then clamps and freezes any item that hit
      a clamp. The total violation direction selects which
      violators get frozen this round (matches the spec's
      "min violators" / "max violators" / "all violators"
      branches).
    - Loop bounded by item count: each iteration either freezes
      an item or distributes all remaining free space.

4. **Per-item cross-axis measurement.** Each item is laid out
   once via `layout_block_at_with(BlockOverrides { width:
   Some(main), height: None })` (or swapped axes for column
   direction). The recursive block layout produces the cross
   extent that emerges from the item's content / explicit cross
   style.

5. **Line cross sizes.** Each line's cross size = max of the
   line's items' margin-box cross sizes. For a single-line
   container with a definite cross size, the line is *clamped*
   to the container's inner cross size (CSS-Flex-1 §9.4 step 15)
   regardless of `align-content` — the line fills the container.

6. **`align-content`** (multi-line only). Distributes the
   container's free cross space across lines:
   `stretch` / `start` / `end` / `center` / `space-between` /
   `space-around` / `space-evenly`. With an indefinite cross
   size or a single line, lines stay at their max-of-items size.

7. **Per-item placement.** For each line:
    - **Auto main margins** absorb free main space first; what's
      left flows into `justify-content`.
    - **`justify-content`** packs items along main:
      `flex-start` / `flex-end` / `center` /
      `space-between` / `space-around` / `space-evenly`,
      plus the `start` / `end` / `left` / `right` aliases.
    - **Cross alignment.** `align-self` (falling through to
      `align-items`) picks `flex-start` / `flex-end` /
      `center` / `stretch`. `stretch` re-lays the item with the
      line's cross extent as the cross dimension via a second
      `BlockOverrides` call. Auto cross margins win against
      `align-self` when present.

8. **Direction reversal.** `row-reverse` / `column-reverse`
   mirrors each item's main position around the container's
   main extent after step 7. `wrap-reverse` already mirrors
   line cross positions before placement.

9. **Source-order restore.** Final children are emitted in
   source order so hit-testing and DOM-style traversal stay
   consistent with the cascaded tree.

The container's used main / cross sizes are returned to
`layout_block`, which feeds them into the parent's
`content_rect` height resolution.

## 4. Boundary with block layout

The recursive block walker exposes:

```rust
pub(crate) struct BlockOverrides {
    pub width: Option<f32>,   // forced content-box width
    pub height: Option<f32>,  // forced content-box height
}

pub(crate) fn layout_block_at_with(
    node: &CascadedNode,
    origin_x: f32,
    origin_y: f32,
    container_w: f32,
    container_h: f32,
    overrides: BlockOverrides,
    ctx: &mut Ctx,
) -> LayoutBox;
```

Override fields are taken at face value (already in content-box
pixels, already min/max-clamped by the flex algorithm). When both
are `None` the call is identical to the original block path.

The block walker also:

- Routes both axes through the shared `clamp_axis` helper — so
  `min-width: 200px` / `max-height: 80px` etc. now affect plain
  blocks as well as flex items.
- Honours horizontal `margin: 0 auto` centering on plain blocks
  with an explicit `width`. Skipped when called via
  `BlockOverrides` (= from the flex layer), so the flex layer's
  own auto-margin pass owns that work and we don't double-count
  free space.

## 5. Property coverage

| Property | Supported | Notes |
|---|---|---|
| `display: flex` | ✅ | Triggers `flex::layout_flex_children`. |
| `display: inline-flex` | ⚠️ | Treated identically to `flex`. |
| `flex-direction` | ✅ | `row`, `row-reverse`, `column`, `column-reverse`. |
| `flex-wrap` | ✅ | `nowrap`, `wrap`, `wrap-reverse`. |
| `flex-flow` | — | Shorthand not parsed; use `flex-direction` + `flex-wrap`. |
| `justify-content` | ✅ | Includes `start`/`end`/`left`/`right` aliases. |
| `align-items` | ✅ | Except `baseline` (degrades to `flex-start`). |
| `align-self` | ✅ | Except `baseline` (degrades to `flex-start`). |
| `align-content` | ✅ | Default `stretch` matches the spec. |
| `order` | ✅ | Stable sort; source order preserved in `LayoutBox`. |
| `gap` | ✅ | Falls through to both axes. |
| `row-gap` | ✅ | Wins over `gap` on the row axis. |
| `column-gap` | ✅ | Wins over `gap` on the column axis. |
| `flex` shorthand | ✅ | All forms expanded by the parser into longhands. |
| `flex-grow` | ✅ | Iterative freeze loop. |
| `flex-shrink` | ✅ | Scaled by `flex-shrink × base_size`. |
| `flex-basis: <length>` | ✅ | Including percentages against a definite container main. |
| `flex-basis: auto` | ✅ | Falls back to the main-axis size property (`width` / `height`). |
| `flex-basis: content` | ❌ | Treated as `auto` with no main size → `0`. |
| `min-width` / `max-width` | ✅ | Clamped during freeze loop and on plain blocks. |
| `min-height` / `max-height` | ✅ | Same. |
| `margin: auto` (main axis) | ✅ | Absorbs free space, split equally across all auto sides. |
| `margin: auto` (cross axis) | ✅ | Consumes leftover line cross space. |
| `box-sizing: border-box` | ✅ | Honoured for `width`, `height`, `flex-basis`, `min-*`, `max-*`. |
| `aspect-ratio` | ❌ | Not modelled. |
| `visibility: collapse` | ❌ | Treated as `visible`. |
| `position: absolute` flex child | ❌ | Engine has no `position` support. |

## 6. Known gaps (deferred work)

In rough priority order if we ever come back to flex:

1. **Baseline alignment.** Needs per-block first-line baseline
   tracking — non-trivial because the inline formatting context
   currently owns ascent/descent state. A clean fix would surface
   `Option<{ascent, descent}>` from `layout_block` for blocks
   whose first descendant is text-bearing.
2. **Intrinsic content basis.** A measurement pre-pass that lays
   each item out at zero main constraint, observes its
   max-content size, and uses that as the base. Bounded extra
   cost: O(items) per flex container.
3. **`min-content` / `max-content` / `fit-content` keywords.**
   Add the variants to `CssLength`, plumb them through `length::resolve`
   (returning `None` for "needs intrinsic measurement" so the caller
   can dispatch), then teach the flex base-size and clamp paths
   to call into the same intrinsic-measurement pre-pass as #2.
4. **`inline-flex` distinction.** Currently identical to `flex`.
   To be correct, the parent's box would need to advertise itself
   as inline-level (so it sits on a line box rather than starting
   a new block) and inherit ascent/descent for inline alignment.
   Mostly a matter of routing through the inline formatting
   context once #1 is done.
5. **Anonymous flex items.** When a flex container has mixed
   element + text children, each contiguous text run should be
   wrapped in an anonymous block-level flex item. The HTML parser
   currently strips whitespace-only text out of element-only
   parents, so this is invisible in practice — but a flex
   container with `<div>Hello<strong>World</strong></div>`-style
   markup would lose the bare "Hello" run today.
6. **`visibility: collapse` on items.** Items with this value
   should behave like `display: none` for line-cross-size
   computation but still keep their line-break-budget contribution.
7. **`aspect-ratio` on items.** Constrains cross extent in terms
   of main extent during phase 4. Useful for image-heavy flex
   layouts; trivial once a few intrinsic-sizing primitives are
   in place.
8. **Forced line breaks.** No equivalent of `<br>` inside a flex
   item triggering a wrap; line breaks are purely greedy on
   outer main width.

## 7. Tests

`crates/wgpu-html-layout/src/tests.rs` ships 16 dedicated flex /
sizing assertions on top of the 52 pre-existing block-layout
tests. New coverage:

- `flex_grow_splits_remaining_main_equally`
- `flex_grow_weighted_by_factor`
- `flex_basis_overrides_width_for_main_size`
- `flex_shorthand_one_value_is_grow_with_zero_basis`
- `flex_shrink_reduces_overflowing_items`
- `flex_min_width_floors_shrunk_item`
- `flex_max_width_caps_grown_item`
- `flex_wrap_breaks_to_new_line`
- `flex_align_self_overrides_align_items`
- `flex_align_content_center_with_two_lines`
- `flex_auto_margin_main_axis_pushes_to_end`
- `flex_order_reorders_visual_layout`
- `flex_row_gap_and_column_gap_independent`
- `min_width_clamps_block_size` (block-level)
- `max_height_clamps_auto_height` (block-level)
- `auto_horizontal_margins_center_block` (block-level)

Plus the parser-side `apply_flex_shorthand` is exercised
through the existing CSS declaration test suite.

## 8. Demo

`crates/wgpu-html-demo/html/flex-grow.html` exercises in order:

1. `flex: 1` / `flex: 1` / `flex: 2` row — proportional split.
2. `max-width` cap, `flex: 1` neutral, `min-width` floor — clamping
   visible alongside an unclamped sibling.
3. Six-item `flex-wrap: wrap` row with `align-content: center`
   placing the two lines vertically centered in the container.
4. `align-items: center` row with one item overriding
   `align-self: flex-start` (top) and another `align-self: flex-end`
   (bottom).
5. `margin-left: auto` pushing the second item to the row's
   trailing edge.
6. Three-item row with the third carrying `order: -1`, showing
   visual reorder against source order.
