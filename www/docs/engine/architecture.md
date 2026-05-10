---
sidebar_position: 1
---

# Engine Architecture

Every frame in wgpu-html follows a fixed pipeline of pure-data transformations:

```
HTML string
  → Parse (wgpu-html-parser)       → Tree (DOM nodes)
  → Cascade (wgpu-html-style)      → CascadedTree (Style per node)
  → Layout (wgpu-html-layout)      → LayoutBox tree (positioned boxes)
  → Paint (wgpu-html/paint.rs)     → DisplayList (quads, glyphs, images, clips)
  → Render (wgpu-html-renderer)    → GPU frame
```

Each arrow is a clean module boundary: input/output are plain data, each stage is independently testable, and JavaScript is never involved.

## Crate Map

| Crate | Responsibility |
|---|---|
| `wgpu-html-parser` | HTML tokenizer, tree builder, CSS declaration parser, stylesheet parser |
| `wgpu-html-tree` | DOM tree, `Element` types, event callbacks, interaction state, focus, selector query engine |
| `wgpu-html-models` | Shared types: `Style`, `CssLength`, `CssColor`, CSS enums, element structs |
| `wgpu-html-style` | CSS cascade, specificity, inheritance, `var()` resolution, `@media` evaluation |
| `wgpu-html-layout` | Block/flex/grid/inline/positioned layout, hit testing, image loading |
| `wgpu-html-text` | Font database, cosmic-text integration, glyph shaping, atlas packing |
| `wgpu-html-renderer` | wgpu pipelines (quad, glyph, image), GPU render passes, scissor clipping |
| `wgpu-html` | Top-level orchestration: `paint_tree`, `classify_frame`, `PipelineCache` |
| `wgpu-html-events` | Typed event structs (`HtmlEvent`, `MouseEvent`, `KeyboardEvent`, etc.) |

## Pipeline Orchestration

The top-level orchestration lives in `crates/wgpu-html/src/lib.rs`:

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

4. **`DisplayList`** is the final draw description — a flat list of draw commands (quads, glyphs, images) partitioned into clip ranges. Backend-agnostic, consumed by `wgpu-html-renderer`.

## Key Invariants

- `LayoutBox` child structure mirrors the source `Tree` — hit-testing and event dispatch depend on path compatibility
- Visual reordering (flex `order`) changes coordinates but preserves source order for hit-testing
- Image loading belongs to the layout crate, not the renderer
- Text selection is split across crates: layout provides hit/cursor geometry, `wgpu-html` stores selection state, paint renders highlights
