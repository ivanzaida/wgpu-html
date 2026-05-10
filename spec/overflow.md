# lui — Overflow & Clipping Spec

The plan and current state of `overflow: hidden` (and its
near-cousins `clip` / `scroll` / `auto`) clipping descendants to
their containing block. Companion to `roadmap.md` (M11 — clipping
& overflow) and `status.md`.

Status: shipped. Layout carries the resolved `overflow` value
through to paint, the paint pass tracks a clip stack and emits
scissor-tagged ranges, and the renderer recordings call
`set_scissor_rect` once per range. 84 layout tests + 21 paint
tests pass; the demo at
`crates/lui-demo/html/overflow.html` shows the
`visible` / `hidden` / `hidden + border-radius` cases side by
side. The known gaps are spelled out in §6.

---

## 1. Goals

- Honour `overflow: hidden` (and `clip` / `scroll` / `auto`,
  collapsed to "clip" for v1) on any block / flex / grid
  container so descendants can't paint past the container's
  padding edge.
- Per-axis longhands (`overflow-x` / `overflow-y`) don't widen the
  clip when one axis is `Visible` and the other isn't — v1
  collapses both axes to the non-`Visible` value (so
  `overflow-y: hidden; overflow-x: visible;` clips on both axes).
- Nested clips compose by rectangle intersection — the inner
  scissor never escapes the outer.
- Honour rounded `border-radius` on the clipping container — when
  `overflow: hidden` is applied to a rounded box, descendants are
  clipped at the *rounded inner-padding edge*, matching browser
  behaviour. The fragment shader runs an SDF discard against the
  rounded shape on top of the rectangular scissor pre-pass.

## 2. Non-goals (current scope)

- **Scroll bars and scrollable content.** `overflow: scroll` /
  `auto` look identical to `hidden` in the clip sense — the scroll
  offset state lives in `InteractionState::scroll_offsets_y` and
  is applied by the demo's paint path. Per-element scroll-container
  scrollbar drag and wheel scroll are partially wired (M-INTER-4);
  hit-testing inside scroll containers does not yet subtract the
  offset, so clicks inside a scrolled container may mis-target.
- **Per-axis hidden / visible mismatch.** `overflow-x: hidden;
  overflow-y: visible;` collapses to "hidden on both axes" rather
  than emitting a 1D clip rect (which wgpu's scissor primitive
  doesn't support directly anyway).
- **`overflow: clip`** as a separate semantics. The CSS spec
  treats `clip` as "no scroll container, but still clip"; v1
  behaves the same as `hidden` because we don't model scroll
  containers.
- **Stacking-context creation.** `overflow != Visible` doesn't
  promote the box into a new stacking context yet (no z-index
  layering exists).
- **Negative scissor rects.** When the clip would clamp to a
  negative-width / negative-height rect (parent clip + child
  padding-box have an empty intersection), we emit a zero-area
  scissor. Browsers behave the same way visually but skip the
  child entirely; we still issue the (zero-area) draw call.

## 3. Architecture

```
                  ┌──────────────────────────┐
        Style ──→ │ effective_overflow()     │ ─→ LayoutBox::overflow
                  └──────────────────────────┘
                                │
                                ▼
                  ┌──────────────────────────┐
                  │ paint_box_in_clip()      │
                  │  clip_stack: Vec<Frame>  │ ─→ DisplayList { clips: Vec<ClipRange>, … }
                  │   (rect, radii_h, radii_v)
                  └──────────────────────────┘
                                │
                                ▼
                  ┌──────────────────────────┐
                  │ Quad/GlyphPipeline       │ ─→ set_scissor_rect
                  │  ::record (per range):   │     + set_bind_group(dyn offset)
                  │                          │     + draw_indexed
                  └──────────────────────────┘
                                │
                                ▼
                  ┌──────────────────────────┐
                  │ Fragment shader          │
                  │  if (clip_active):       │
                  │    sd_rounded_box discard│
                  └──────────────────────────┘
```

The pipeline is in three stages:

1. **Cascade → layout.** `effective_overflow(style)` collapses
   `overflow` / `overflow-x` / `overflow-y` into one
   `Overflow` value and stamps it onto the resulting
   `LayoutBox::overflow`. The layout box already carries
   `border_rect`, per-side `border` insets, and `border_radius`
   — everything needed to compute the padding-box rect *and*
   the rounded inner-padding edge at paint time.

2. **Paint with clip stack.** `paint_box_in_clip` walks the
   layout tree depth-first. Before recursing into children of a
   non-`Visible` box, it computes the effective rect
   `padding_box(b) ∩ parent_rect` and the inner-padding-edge
   radii via `padding_box_radii(b)` (outer
   `border-radius` shrunk by the per-side border thickness),
   pushes a `ClipFrame { rect, radii_h, radii_v }` onto its
   stack, and asks the `DisplayList` to open a new `ClipRange`
   carrying the rect *and* the corner radii. After children,
   it pops and re-opens a range under the parent's clip.

3. **Renderer two-tier discard.** Each pipeline pre-resolves a
   per-clip-range scissor rect (the rectangular fast-path) plus
   a `Globals` block carrying the rounded-clip data, stored in
   one slot of a dynamic-offset uniform buffer. `record` walks
   the runs, calling `set_bind_group(dyn_offset)`,
   `set_scissor_rect`, and `draw_indexed` per range. The
   fragment shader then runs `sd_rounded_box` against
   `clip_rect` / `clip_radii_h` / `clip_radii_v` and discards
   any pixel past the rounded outer edge — the same SDF that
   shapes the box's own corners. When `clip_active.x` is `0.0`
   (a plain rectangular clip, or no clip), the discard is
   skipped entirely.

The clip rect for `overflow: hidden` is the *padding box* per
CSS-2.2 §11.1.1: the area between the border's inner edge and
the content. It's computed as `border_rect inset by b.border`
in `padding_box()`. The rounded-clip radii are the outer
`border-radius` shrunk by the matching adjacent border
thickness on each corner — same `inset_radii` rule layout uses
for the painted background's rounded path.

`DisplayList` keeps `clips` initialised with one `None`-rect
range so producers that bypass `push_clip` / `pop_clip` (e.g.
the existing paint tests for non-overflow scenarios) still
expose a valid partition. `finalize()` is called at the end of
`paint_tree` / `paint_layout` to drop empty ranges and ensure
the trailing range covers every instance.

## 4. Property coverage

| Property | Supported | Notes |
|---|---|---|
| `overflow: visible` | ✅ | Default. No clip range emitted. |
| `overflow: hidden` | ✅ | Clips children to the padding box. |
| `overflow: clip` | ⚠️ | Treated identically to `hidden`. Real spec semantics (no scroll container, but no programmatic scroll either) deferred. |
| `overflow: scroll` | ⚠️ | Treated as `hidden`. No scroll bars. |
| `overflow: auto` | ⚠️ | Same. |
| `overflow-x` / `overflow-y` | ⚠️ | Either axis non-`Visible` clips both axes. Independent per-axis clipping deferred. |
| Rounded-corner clipping | ✅ | Inner-padding rounded edge derived from `border-radius` insets. SDF discard in the fragment shader. |
| Scroll position / scroll bars | ❌ | Out of scope. |
| Stacking-context creation | ❌ | Z-index layering not modelled. |

## 5. Known gaps (deferred work)

In rough order of usefulness:

1. **Independent per-axis clipping.** `overflow-x: hidden;
   overflow-y: visible;` should clip horizontally only. wgpu's
   scissor only supports rectangular clipping, but we could
   expand the rect to the full viewport on the unclamped axis.
   A small refactor of `effective_overflow` to keep both axes
   plus a per-axis viewport-extent fallback in the renderer.
2. **Scroll container hit-test offsets.** `scroll_offsets_y` now
   exists on `InteractionState` and is read at paint time to
   translate scroll-container descendants. However hit-testing
   inside a scroll container does not yet subtract the offset
   before recursing (M-INTER-4 gap), so clicks misfire on
   scrolled content.
3. **Composing nested rounded clips.** Today only the innermost
   rounded clip's radii reach the shader; an outer rounded
   clip wrapping a non-rounded child still clips correctly via
   intersection of the rectangular scissors, but two nested
   rounded clips with different radii won't both trim the
   inner content. Fixing this needs the shader to sample a
   small stack of clip shapes (or a precomputed mask).
4. **`overflow: clip` distinction.** Per CSS, `clip` means
   "no scroll container, no programmatic scrolling, but still
   clip". With no scroll story, `clip` and `hidden` are
   indistinguishable today.
5. **Stacking-context promotion.** Once z-index lands,
   `overflow != Visible` should create a new stacking context
   so descendants don't escape the clip via positioning.
6. **`clip-path`.** Arbitrary masking (polygon, inset, circle,
   ellipse) reuses the same per-clip-range uniform machinery
   but needs new SDF / mask paths in the shader.

## 6. Tests

- **Layout** (`crates/lui-layout/src/tests.rs`):
  - `overflow_field_propagates_from_style`
  - `overflow_visible_is_default`
  - `overflow_axis_longhand_wins_over_shorthand`
- **Paint** (`crates/lui/src/paint.rs` tests):
  - `overflow_visible_emits_single_clip_range`
  - `overflow_hidden_emits_clip_range_at_padding_box`
  - `overflow_clip_range_only_covers_descendants`
  - `nested_overflow_hidden_intersects_clips`
  - `overflow_hidden_with_border_radius_emits_rounded_clip`
  - `overflow_hidden_padding_box_radii_inset_by_border`

No GPU tests — the renderer-side scissor walk is exercised
visually through the demo and indirectly through the paint
tests' display-list snapshots.

## 7. Demo

`crates/lui-demo/html/overflow.html` shows three side-by-
side panels:

1. `overflow: visible` — the magenta blob extends beyond the
   panel's blue border.
2. `overflow: hidden` — same blob, clipped at the panel's
   padding-box rect.
3. `overflow: hidden` + `border-radius: 24px` — same blob, this
   time cut off by the rounded inner-padding edge thanks to the
   fragment-shader SDF discard.
