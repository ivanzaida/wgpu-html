---
title: Engine Internals
---

# Engine Internals

Developer documentation for the lui rendering engine. These docs describe the internal architecture with code references for contributors.

## Pipeline Overview

Every frame follows this path:

```
HTML string
  -> Parse (lui-parser)        -> Tree (DOM nodes)
  -> Cascade (lui-style)       -> CascadedTree (Style per node)
  -> Layout (lui-layout)       -> LayoutBox tree (positioned boxes)
  -> Paint (lui/paint.rs)      -> DisplayList (quads, images, glyphs)
  -> Render (lui-renderer)     -> GPU frame
```

The top-level orchestration lives in `crates/lui/src/lib.rs`:

| Function | Line | Purpose |
|---|---|---|
| `paint_tree()` | 25 | Cascade + layout + paint (no text) |
| `paint_tree_with_text()` | 36 | Full pipeline with text shaping |
| `paint_tree_cached()` | 301 | Cached pipeline, skips unchanged stages |
| `classify_frame()` | 255 | Decides: full / partial-cascade / layout-only / repaint-only |

The cached path (`paint_tree_cached`) uses `PipelineCache` and dirty flags to skip cascade/layout when only interaction state changed.

## Crate Map

| Crate | Responsibility |
|---|---|
| `lui-parser` | HTML tokenizer, CSS parser, stylesheet parser |
| `lui-tree` | DOM tree, `Element` types, selector query engine |
| `lui-models` | Shared types: `Style`, `CssLength`, `CssColor`, enums |
| `lui-style` | CSS cascade, specificity, inheritance, `var()` resolution |
| `lui-layout` | Block/flex/grid/inline/positioned layout algorithms |
| `lui-text` | cosmic-text integration, glyph shaping, atlas packing |
| `lui-renderer` | wgpu pipelines (quad/image/glyph), GPU render passes |
| `lui` | Top-level orchestration, paint, interactivity |
