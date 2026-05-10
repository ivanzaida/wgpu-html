---
sidebar_position: 4
---

# Profiling

## Built-in Profiler

The workspace includes a ring-buffer frame profiler that tracks cascade, layout, and paint timing.

### Enabling

```rust
let mut devtools = Devtools::attach(&mut tree, true); // true = enable profiler
```

Or create a profiler directly:

```rust
tree.profiler = Some(Profiler::new());
```

### Viewing Results

Press **F9** to dump a summary to stderr:

```
Frame #123 — FullPipeline (60.0 fps)
  cascade    1.23ms ████
  layout     2.45ms ████████
  paint      0.89ms ███
  nodes: 456  quads: 234  glyphs: 1234  boxes: 89
```

### Profiler Architecture

- **Ring buffer**: 240 frames (~4 seconds at 60fps)
- **Scope-based**: RAII guards with `prof_scope!` macro
- **Zero-cost when disabled**: expands to `Option::map` eliminated by compiler
- **Alert threshold**: only reports frames exceeding a configurable threshold

## PipelineTimings (always available)

```rust
pub struct PipelineTimings {
    pub cascade_ms: f64,
    pub layout_ms: f64,
    pub paint_ms: f64,
}

let (_, _, timings) = paint_tree_returning_layout_profiled(...);
println!("Frame: {timings:?}");
```

## PipelineCache

The `PipelineCache` classifies frames for incremental rendering:

```rust
pub enum PipelineAction {
    FullPipeline,        // Cascade + layout + paint
    PartialCascade,      // Re-cascade dirty nodes only
    LayoutOnly,          // Re-layout, then paint
    PatchFormControls,   // Patch form control state, repaint only
    RepaintOnly,         // Re-paint with existing layout
}
```

Frame classification considers `tree.generation`, `cascade_generation`, `form_control_generation`, and `dirty_paths`.

## Performance Characteristics

| Stage | Cold Frame | Incremental Frame |
|---|---|---|
| Cascade | O(n × rules) | O(dirty_nodes × rules) |
| Layout (block) | O(n) | O(dirty_subtree) |
| Layout (flex) | O(items²) | O(items) if container dirty |
| Layout (grid) | O(items × tracks) | O(items × tracks) if container dirty |
| Paint | O(draw_commands) | O(draw_commands) |

## Performance Tips

- Register fonts before the first cascade/layout
- Use `PipelineCache` to avoid redundant cascade on hover/focus changes
- Avoid frequent tree mutations that dirty large subtrees
- Preload images with `tree.preload_asset()` to avoid blocking layout
- Set reasonable `asset_cache_ttl` to avoid redundant image re-decodes
