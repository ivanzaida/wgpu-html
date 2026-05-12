# Clone Audit — Layout Phase

This document catalogues redundant `.clone()` calls found in the layout crate
(`lui-layout-old`, `lui-models`, `lui-style`) and ranks them by
impact. Most fall into three root causes:

1. **Pure enums missing `Copy`** — cloning a `Display` or `BoxSizing` is heap-free
   but still emits a (trivial) `memcpy` and blocks the compiler from treating the
   value as a register-live plain integer.
2. **Collection types owned by `Style`** — `HashMap<String, String>` fields that
   cascade inheritance deep-copies once per node.
3. **`LayoutBox` clones propagating into nested Vecs** — `ShapedRun` is cloned
   when flex re-uses a laid-out box, copying potentially thousands of glyph entries.

---

## Category A — Add `Copy` to pure CSS enums

Every enum below contains no heap-allocated fields and can safely derive `Copy`.
After the change, every `.clone()` call on them becomes a no-op (the compiler
replaces `Clone::clone` with an integer copy / register move).

| Enum | Fields / variants | Currently cloned in |
|---|---|---|
| `Display` | 18 unit variants | `lib.rs:2679` — once per block layout call |
| `Position` | 5 unit variants | `lib.rs:2743-2744, 3315` — per child iteration |
| `BackgroundRepeat` | 6 unit variants | `lib.rs:1614` — per `background-image` resolve |
| `BorderStyle` | 10 unit variants | `lib.rs:2839-2842, 4116-4119` — 4× per block |
| `FontWeight` | 4 unit + `Weight(u16)` | cascade `merge_field!` — per inherited node |
| `FontStyle` | 3 unit variants | cascade — per inherited node |
| `TextAlign` | 6 unit variants | cascade — per inherited node |
| `TextTransform` | 4 unit variants | cascade — per inherited node |
| `WhiteSpace` | 6 unit variants | `lib.rs:3572` — called from many text paths |
| `Visibility` | 3 unit variants | cascade — per inherited node |
| `FlexDirection` | 4 unit variants | `flex.rs:58-61` — per flex container |
| `FlexWrap` | 3 unit variants | `flex.rs:62` — per flex container |
| `JustifyContent` | 10 unit variants | `flex.rs:63-66` — per flex container |
| `AlignItems` | 8 unit variants | `flex.rs:67-70` — per flex container |
| `AlignContent` | 9 unit variants | `flex.rs:71-74` — per flex container |
| `AlignSelf` | 9 unit variants | `flex.rs:650` — per flex item |
| `JustifyItems` | 10 unit variants | cascade — per inherited node |
| `JustifySelf` | 11 unit variants | cascade — per inherited node |
| `GridAutoFlow` | 4 unit variants | `grid.rs` — per grid container |
| `GridLine` | `Auto`, `Line(i32)`, `Span(u32)` | `grid.rs` — per placed item |
| `BoxSizing` | 2 unit variants | `lib.rs:2559, 2600, 2672, 4033` — 4× per block |
| `BackgroundClip` | 3 unit variants | `lib.rs:5128` — per box |
| `PointerEvents` | 2 unit variants | cascade — per inherited node |
| `UserSelect` | 4 unit variants | cascade — per inherited node |

**Change:** add `Copy, PartialEq, Eq` (or `Copy` alone for the numeric-variant
enums) to each `#[derive(…)]` in `crates/lui-models/src/common/css_enums.rs`.

**Cannot be `Copy`:**
- `CssLength` — contains `Box<CssMathExpr>` and `Vec<CssLength>`
- `GridTrackSize` — contains `CssLength`
- `CssColor` — has `Named(String)`, `Hex(String)`, `Function(String)`
- `CssImage` — has `Url(String)`, `Function(String)`
- `Cursor` — has `Raw(String)`

---

## Category B — `LayoutBox::text_run: Option<Arc<ShapedRun>>`

### What gets cloned

`FlexItem::box_` is `Option<LayoutBox>`. In `flex.rs:437`:

```rust
item.box_.clone().expect("item box was laid out in phase 4")
```

This clones the entire `LayoutBox` subtree of a flex item whenever it does **not**
need a stretch re-layout. A single `LayoutBox` clone deep-copies:

- `text_run: Option<ShapedRun>` — contains `Vec<PositionedGlyph>` (up to thousands
  for a paragraph), `Vec<usize>` (`glyph_chars`), `Vec<ShapedLine>`, `String`
  (`text`), `Vec<usize>` (`byte_boundaries`). Together this is easily 100–200
  bytes × glyph-count.
- `children: Vec<LayoutBox>` — recursively.
- `background_image: Option<BackgroundImagePaint>` — includes `tiles: Vec<Rect>`.

### Fix

Change `LayoutBox`:

```rust
// before
pub text_run: Option<ShapedRun>,

// after
pub text_run: Option<Arc<ShapedRun>>,
```

`ShapedRun` is write-once: it is produced by the text shaper, placed into
`LayoutBox`, and only ever read afterward (paint + hit-test). Nothing mutates it
post-construction. The `Arc` clone is 8 bytes (pointer + refcount bump).

**Sites to update:** `layout_block`, `layout_inline_block_children`,
`measure_text_leaf` in `lib.rs`; read access via `.as_ref()` in `paint.rs` and
hit-test helpers; the text-cache in `shape.rs` already returns `ShapedRun` by
value so the shaper is unaffected.

**Also:** `BackgroundImagePaint::tiles: Vec<Rect>` is similarly written-once.
Wrapping it in `Arc<Vec<Rect>>` (or simply `Box<[Rect]>`) avoids copying the tile
list when a `LayoutBox` is cloned.

---

## Category C — `FlexItem::box_` ownership vs. clone

The clone at `flex.rs:437` is a code-structure issue as much as a type issue.
`FlexItem::box_: Option<LayoutBox>` is set in phase 4, then either:

1. Thrown away and replaced in phase 7 (stretch re-layout) — the clone is wasted.
2. Used directly in the final positioning step — but `.clone()` is called
   defensively because `items` is still needed for the sizing loop that follows.

**Fix:** split the phase 4 / phase 7 / final-positioning logic so that
`item.box_` is **moved** (`.take()`) out of the item rather than cloned.
The sizing bookkeeping (main size, cross size, positions) can be separated into
a lightweight `FlexItemGeometry` struct that doesn't own the `LayoutBox`.

This eliminates the deep clone regardless of whether `ShapedRun` becomes an `Arc`.

---

## Category D — `Style` map fields → `Arc<HashMap<String, String>>`

### What gets cloned

During cascade inheritance (`lui-style/src/lib.rs:939`), the `inherit!`
macro runs:

```rust
child.$field = parent.$field.clone();
```

For the map fields this deep-clones the entire collection:

| Field | Type | Clone cost |
|---|---|---|
| `custom_properties` | `HashMap<String, String>` | Per key: `String::clone` of key + value |
| `deferred_longhands` | `HashMap<String, String>` | Same |
| `var_properties` | `HashMap<String, String>` | Same |
| `reset_properties` | `HashSet<String>` | Per entry |
| `keyword_reset_properties` | `HashSet<String>` | Per entry |

For a typical document, `custom_properties` is set on a handful of ancestors and
inherited hundreds of times. Each inheritance call copies every entry even when
the child adds nothing.

### Fix

Wrap in `Arc`:

```rust
pub custom_properties:          Arc<HashMap<String, String>>,
pub deferred_longhands:         Arc<HashMap<String, String>>,
pub var_properties:             Arc<HashMap<String, String>>,
pub reset_properties:           Arc<HashSet<String>>,
pub keyword_reset_properties:   Arc<HashSet<String>>,
```

Inheritance becomes `child.custom_properties = Arc::clone(&parent.custom_properties)`
— pointer copy only. When a child *overrides* a property: call `Arc::make_mut` on
the field before inserting; if `Arc::strong_count == 1` (only this style uses it)
the map is mutated in place; if `> 1` it is cloned once (CoW).

In practice most nodes share the exact same `Arc`, so the amortized clone cost
across a whole document drops from O(entries × nodes) to O(entries × distinct
override sites).

**Downstream:** `write_computed_style` in `winit.rs` iterates these maps for the
JSON dump — no change needed; it already borrows them.

---

## Category E — `Style::font_family` and other single-string fields → `Arc<str>`

### What gets cloned

`font_family: Option<String>` is inherited by every text node and cloned in the
cascade `merge_field!` macro. For a document with 500 text nodes all using the
same font family, this is 500 individual `String` heap allocations containing the
same bytes.

Same pattern applies to: `background_size`, `background_position`,
`text_decoration`, `box_shadow`, `border`, `flex`, `grid_column`, `grid_row`.

### Fix

Change the type to `Option<Arc<str>>`. The cascade path becomes an `Arc` pointer
clone; the parser calls `.into()` on the `&str` once at construction time.

**Cannot be `Arc<str>`:** fields that layout code calls `.as_deref()` on to get
`Option<&str>` — they already work as `Arc<str>` since `Deref<Target = str>`
is implemented for `Arc<str>`.

---

## Category F — `visible_text.clone()` in IFC paragraph builder (`lib.rs:4926`)

```rust
let run = ShapedRun {
    // ...
    text: visible_text.clone(),                       // ← clone
    byte_boundaries: lui_text::utf8_boundaries(&visible_text),
    // ...
};
```

`visible_text` is a `String` built up from paragraph spans. After constructing
the `ShapedRun`, it is not used again. The clone is entirely avoidable:

```rust
let byte_boundaries = lui_text::utf8_boundaries(&visible_text);
let run = ShapedRun {
    text: visible_text,   // move
    byte_boundaries,
    // ...
};
```

---

## Category G — `entry.value.clone()` in `load_image_url` (`lib.rs:1334`)

```rust
entry.value.clone()
```

`entry.value` is a `RawState` enum. `RawState::Ready(DecodedAsset)` contains
`Arc<Vec<u8>>` for pixel data (already arc'd), so the clone is effectively just
two `Arc` pointer bumps plus copying the `w`/`h` `u32`s. **This is already cheap
and does not need to change.**

---

## Summary & Priority

| # | Change | Files | Impact | Effort |
|---|---|---|---|---|
| A | Add `Copy` to 20+ pure CSS enums | `css_enums.rs` | Eliminates ~40 redundant copies per element per layout pass | Very Low — one line per enum |
| B | `text_run: Option<Arc<ShapedRun>>` | `lib.rs`, `paint.rs`, `shape.rs` | Eliminates O(glyphs) deep copy on every flex re-use clone | Low — mechanical type change |
| C | Move ownership in `FlexItem::box_` | `flex.rs` | Avoids the entire `LayoutBox` deep clone in flex phase 5–7 | Medium — refactor loop structure |
| D | `Arc<HashMap>` for `Style` map fields | `style.rs`, cascade `lib.rs` | Reduces cascade from O(entries×nodes) → O(entries×overrides) | Low — type change + `Arc::make_mut` at write sites |
| E | `Option<Arc<str>>` for inherited strings | `style.rs`, cascade `lib.rs` | Eliminates per-node string allocs for font/bg properties | Low — mechanical type change |
| F | Remove `visible_text.clone()` | `lib.rs:4926` | Saves one `String` alloc per IFC paragraph | Trivial — move instead of clone |

Recommended order: **A → F → B → D → C → E**.

A (Copy enums) and F (visible_text move) are pure mechanical wins with no API
surface changes. B (Arc ShapedRun) affects the public `LayoutBox` type and
should be done alongside any downstream serialisation changes. C requires
restructuring the flex algorithm's ownership model. D and E affect `Style`'s
public API and touch the cascade tests.
