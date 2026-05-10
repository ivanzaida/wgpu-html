---
title: Engine Internals
---

# Engine Internals

Developer documentation for the wgpu-html rendering engine. These docs describe the internal architecture with code references for contributors.

## Pipeline Overview

Every frame follows this path:

```
HTML string
  -> Parse (wgpu-html-parser)        -> Tree (DOM nodes)
  -> Cascade (wgpu-html-style)       -> CascadedTree (Style per node)
  -> Layout (wgpu-html-layout)       -> LayoutBox tree (positioned boxes)
  -> Paint (wgpu-html/paint.rs)      -> DisplayList (quads, images, glyphs)
  -> Render (wgpu-html-renderer)     -> GPU frame
```

The top-level orchestration lives in `crates/wgpu-html/src/lib.rs`:

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
| `wgpu-html-parser` | HTML tokenizer, CSS parser, stylesheet parser |
| `wgpu-html-tree` | DOM tree, `Element` types, selector query engine |
| `wgpu-html-models` | Shared types: `Style`, `CssLength`, `CssColor`, enums |
| `wgpu-html-style` | CSS cascade, specificity, inheritance, `var()` resolution |
| `wgpu-html-layout` | Block/flex/grid/inline/positioned layout algorithms |
| `wgpu-html-text` | cosmic-text integration, glyph shaping, atlas packing |
| `wgpu-html-renderer` | wgpu pipelines (quad/image/glyph), GPU render passes |
| `wgpu-html` | Top-level orchestration, paint, interactivity |
