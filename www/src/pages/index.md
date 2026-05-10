---
title: lui
hide_table_of_contents: true
---

# lui

GPU-accelerated HTML/CSS renderer for Rust.

<div style={{ textAlign: 'center', margin: '3rem 0' }}>
<a href="/docs/intro" className="button button--primary button--lg">
  Read the Documentation
</a>
</div>

:::tip Quick Links

- **[Quick Start Guide](/docs/getting-started/quick-start)** — get a window on screen in 30 lines
- **[Supported CSS](/docs/features/supported-css)** — all supported CSS properties
- **[Getting Started](/docs/getting-started/overview)** — learn what lui is
- **[GitHub](https://github.com/ivanzaida/lui)**

:::

## What is lui?

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
