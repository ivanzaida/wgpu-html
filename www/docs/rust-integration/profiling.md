---
title: Profiling and Performance
---

# Profiling and Performance

wgpu-html includes built-in timing instrumentation and a caching pipeline that skips work when inputs haven't changed.

## PipelineTimings

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct PipelineTimings {
    pub cascade_ms: f64,
    pub layout_ms: f64,
    pub paint_ms: f64,
}

impl PipelineTimings {
    pub fn total_ms(self) -> f64 {
        self.cascade_ms + self.layout_ms + self.paint_ms
    }
}
```

Returned by `compute_layout_profiled()` and `paint_tree_returning_layout_profiled()`:

```rust
let (list, layout, timings) = wgpu_html::paint_tree_returning_layout_profiled(
    &tree, &mut text_ctx, &mut image_cache,
    viewport_w, viewport_h, scale,
);
println!("frame: cascade={:.2}ms layout={:.2}ms paint={:.2}ms total={:.2}ms",
    timings.cascade_ms, timings.layout_ms, timings.paint_ms, timings.total_ms());
```

## PipelineCache

```rust
pub struct PipelineCache {
    snapshot: InteractionSnapshot,
    viewport: (f32, f32),
    scale: f32,
    font_generation: u64,
    tree_generation: u64,
    layout: Option<LayoutBox>,
    cascaded: Option<CascadedTree>,
    pub paint_only_pseudo_rules: bool,
}
```

### Three Action Levels

```rust
pub enum PipelineAction {
    FullPipeline,      // DOM/viewport/fonts changed — full re-cascade
    PartialCascade,    // Only pseudo-class state changed (hover/active/focus)
    RepaintOnly,       // Only scroll/selection/caret changed
}

pub fn classify_frame(
    tree: &Tree, cache: &PipelineCache,
    image_cache: &ImageCache,
    viewport_w: f32, viewport_h: f32, scale: f32,
) -> PipelineAction
```

`classify_frame()` compares:
- Cached vs current viewport size and scale.
- Cached vs current `tree.generation` (DOM mutations).
- Cached vs current `tree.fonts.generation()` (font changes).
- Cached vs current `InteractionSnapshot` (hover/active/focus paths).
- Whether images are still loading or animated.

### paint_only_pseudo_rules

```rust
cache.paint_only_pseudo_rules = wgpu_html_style::pseudo_rules_are_paint_only(tree);
```

When all pseudo-class rules (`:hover`, `:active`, `:focus`) only set paint properties (`color`, `background-color`, `opacity`), the `PartialCascade` path skips re-layout entirely. Instead, `patch_layout_colors()` does an O(n) walk updating color fields in the existing `LayoutBox` tree — no geometry recomputation.

### Usage

```rust
let mut cache = wgpu_html::PipelineCache::new();

// Each frame:
let (list, layout, timings) = wgpu_html::paint_tree_cached(
    &tree, &mut text_ctx, &mut image_cache,
    viewport_w, viewport_h, scale, &mut cache,
);
```

`paint_tree_cached()` automatically determines the needed action and only does the required work.

## Generational Tracking

```rust
pub struct Tree {
    pub generation: u64,    // bumped on DOM mutation
    pub fonts: FontRegistry, // .generation() bumped on font registration
}
```

The pipeline cache compares these against stored values. Hosts bump `tree.generation` when mutating the DOM:

```rust
tree.generation += 1;  // or use tree.set_custom_property() which bumps automatically
```

## Per-Frame Profiler

```rust
tree.profiler = Some(wgpu_html_tree::Profiler::new());
```

When set, the pipeline records each stage's duration via `profiler.record("cascade", duration)`. Call `profiler.flush()` to read the accumulated stats.

## ProfileWindow

The demo crate (`wgpu-html-demo`) can display a `ProfileWindow` with 1-second rolling averages per stage:

```
[profile] cascade: 0.82ms | layout: 2.31ms | paint: 0.45ms | total: 3.58ms | fps: 60
```

## Dev Profile

For development, set hot crates to `opt-level = 2` in your workspace `Cargo.toml`:

```toml
[profile.dev.package]
wgpu-html-layout = { opt-level = 2 }
wgpu-html-style = { opt-level = 2 }
wgpu-html-renderer = { opt-level = 2 }
wgpu-html-text = { opt-level = 2 }
```

This keeps debug builds fast enough for interactive development while preserving debug info in your host code.

## Complete Example

```rust
let mut cache = wgpu_html::PipelineCache::new();
tree.profiler = Some(wgpu_html_tree::Profiler::new());

// Per-frame:
let action = wgpu_html::classify_frame(&tree, &cache, &image_cache, vw, vh, scale);

let (list, layout, timings) = wgpu_html::paint_tree_cached(
    &tree, &mut text_ctx, &mut image_cache, vw, vh, scale, &mut cache,
);

match action {
    PipelineAction::FullPipeline => println!("full frame: {:.2}ms", timings.total_ms()),
    PipelineAction::PartialCascade => println!("cascade only: {:.2}ms", timings.cascade_ms),
    PipelineAction::RepaintOnly => println!("repaint only"),
}
```
