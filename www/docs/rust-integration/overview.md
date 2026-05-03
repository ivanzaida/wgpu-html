---
id: overview
title: Rust Integration Overview
---

# Rust Integration Overview

The `wgpu-html` crate is the top-level facade. It re-exports all sub-crates and provides convenient pipeline functions so you only need one dependency.

## Integration Approaches

| Approach | Use case | Crate |
|---|---|---|
| Raw API | Full control over render loop, font loading, event dispatch | `wgpu-html` |
| Winit harness | Batteries-included window app | `wgpu-html-winit` |
| egui backend | Embed in egui/eframe | `wgpu-html-egui` |

## Core Facade

```rust
// Cargo.toml
[dependencies]
wgpu-html = "0.1"
```

The facade re-exports:
- `wgpu_html_layout` as `layout`
- `wgpu_html_models` as `models`
- `wgpu_html_parser` as `parser`
- `wgpu_html_renderer` as `renderer`
- `wgpu_html_style` as `style`
- `wgpu_html_text` as `text`
- `wgpu_html_tree` as `tree`
- `wgpu_html::paint`, `wgpu_html::interactivity`, `wgpu_html::scroll`

## Main API

### One-Shot Pipeline

```rust
pub fn paint_tree_returning_layout(
    tree: &Tree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
) -> (DisplayList, Option<LayoutBox>)
```

Cascade → layout → paint in one call. Returns a finalized `DisplayList` ready for the renderer, plus the `LayoutBox` for hit-testing.

### Profiled Variant

```rust
pub fn paint_tree_returning_layout_profiled(
    tree: &Tree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
) -> (DisplayList, Option<LayoutBox>, PipelineTimings)
```

Returns `PipelineTimings { cascade_ms, layout_ms, paint_ms }`.

### Separate Layout + Paint

```rust
pub fn compute_layout(
    tree: &Tree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
) -> Option<LayoutBox>

pub fn paint_layout(root: &LayoutBox, list: &mut DisplayList);
```

Use when you need the layout for hit-testing between frames without repainting.

### Cached Pipeline

```rust
pub fn paint_tree_cached<'c>(
    tree: &Tree,
    text_ctx: &mut TextContext,
    image_cache: &mut ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
    cache: &'c mut PipelineCache,
) -> (DisplayList, Option<&'c LayoutBox>, PipelineTimings)
```

Automatically skips cascade + layout when inputs haven't changed (viewport size, tree generation, font generation, interaction state). Three action levels:
- `FullPipeline` — DOM/viewport/fonts changed, full re-cascade
- `PartialCascade` — only pseudo-class state changed (hover/active/focus)
- `RepaintOnly` — only scroll/selection/caret changed

### Screenshot APIs

```rust
renderer.capture_to(&list, width, height, "frame.png")?;
renderer.capture_rect_to(&list, region, "region.png")?;
wgpu_html::screenshot_node_to(&tree, &mut text_ctx, &mut image_cache, &mut renderer, &path, ...)?;
```

## Sub-Pages

- [Integration Guide](./integrating) — step-by-step from scratch
- [Winit Harness](./winit-harness) — batteries-included `WgpuHtmlWindow`
- [egui Backend](./egui-backend) — embedding in egui/eframe
- [Screenshots](./screenshots) — capture to PNG
- [Profiling](./profiling) — performance measurement
