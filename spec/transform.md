# CSS `transform` — Implementation Plan

The current codebase stores `transform` / `transform-origin` as raw `Option<String>` in `Style` and never consumes them. This document describes the full plan to wire them end-to-end: parse → typed value → layout passthrough → matrix math → DisplayList encoding → GPU shaders → paint → hit-test inverse.

> **Key invariant:** transform is intentionally **not** applied during layout. CSS transforms are post-layout visual effects; `LayoutBox` geometry is left untouched.

---

## Milestone 1 — Typed Value Model

**Crates:** `wgpu-html-models`, `wgpu-html-parser`

**Files:**
- `crates/wgpu-html-models/src/common/css_enums.rs`
- `crates/wgpu-html-models/src/css/style.rs`
- `crates/wgpu-html-parser/src/css_parser.rs`

### New types in `css_enums.rs`

```rust
pub enum CssAngle {
    Deg(f32),
    Rad(f32),
    Grad(f32),
    Turn(f32),
}
impl CssAngle {
    pub fn to_radians(self) -> f32 { … }
}

pub enum TransformFunction {
    // 2D — fully implemented
    Translate(CssLength, CssLength),
    TranslateX(CssLength),
    TranslateY(CssLength),
    Rotate(CssAngle),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    Skew(CssAngle, CssAngle),
    SkewX(CssAngle),
    SkewY(CssAngle),
    Matrix(f32, f32, f32, f32, f32, f32),  // a b c d e f
    // 3D stubs — parse correctly, evaluate to identity
    TranslateZ(CssLength),
    Translate3d(CssLength, CssLength, CssLength),
    RotateX(CssAngle),
    RotateY(CssAngle),
    RotateZ(CssAngle),
    Scale3d(f32, f32, f32),
    Matrix3d([f32; 16]),
    Perspective(CssLength),
}

pub struct TransformList(pub Vec<TransformFunction>);

pub struct TransformOrigin {
    pub x: CssLength,   // default: 50%
    pub y: CssLength,   // default: 50%
}
```

### `Style` field changes

```rust
pub transform:        Option<TransformList>,
pub transform_origin: Option<TransformOrigin>,
```

`merge_field!` in `wgpu-html-style/src/merge.rs` only requires `Option<T: Clone>`, so cascade/merge compiles unchanged.

### Parser additions (`css_parser.rs`)

Replace the existing raw-string assignments for `"transform"` and `"transform-origin"` with:

- `parse_transform_list(value: &str) -> Option<TransformList>` — tokenises CSS function calls with a recursive-descent sub-parser; handles `none` and space-separated function chains.
- `parse_transform_origin(value: &str) -> Option<TransformOrigin>` — handles 1-value and 2-value forms; keywords `top`/`bottom`/`left`/`right`/`center` map to `0%`/`100%`/`50%`.

### Tests

In `crates/wgpu-html-parser/tests/css/declarations.rs`:
- `translate(10px, 5px)` → `TransformList([Translate(Px(10.0), Px(5.0))])`
- `rotate(45deg)` / `rotate(0.785rad)` → `Rotate(Deg(45.0))` / `Rotate(Rad(0.785))`
- `scale(2)` → `Scale(2.0, 2.0)`
- `skewX(30deg)` → `SkewX(Deg(30.0))`
- `matrix(1,0,0,1,10,5)` → `Matrix(1,0,0,1,10,5)`
- chained: `translate(10px,0) rotate(45deg)` → list of two functions
- `none` → `None`
- `transform-origin: center` → `TransformOrigin { x: Percent(50), y: Percent(50) }`
- `transform-origin: top left` → `TransformOrigin { x: Percent(0), y: Percent(0) }`

---

## Milestone 2 — Layout Passthrough

**Crate:** `wgpu-html-layout`

**File:** `crates/wgpu-html-layout/src/lib.rs`

Add two fields to `LayoutBox` immediately after `opacity`:

```rust
pub transform:        Option<TransformList>,
pub transform_origin: TransformOrigin,  // always resolved; default = (50%, 50%)
```

In the section of `layout_with_text` that populates `LayoutBox` from a `CascadedNode`, copy `style.transform` verbatim and resolve `style.transform_origin` — default to `(CssLength::Percent(50.0), CssLength::Percent(50.0))`. **Do not change any size or position computation.**

### Tests

In `crates/wgpu-html-layout/src/tests.rs`:
- `transform_does_not_affect_layout` — element with `transform: translate(200px,200px)` and a neighbouring element must have identical `border_rect`/`margin_rect` to the no-transform baseline.
- `transform_origin_defaults_to_50pct` — element with no `transform-origin` has `TransformOrigin { x: Percent(50), y: Percent(50) }`.

---

## Milestone 3 — Matrix Math Library

**Crate:** `wgpu-html-layout`

**New file:** `crates/wgpu-html-layout/src/transform_math.rs` (exported `pub` from the crate root)

### Types

```rust
/// 2D affine transform — same 6 components as CSS matrix(a,b,c,d,e,f).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine2 {
    pub a: f32, pub b: f32,
    pub c: f32, pub d: f32,
    pub e: f32, pub f: f32,
}
```

### Key functions

| Function | Description |
|---|---|
| `Affine2::identity()` | Returns the identity matrix |
| `Affine2::from_function(f, em_px, ctx_size)` | Resolves one `TransformFunction` → matrix; `ctx_size` is `(border_w, border_h)` for % translates |
| `Affine2::from_list(list, em_px, ctx_size)` | Folds all functions via `concat` (left-to-right CSS application order) |
| `Affine2::concat(self, rhs)` | Matrix multiply: apply `self` first, then `rhs` |
| `Affine2::with_origin(self, ox, oy)` | `T(ox,oy) ∘ self ∘ T(-ox,-oy)` — applies transform around an origin |
| `Affine2::transform_point(self, x, y)` | Applies affine transform to a point |
| `Affine2::transform_rect_aabb(self, rect)` | Transforms all 4 corners → axis-aligned bounding box (for scissor rects) |
| `Affine2::inverse(self)` | Returns `None` when determinant ≈ 0 |
| `Affine2::is_identity(self)` | Fast check; used to skip no-op paint paths |
| `Affine2::to_gpu(self)` | Packs as `[a,b,c,d,e,f,0,0]: [f32; 8]` for GPU upload |

### Tests (inline in `transform_math.rs`)

- Identity is a passthrough for points and `concat`.
- `translate(10,5)` produces a matrix with `e=10, f=5`.
- `rotate(90°)` around origin transforms `(1,0)` → `(0,1)` (within epsilon).
- `concat(scale(2), rotate(90°))` composed correctly.
- `inverse` roundtrip: `M * M⁻¹ ≈ identity`.

---

## Milestone 4 — DisplayList Transform Encoding

**Crate:** `wgpu-html-renderer`

**File:** `crates/wgpu-html-renderer/src/paint.rs`

### `ClipRange` extension

Add one field:

```rust
pub transform: Option<[f32; 8]>,  // packed Affine2; None = identity (zero overhead)
```

### `DisplayList` API additions

- `push_transform_clip(rect, radii_h, radii_v, transform: Option<[f32; 8]>)` — like `push_clip` but also stores the transform.
- Existing `push_clip` / `pop_clip` calls pass `None` — no behaviour change.
- `finalize()` remap logic (the existing empty-clip index fix) handles transformed clips identically; no changes needed.

### Tests

- Pushing a `TransformClip` followed by quads produces a `ClipRange` with a non-identity `transform`.
- `push_clip(…, None)` produces a `ClipRange` with `transform == None`.

---

## Milestone 5 — GPU Pipeline Transform Support

**Crate:** `wgpu-html-renderer`

**Files:** `quad_pipeline.rs`, `glyph_pipeline.rs`, `image_pipeline.rs` + matching `.wgsl` shaders

### `Globals` struct extension (all three pipelines)

```rust
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct Globals {
    viewport:     [f32; 4],
    clip_rect:    [f32; 4],
    clip_radii_h: [f32; 4],
    clip_radii_v: [f32; 4],
    clip_active:  [f32; 4],
    // NEW: packed Affine2 — [a,b,c,d] then [e,f,0,0]
    transform_ab: [f32; 4],
    transform_ef: [f32; 4],
}
// Total: 112 bytes — fits within existing 256-byte CLIP_SLOT_STRIDE
```

### WGSL vertex shader (same pattern in all three shaders)

```wgsl
struct Globals {
    // ...existing fields...
    transform_ab: vec4<f32>,  // [a, b, c, d]
    transform_ef: vec4<f32>,  // [e, f, 0, 0]
}

// In vs_main, after computing local_px:
let a = globals.transform_ab.x;
let b = globals.transform_ab.y;
let c = globals.transform_ab.z;
let d = globals.transform_ab.w;
let e = globals.transform_ef.x;
let f = globals.transform_ef.y;
let px = vec2<f32>(
    a * local_px.x + c * local_px.y + e,
    b * local_px.x + d * local_px.y + f,
);
// NDC conversion continues using `px` instead of `local_px`
```

When `ClipRange::transform` is `None`, upload `[1,0,0,1,0,0,0,0]` (identity) — no cost for untransformed boxes.

### Scissor rect in `prepare` methods

When `ClipRange::transform` is non-identity, compute scissor as `Affine2::transform_rect_aabb(clip_rect)` before calling `set_scissor_rect`. Otherwise unchanged.

### Phase 2 deferred: SDF clip under transform

The fragment-shader SDF rounded-clip (`clip_active`) operates in screen space. For a transformed clip with `border-radius`, it needs an inverse-transform pass in the fragment shader. **Phase 1:** fall back to rectangular scissor AABB for transformed clips. Flag all such sites with `// FIXME(transform-clip): phase 2 — inverse fragment transform for rounded border-radius`.

---

## Milestone 6 — Paint: Transform Contexts

**Crate:** `wgpu-html`

**File:** `crates/wgpu-html/src/paint.rs`

In `paint_box_in_clip`, when `b.transform` is `Some(list)` and the resolved matrix is not identity:

1. Resolve `transform-origin` against `b.border_rect` (% → absolute px).
2. Compute `Affine2::from_list(list, em_px, (border_w, border_h)).with_origin(ox, oy)`.
3. Compose with the inherited `transform_stack: Option<Affine2>` parameter (supports nested transforms).
4. Compute scissor AABB: `composed.transform_rect_aabb(b.border_rect)` intersected with the current parent clip rect.
5. Call `out.push_transform_clip(scissor_aabb, zeros, zeros, Some(composed.to_gpu()))`.
6. Recurse into children with the new `transform_stack`.
7. Call `out.pop_clip(…)`.

**Stacking context:** `transform ≠ none` implicitly creates a new CSS stacking context. The current traversal order satisfies this for the typical use-case. Add a `// TODO(stacking-context): full z-index ordering` comment.

**`overflow: hidden` on a transformed element:** push the padding-box as the child clip rect; its scissor AABB is `composed.transform_rect_aabb(padding_box)`. Rounded-clip SDF under transform is Phase 2 (see Milestone 5).

### Tests

In `crates/wgpu-html/src/paint.rs`:
- Parse a small HTML snippet with `transform: translate(30px, 0px)`, run `paint_tree`, assert the `ClipRange` carries the non-identity matrix and child quads sit at layout-space (pre-transform) positions.
- Identity transform does **not** push an extra `ClipRange`.

---

## Milestone 7 — Hit Testing: Inverse Transform

**Crate:** `wgpu-html-layout`

**File:** `crates/wgpu-html-layout/src/lib.rs`

In `collect_hit_path_scrolled`, add `transform_stack: Option<Affine2>`. When inspecting a `LayoutBox` with a non-identity transform:

1. Compose with `transform_stack` → `composed`.
2. Fast-reject using `composed.transform_rect_aabb(border_rect)` (screen-space AABB check).
3. Inverse-transform the test point: `composed.inverse()?.transform_point(px, py)` → local-space point.
4. Recurse into children with the local-space point and the new `transform_stack`.
5. If `inverse()` returns `None` (degenerate matrix) → treat as a miss.

Apply the same pattern to `hit_path` (non-scrolled variant).

### Tests

In `crates/wgpu-html-layout/src/tests.rs`:

```
hit_test_translate_inverse
  Box at (0,0) 100×100 with transform: translate(200px,0)
  → effectively at screen columns 200–300
  → click at (250,50) → hit
  → click at (50,50) → miss

hit_test_rotate90_inverse
  Box at (100,100) 100×100, transform: rotate(90deg) around centre
  → click at its rotated footprint → hit
  → click well outside → miss

hit_test_scale_inverse
  Box at (0,0) 100×100, transform: scale(2)
  → click at (150,50) → hit (within scaled footprint)
  → click at (250,50) → miss
```

---

## Milestone 8 — Screenshot / Integration Tests

- **Display-list**: `translate(10px,5px)` → `Affine2 { a:1, b:0, c:0, d:1, e:10, f:5 }`; identity does not emit an extra `ClipRange`.
- **Paint integration**: parse HTML, call `paint_tree`, inspect `DisplayList` structure.
- **Screenshot regression** (`#[ignore]` until software renderer in CI): render `rotate(45deg)` box, compare PNG.

---

## Milestone 9 — Docs / Status Updates

Update `spec/css-properties.md` and `docs/full-status.md`:

| Property | Status after this milestone |
|---|---|
| `transform` | ⚠️ Partial — 2D functions fully rendered; 3D stubs parse, evaluate to identity |
| `transform-origin` | ⚠️ Partial — 2D only; `z` component ignored |
| `transform-box` | ❌ Deferred — `border-box` assumed |
| `transform-style` | ❌ Deferred — no `preserve-3d` |
| `backface-visibility` | ❌ Deferred — no 3D rendering |
| `rotate` / `scale` / `translate` (individual props) | ❌ Deferred — same enum, Phase 2 |

---

## Dependency Graph

```
M1 (types)
 └─► M2 (layout passthrough)
      └─► M3 (matrix math)
           ├─► M4 (DisplayList encoding)
           │    └─► M5 (GPU shaders)
           │         └─► M6 (paint)
           └─► M7 (hit-test inverse)
                └─► M8 (tests) ─► M9 (docs)
```

M1–M3 have no runtime behaviour change and can be landed/reviewed independently. M5 is highest risk — gate behind `#[cfg(feature = "css-transform")]` until M6 is proven, then enable unconditionally.

---

## Design Decisions

| Decision | Rationale |
|---|---|
| Transform is post-layout only | CSS spec; avoids layout invalidation cascade |
| `Affine2` not `Mat4` | 2D is sufficient for all 2D functions; simpler GPU struct |
| `None` = identity in `ClipRange` | Zero overhead for the common untransformed case |
| 256-byte `CLIP_SLOT_STRIDE` unchanged | 112-byte extended `Globals` still fits |
| Rounded-clip + transform SDF deferred | Requires inverse-transform in fragment shader — Phase 2 |
| Individual `rotate`/`scale`/`translate` props deferred | Same enum extension once CSS compositing order lands |
| Feature flag for M5 | Shader changes are high risk; flag allows partial rollout |

---

## Further Considerations

1. **`transition` / `animation`:** a separate system that drives `TransformList` values over time. Out of scope here; architecture is compatible (paint already recomputes `Affine2` from `TransformList` every frame).

2. **GPU identity bypass:** a `clip_transform_active: [f32; 4]` flag (like the existing `clip_active`) could let the vertex shader skip the multiply. Profile first — the identity multiply is two FMAs per vertex and likely free.

3. **Phase 2 — rounded clip under transform:** fragment shader receives `frag_coord` (screen space); to correctly clip at a rounded ancestor that is itself transformed, compute `local_coord = affine2_inv * frag_coord` before running the SDF. Requires uploading `transform_ab_inv` / `transform_ef_inv` alongside `transform_ab` / `transform_ef` in `Globals`.
