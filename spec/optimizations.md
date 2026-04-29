# Optimization Spec

Performance optimization roadmap for wgpu-html. Each item is
categorised by pipeline stage, estimated impact, and implementation
complexity.

The pipeline per frame is:

```
sync_fonts → cascade → layout (+ text shaping) → paint → GPU render
```

Profiling is available via F9 in the demo (prints per-second summary
of cascade / layout / paint / render ms).

---

## O1 — Incremental layout (dirty-bit tracking)

**Stage:** layout
**Impact:** high — layout is the most expensive stage; today the
entire tree is re-walked even when nothing changed.
**Complexity:** high

`layout_with_text` (layout/lib.rs:1988) recurses from the root on
every frame. Each block allocates child Vecs, measures text, resolves
flex, etc.

### Plan

1. Assign each `CascadedNode` a generation counter (bumped when the
   node's style or text content changes).
2. In `layout_block` / `layout_inline_block_children`, compare the
   current generation against a cached `LayoutBox` from the previous
   frame.
3. If the generation matches AND the available width / height haven't
   changed, return the cached box immediately.
4. Propagate a "dirty" flag upward: if any descendant is dirty, the
   ancestor must re-layout (its content height may change).
5. Store the cache on `TextContext` (or a new `LayoutCache` struct)
   keyed by node identity + available dimensions.

### Dirty triggers

- `Tree::root` replaced or mutated (set_custom_property, DOM
  mutation).
- Viewport resize (invalidates root + any %-sized descendants).
- Scroll offset change (only affects paint, not layout — skip).
- Font registration change (invalidates all text leaves).

---

## O2 — Text measurement cache

**Stage:** layout (text shaping)
**Impact:** high — `shape_and_pack` and `shape_paragraph` are the
most expensive functions inside layout; called once per text node
per frame.
**Complexity:** medium

`shape_text_run` (layout/lib.rs:2951) normalises whitespace,
applies text-transform, picks a font, and calls
`TextContext::shape_and_pack`. None of these results are cached
between frames.

### Plan

1. Define a cache key:
   ```
   TextCacheKey {
       text_hash: u64,        // hash of the raw text content
       font_handle: FontHandle,
       size_px: OrderedFloat<f32>,
       line_height_px: OrderedFloat<f32>,
       weight: u16,
       style_axis: FontStyleAxis,
       max_width: Option<OrderedFloat<f32>>,
   }
   ```
2. Store `HashMap<TextCacheKey, ShapedRun>` on `TextContext`.
3. On cache hit, clone the `ShapedRun` (glyphs are small structs;
   this is cheaper than re-shaping). Or use `Arc<ShapedRun>` to
   avoid cloning entirely.
4. Invalidate on font sync (generation counter on `FontDb`).
5. Apply the same cache to `shape_paragraph` with a similar key
   that hashes all span texts + attributes.

### Expected gain

Text shaping is typically 30-60 % of layout time. Caching
eliminates it for static documents and reduces it to a hash
lookup for interactive documents where only a few text nodes
change per frame.

---

## O3 — Display list caching

**Stage:** paint
**Impact:** medium-high — display list rebuild is O(n) in the
layout tree every frame.
**Complexity:** medium

`paint_tree_with_text` (paint.rs:39) walks the entire `LayoutBox`
tree, emitting quads and glyphs into a fresh `DisplayList`. For
static content this work is redundant.

### Plan

1. Assign a generation counter to the `LayoutBox` tree (incremented
   when layout produces a new root).
2. Cache the last `DisplayList` alongside its generation.
3. On the next frame, if the generation matches AND scroll offsets /
   selection state haven't changed, reuse the cached list.
4. For interactive changes (scroll, selection, hover), implement a
   partial repaint path that patches the cached list instead of
   rebuilding from scratch.

---

## O4 — Cascade inheritance with COW / Arc

**Stage:** cascade
**Impact:** medium — reduces allocation pressure, especially for
deep DOM trees with many inherited properties.
**Complexity:** medium

`inherit_into` (style/lib.rs) unconditionally clones inherited
property values (`font-family`, `color`, `line-height`, etc.) from
parent to every child. Most children don't override these, so the
cloned value is identical to the parent's.

### Plan

1. Wrap commonly inherited `String` fields (`font_family`,
   `color` string form, custom properties) in `Arc<str>` or
   `Cow<'parent, str>`.
2. Inheritance becomes `Arc::clone()` (pointer bump) instead of
   `String::clone()` (heap alloc + memcpy).
3. Only allocate a new `Arc` when a child's own rules override
   the inherited value.

---

## O5 — Hit-test spatial index

**Stage:** event handling
**Impact:** medium — `pointer_move` is high-frequency; today it
walks the full layout tree.
**Complexity:** medium

`collect_hit_path` (layout/lib.rs:1922) does a depth-first reverse-
child-order walk of the entire layout tree on every pointer event.
For large DOMs this is O(n).

### Plan

1. At layout time, build a flat list of `(Rect, path)` entries
   sorted by Z-order (paint order = document order with
   `z-index` overrides).
2. On pointer_move, binary-search or BVH-query the list for the
   topmost element containing `(x, y)`.
3. Cache the list; invalidate when layout changes.

---

## O6 — GPU buffer reuse

**Stage:** renderer
**Impact:** low-medium — avoids per-frame Vec allocations and
occasional buffer reallocation.
**Complexity:** low

### Plan

1. **Staging Vecs**: keep `Vec<QuadInstance>`, `Vec<GlyphInstance>`
   across frames. Clear + repopulate instead of allocating fresh
   (glyph_pipeline.rs:453, quad_pipeline.rs similar).
2. **Buffer growth**: current strategy is `next_power_of_two` on
   overflow. Switch to 2x growth with a minimum headroom of 25 %
   to reduce reallocation frequency.
3. **Bind group caching**: only rebuild the bind group when the
   underlying buffer handle changes, not on every capacity growth.

---

## O7 — Text processing with Cow<str>

**Stage:** layout
**Impact:** low-medium — eliminates most String allocations in the
text normalisation path.
**Complexity:** low

`normalize_text_for_style` (layout/lib.rs:3090) and
`apply_text_transform` (layout/lib.rs:3053) allocate new Strings
even when the input doesn't change (common case: `white-space:
normal` with no leading/trailing whitespace, no `text-transform`).

### Plan

1. Change return types to `Cow<'_, str>`.
2. In `normalize_text_for_style`: scan text first; if no
   whitespace collapsing is needed, return `Cow::Borrowed`.
3. In `apply_text_transform`: already returns `Option<String>` —
   callers use original `&str` on `None`. This is fine; no change
   needed here.
4. In `trim_collapsed_whitespace_edges`: return `&str` slice when
   possible.

---

## O8 — Atlas eviction / growth

**Stage:** text
**Impact:** low (correctness + memory) — today atlas-full silently
drops glyphs; no eviction or resize.
**Complexity:** medium

### Plan

1. Track per-glyph last-used frame counter.
2. When `Atlas::insert` returns `None`, evict the least-recently-
   used shelf (or grow the atlas texture to 2x).
3. On eviction, remove affected entries from `glyph_cache` and
   mark the atlas dirty for re-upload.
4. Expose an `atlas_occupancy()` metric for profiling.

---

## O9 — Font sync short-circuit

**Stage:** text
**Impact:** low — avoids cloning FontRegistry when nothing changed.
**Complexity:** low

`TextContext::sync_fonts` (shape.rs:201) clones the entire
`FontRegistry` every frame. Most frames have no font changes.

### Plan

1. Add a `generation: u64` field to `FontRegistry`, bumped on
   every `register()` call.
2. In `sync_fonts`, compare the incoming generation against the
   last-synced generation. Skip clone + `font_db.sync()` if equal.

---

## O10 — Pre-sorted glyph index for hit-testing

**Stage:** layout / event handling
**Impact:** low — removes per-pointer-move sort in
`hit_glyph_boundary`.
**Complexity:** low

`hit_glyph_boundary` (layout/lib.rs:1871) collects glyphs into a
Vec and sorts by X on every pointer_move when hovering text.

### Plan

1. At shaping time, store a parallel `Vec<usize>` of glyph indices
   sorted by X position in `ShapedRun`.
2. Use binary search on that index in `hit_glyph_boundary`.

---

## O11 — Cascade performance

**Stage:** cascade
**Impact:** high — cascade is 40 ms on a modest page (icons-demo.html),
dominating the frame budget. Every hover / click triggers a full
re-cascade because `:hover` / `:active` / `:focus` pseudo-classes
must be re-evaluated.
**Complexity:** medium-high

### Root cause

`wgpu_html_style::cascade()` walks the entire tree on every call,
re-matching selectors for every element. When the only thing that
changed is `hover_path` (one element entered, one left), the cascade
still recomputes all ~200 elements.

### Plan

1. **Profile cascade internals**: instrument `cascade_node`,
   selector matching, inheritance. Find whether time is in
   selector matching, property cloning, or something else.
2. **Partial re-cascade for pseudo-class changes**: when only
   `hover_path` / `active_path` / `focus_path` changed, identify
   the old and new paths, re-cascade only those elements + their
   ancestors (whose computed style may inherit differently).
3. **Cache the CascadedTree**: store the previous cascade result.
   On a pseudo-class-only change, clone the cached tree and patch
   only the affected nodes.

---

## O12 — Per-tree image cache

**Stage:** layout / architecture
**Impact:** high (correctness) — required for multi-tree apps
(e.g. devtools inspecting a main document).
**Complexity:** high

The image cache system uses global statics (`OnceLock<Mutex<HashMap>>`)
for raw decode cache, sized cache, TTL, budget, sweep timing,
revision counter, and the worker pool. All trees share one cache.

### Plan

1. Create `ImageCache` struct holding `raw: HashMap`, `sized: HashMap`,
   config fields, and a `completions: Arc<Mutex<Vec<...>>>` shared
   queue for worker write-back.
2. Convert free functions (`load_image`, `preload_image`, `has_pending`,
   `has_animated`, `sweep`, `purge`) to methods on `ImageCache`.
3. Add `ImageCache` to `TextContext` (already per-tree, threaded
   through layout via `Ctx`).
4. Workers write completed results to a global completion channel;
   `ImageCache::drain_completions()` at layout start picks up results
   for its own pending URLs.
5. Keep the worker pool global (shared resource, safe to share).
6. Remove old global statics.

---

## Status

| Item | Status |
|------|--------|
| O1 — Pipeline cache (skip cascade+layout on idle) | **done** |
| O2 — Text measurement cache | **done** |
| O9 — Font sync short-circuit | **done** |
| O11 — Cascade performance | **done** |
| O11a — Incremental cascade (dirty flags + cached CascadedTree) | **done** |
| O11b — Pseudo-class invalidation sets | **done** |
| O11c — Bulk HashMap inheritance | **done** |
| O11d — O(n²) → O(1) in selector matching | **done** |
| O12 — Per-tree image cache | **next** |
| O3 — Display list caching | pending |
| O7 — Cow<str> text processing | pending |
| O6 — GPU buffer reuse | pending |
| O4 — Cascade COW/Arc | pending |
| O5 — Hit-test spatial index | pending |
| O10 — Pre-sorted glyph index | pending |
| O8 — Atlas eviction/growth | pending |

## Measured results (click-demo.html)

Before optimizations:
```
cascade=40ms  layout=3.5ms  paint=0.2ms  render=6ms  → 10 fps
```

After O1+O2+O9+O11:
```
cascade=0-3ms  layout=0.08ms  paint=0.02ms  render=1ms  → 60 fps
```

## Priority order

| Priority | Item | Impact | Complexity |
|----------|------|--------|------------|
| 1 | O12 — Per-tree image cache | **high** (correctness) | high |
| 2 | O3 — Display list caching | medium-high | medium |
| 3 | O7 — Cow<str> text processing | low-medium | low |
| 4 | O6 — GPU buffer reuse | low-medium | low |
| 5 | O4 — Cascade COW/Arc | medium | medium |
| 6 | O5 — Hit-test spatial index | medium | medium |
| 7 | O10 — Pre-sorted glyph index | low | low |
| 8 | O8 — Atlas eviction/growth | low | medium |

O12 (per-tree image cache) is a correctness requirement for
multi-tree apps, not just a performance optimization. All
performance-critical items (O1, O2, O9, O11) are done.
