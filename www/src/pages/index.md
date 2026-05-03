---
title: wgpu-html
hide_table_of_contents: true
---

# wgpu-html

GPU-accelerated HTML/CSS renderer for Rust.

<div style={{ textAlign: 'center', margin: '3rem 0' }}>
  <a href="/docs/" className="button button--primary button--lg">
    Read the Documentation
  </a>
</div>

:::tip Quick Links

- **[Quick Start Guide](/docs/getting-started/quick-start)** — get a window on screen in 30 lines
- **[CSS Property Index](/docs/css/property-index)** — all supported CSS properties
- **[vs RmlUI Comparison](/docs/comparison-wgpu-html-vs-rmlui)** — see how wgpu-html stacks up
- **[GitHub](https://github.com/wgpu-html/wgpu-html)**

:::

## What is wgpu-html?

A Rust library that parses HTML and CSS, computes layout (block, flexbox, grid), shapes text, and paints everything to a GPU-backed display list via wgpu. Zero JavaScript — ever.

## Architecture

```
HTML/CSS string
  → Parser (tokenizer + tree builder + CSS parser)
  → Style (cascade + inheritance + UA defaults)
  → Layout (block flow + flexbox + grid + inline)
  → Paint (DisplayList of quads + glyphs + images)
  → Renderer (GPU via wgpu)
```
