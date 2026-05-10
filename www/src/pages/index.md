---
title: wgpu-html
hide_table_of_contents: true
---

# wgpu-html

GPU-accelerated HTML/CSS renderer for Rust.

<div style={{ textAlign: 'center', margin: '3rem 0' }}>
<a href="/wgpu-html/docs/intro" className="button button--primary button--lg">
  Read the Documentation
</a>
</div>

:::tip Quick Links

- **[Quick Start Guide](/wgpu-html/docs/getting-started/quick-start)** — get a window on screen in 30 lines
- **[Supported CSS](/wgpu-html/docs/features/supported-css)** — all supported CSS properties
- **[Getting Started](/wgpu-html/docs/getting-started/overview)** — learn what wgpu-html is
- **[GitHub](https://github.com/ivanzaida/wgpu-html)**

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
