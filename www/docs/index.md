---
title: Welcome to wgpu-html
---

# wgpu-html

**GPU-accelerated HTML/CSS renderer for Rust, built on wgpu.**

wgpu-html is a complete HTML/CSS rendering engine that runs directly on the GPU via wgpu (Vulkan, Metal, DX12). It parses real HTML5, applies CSS styling (Flexbox + Grid), lays out page geometry, and renders through custom GPU pipelines — all from a single Rust binary. **No web browser. No JavaScript. Ever.**

## Key Features

- **GPU rendering via wgpu** — Quad SDF pipeline, glyph atlas rendering, and image pipeline all run on Vulkan/Metal/DX12
- **CSS Flexbox & Grid** — Complete CSS Flexbox Level 1 and CSS Grid layout with `fr` units, `minmax()`, `repeat()`, and auto-placement
- **Full text shaping** — cosmic-text (HarfBuzz-based) shaping with glyph atlas, `text-align`, `letter-spacing`, `text-transform`, and text decorations
- **Component framework** — Elm-architecture component system with scoped CSS, reactive state (`Store<T>`), render caching, and an `El` builder DSL
- **Form controls** — `<input>` (22 types), `<textarea>`, `<button>`, `<select>`, full text editing with caret, selection, and clipboard
- **No JavaScript needed** — All logic is Rust. Callbacks are Rust closures. The component framework handles UI state.

## Architecture Pipeline

```
HTML/CSS string
   │
   ▼  wgpu-html-parser           Tokenizer + tree builder + CSS parser
Tree<Node<Element>>
   │
   ▼  wgpu-html-style            UA defaults + selector match + cascade + inheritance
CascadedTree<CascadedNode>
   │
   ▼  wgpu-html-layout           Block flow + Flex + Grid + Inline (IFC) + text shaping
LayoutBox tree
   │
   ▼  wgpu-html (paint.rs)       LayoutBox → DisplayList (quads + glyphs + clip ranges)
DisplayList
   │
   ▼  wgpu-html-renderer         Quad pipeline (SDF) + Glyph pipeline + scissor/clip
Frame on wgpu surface
```

## First Steps

New to wgpu-html? Start with the [Getting Started overview](getting-started/overview) to understand what wgpu-html is, then follow the [Quick Start guide](getting-started/quick-start) to get your first window on screen.
