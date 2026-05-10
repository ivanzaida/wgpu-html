---
sidebar_position: 1
---

# Engine Architecture

Every frame in lui follows a fixed pipeline of pure-data transformations:

```
HTML string
  → Parse (lui-parser)              → Tree (DOM nodes)
  → Cascade (lui-style)             → CascadedTree (Style per node)
  → Layout (lui-layout)             → LayoutBox tree (positioned boxes)
  → Paint (lui/paint.rs)            → DisplayList (quads, glyphs, images, clips)
  → Render (B: RenderBackend)       → GPU frame
```

Each arrow is a clean module boundary: input/output are plain data, each stage is independently testable, and JavaScript is never involved.

## Crate Map

| Crate | Responsibility |
|---|---|
| `lui-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `lui-tree` | DOM tree, `Element` types, event callbacks, interaction state, focus, selector query engine |
| `lui-models` | Shared types: `Style`, `CssLength`, `CssColor`, CSS enums, element structs |
| `lui-style` | CSS cascade, specificity, inheritance, `var()` resolution, `@media` evaluation |
| `lui-layout` | Block/flex/grid/inline/positioned layout, hit testing, image loading |
| `lui-text` | Font database, cosmic-text integration, glyph shaping, atlas packing |
| `lui-display-list` | Backend-agnostic display list IR: `DisplayList`, `Quad`, `Rect`, `Color`, etc. |
| `lui-render-api` | `RenderBackend` trait and `RenderError` — the abstraction any GPU renderer implements |
| `lui-renderer-wgpu` | wgpu implementation of `RenderBackend` (quad, glyph, image pipelines) |
| `lui` | Top-level orchestration: `paint_tree`, `classify_frame`, `PipelineCache` |
| `lui-events` | Typed event structs (`HtmlEvent`, `MouseEvent`, `KeyboardEvent`, etc.) |

## Pipeline Orchestration

The top-level orchestration lives in `crates/lui/src/lib.rs`:

| Function | Purpose |
|---|---|
| `paint_tree()` | Full cascade + layout + paint (no text) |
| `paint_tree_with_text()` | Full pipeline with text shaping |
| `paint_tree_cached()` | Cached pipeline, skips unchanged stages |
| `classify_frame()` | Decides: full / partial-cascade / layout-only / repaint-only |

The cached path uses `PipelineCache` and dirty flags to skip cascade/layout when only interaction state (hover, focus) changed.

## Data Flow

1. **`Tree`** is the source of truth — holds DOM nodes, fonts, interaction state, and event callbacks. The application mutates the tree to change content.

2. **`CascadedTree`** is derived — produced by `wgpu_style::cascade()`, carrying a resolved `Style` per element. For re-cascade on hover/focus change, `cascade_incremental()` patches in-place.

3. **`LayoutBox`** tree is derived — produced by `wgpu_style_layout::layout_with_text()`. All geometry in absolute pixels, including margins, borders, padding, text runs, and image data.

4. **`DisplayList`** is the final draw description — a flat list of draw commands (quads, glyphs, images) partitioned into clip ranges. Defined in `lui-display-list` (no GPU types), consumed by any `RenderBackend` implementation.

## Key Invariants

- `LayoutBox` child structure mirrors the source `Tree` — hit-testing and event dispatch depend on path compatibility
- Visual reordering (flex `order`) changes coordinates but preserves source order for hit-testing
- Image loading belongs to the layout crate, not the renderer
- Text selection is split across crates: layout provides hit/cursor geometry, `lui` stores selection state, paint renders highlights
