---
title: Profiling and Performance
---

# Profiling and Performance

lui includes built-in timing instrumentation, a ring-buffer frame
profiler, and a caching pipeline that skips work when inputs haven't
changed. Zero-overhead when disabled — the only cost is a single
`Option::is_some` branch per instrumented site.

---

## Quick start

```rust
use lui_tree::Profiler;

// Enable the profiler on your tree:
tree.profiler = Some(Profiler::tagged("my app"));
tree.profiler.as_ref().map(|p| p.enable());
```

The pipeline automatically records every stage. After each frame, call
`summary_string()` to get a human-readable block:

```rust
if let Some(ref prof) = tree.profiler {
    if let Some(summary) = prof.summary_string() {
        eprintln!("{summary}");
    }
}
```

Output:

```
[my app] frame 4123  total 3.82 ms  fps 60.0
  cascade         0.42 ms  ████████
  layout          2.12 ms  ██████████████████████████████████████
  └ flex          1.34 ms  ████████████████████████
  paint           0.28 ms  █████
  └ glyphs        0.11 ms  ██
counters: nodes=412  boxes=388  quads=903  glyphs=218
```

Bar width = fraction of the slowest span × 40. Children are indented
with `└`.

---

## Profiler API

### Enabling and disabling

```rust
prof.set_enabled(true);
prof.enable();                // shortcut
prof.disable();               // shortcut
prof.is_enabled();            // → bool
```

When disabled, every recording method is a no-op and the ring buffer is
not updated.

### Alert threshold (on-when-slow mode)

Silent until a frame exceeds a threshold. Useful for catching
intermittent jank without log spam during normal operation.

```rust
prof.set_alert_threshold(Some(33_000_000)); // 33 ms (two vsyncs)
// summary_string() returns None for frames below the threshold
prof.set_alert_threshold(None);             // always report
```

### Frame lifecycle

The harness manages frame boundaries:

```rust
prof.frame_begin();
// … pipeline work (scopes, counters) …
prof.frame_end();
```

Functions that may be called standalone use `ensure_frame_begin()`,
which is a no-op if already inside a frame:

```rust
prof.ensure_frame_begin();
// … work …
```

### Scopes (RAII spans)

Wrap a block of work in a named scope. Timing is recorded automatically
when the guard drops. Nested scopes track parent-child relationships
via an internal stack.

```rust
{
    let _guard = prof.scope("cascade");
    // … cascade work …
}
// guard dropped — end time recorded
```

For `Option<Profiler>` boundaries, use the macro:

```rust
use lui_tree::prof_scope;

prof_scope!(&tree.profiler, "layout");
// ^ expands to:
//   let __prof_scope_guard = $prof_opt.as_ref().map(|p| p.scope("layout"));
// Compiles to a no-op when tree.profiler is None.
```

For non-RAII paths (async, callbacks):

```rust
let id = prof.begin_span("upload_atlas");
// … work …
prof.end_span(id);
```

### Counters and events

Record scalar values and zero-duration markers per frame:

```rust
prof.counter("nodes", 412);
prof.counter("quads", 903);

prof.event("viewport_resize");
```

Both appear in `summary_string()` (counters line) and trace export.

### Reading the ring buffer

```rust
prof.last_frame()           // → Option<FrameRecord>
prof.frame_history()        // → Vec<FrameRecord> (oldest first)
prof.history_len()          // → usize (frames in buffer)
prof.total_frames()         // → u64 (frames ever recorded, including overwritten)
```

Each `FrameRecord` contains:

```rust
pub struct FrameRecord {
    pub frame_index: u64,
    pub frame_start_ns: u64,
    pub spans: Vec<Span>,
    pub counters: Vec<(LabelId, i64)>,
    pub events: Vec<Event>,
}

pub struct Span {
    pub name: LabelId,
    pub start_ns: u64,
    pub end_ns: u64,
    pub parent: Option<u16>,    // index into FrameRecord::spans
    pub category: SpanCategory, // Cpu | Gpu
}
```

---

## PipelineTimings

The numeric return value. Always available (no profiler required).

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

Returned by `compute_layout_profiled()` and
`paint_tree_returning_layout_profiled()`:

```rust
let (list, layout, timings) = lui::paint_tree_returning_layout_profiled(
    &tree, &mut text_ctx, &mut image_cache,
    viewport_w, viewport_h, scale, viewport_scroll_y,
);
println!("frame: cascade={:.2}ms layout={:.2}ms paint={:.2}ms total={:.2}ms",
    timings.cascade_ms, timings.layout_ms, timings.paint_ms, timings.total_ms());
```

---

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

### Three action levels

```rust
pub enum PipelineAction {
    FullPipeline,      // DOM / viewport / fonts changed — full re-cascade
    PartialCascade,    // Only pseudo-class state changed (hover / active / focus)
    RepaintOnly,       // Only scroll / selection / caret changed
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
- Cached vs current `InteractionSnapshot` (hover / active / focus paths).
- Whether images are still loading or animated.

### paint_only_pseudo_rules

```rust
cache.paint_only_pseudo_rules = lui_style::pseudo_rules_are_paint_only(tree);
```

When all pseudo-class rules (`:hover`, `:active`, `:focus`) only set paint
properties (`color`, `background-color`, `opacity`), the `PartialCascade`
path skips re-layout entirely. Instead, `patch_layout_colors()` does an
O(n) walk updating color fields in the existing `LayoutBox` tree — no
geometry recomputation.

### Usage

```rust
let mut cache = lui::PipelineCache::new();

// Each frame:
let (list, layout, timings) = lui::paint_tree_cached(
    &tree, &mut text_ctx, &mut image_cache,
    viewport_w, viewport_h, scale, viewport_scroll_y, &mut cache,
);
```

`paint_tree_cached()` automatically determines the needed action and only
does the required work. It also manages the profiler's frame lifecycle
(`ensure_frame_begin` → stages → `frame_end`).

---

## Generational tracking

```rust
pub struct Tree {
    pub generation: u64,    // bumped on DOM mutation
    pub fonts: FontRegistry, // .generation() bumped on font registration
}
```

The pipeline cache compares these against stored values. Hosts bump
`tree.generation` when mutating the DOM:

```rust
tree.generation += 1;  // or use tree.set_custom_property() which bumps automatically
```

---

## Demo profiling (F9)

The demo (`lui-demo`) has two profiling outputs, both toggled with F9:

**Compact (stdout):** rolling 1-second averages per stage with max values
and hover latency.

```
profile: 1.01s frames=60 fps=59.4  cascade=0.42/2.10  layout=2.12/8.34  paint=0.28/1.05  render=1.04/3.21  hover[moves=217 changed=12 ptr=0.042/0.310ms]
```

**Ring-buffer detail (stderr):** every frame's `summary_string()` output
with nested spans and proportional bars.

Launch with `--profile` to enable both at startup:

```bash
cargo run -p lui-demo -- --profile
```

---

## Dev profile

For development, set hot crates to `opt-level = 2` in your workspace
`Cargo.toml`:

```toml
[profile.dev.package]
lui-layout = { opt-level = 2 }
lui-style = { opt-level = 2 }
lui-renderer = { opt-level = 2 }
lui-text = { opt-level = 2 }
```

This keeps debug builds fast enough for interactive development while
preserving debug info in your host code.

---

## Data types reference

| Type | Crate | Description |
|------|-------|-------------|
| `Profiler` | `lui-tree` | Ring-buffer frame profiler with scopes, counters, string interner |
| `ScopeGuard` | `lui-tree` | RAII guard returned by `Profiler::scope()` |
| `SpanId` | `lui-tree` | Opaque handle for manual `begin_span` / `end_span` |
| `LabelId` | `lui-tree` | Interned string identifier for span/counter/event names |
| `FrameRecord` | `lui-tree` | All profiling data for one frame |
| `Span` | `lui-tree` | One measured time span with parent tracking |
| `RingBuffer<N>` | `lui-tree` | Fixed-capacity ring buffer (default N=240, ≈ 4s at 60 Hz) |
| `PipelineTimings` | `lui` | `{cascade_ms, layout_ms, paint_ms}` return value |
| `PipelineCache` | `lui` | Caches layout + cascade to skip redundant work |
| `PipelineAction` | `lui` | `FullPipeline \| PartialCascade \| RepaintOnly` |
| `FrameTimings` | `lui-winit` | `{cascade_ms, layout_ms, paint_ms, render_ms}` passed to `AppHook::on_frame` |

---

## Complete example

```rust
use lui::PipelineCache;
use lui_tree::Profiler;

// Setup
let mut cache = PipelineCache::new();
tree.profiler = Some(Profiler::tagged("my app"));
tree.profiler.as_ref().map(|p| p.enable());

// Per-frame:
let (list, layout, timings) = lui::paint_tree_cached(
    &tree, &mut text_ctx, &mut image_cache,
    viewport_w, viewport_h, scale, viewport_scroll_y, &mut cache,
);

// Print profiler summary (only when above threshold, if set)
if let Some(ref prof) = tree.profiler {
    if let Some(summary) = prof.summary_string() {
        eprintln!("{summary}");
    }
}

// PipelineTimings always available regardless of profiler state:
match lui::classify_frame(&tree, &cache, &image_cache, vw, vh, scale) {
    PipelineAction::FullPipeline => println!("full frame: {:.2}ms", timings.total_ms()),
    PipelineAction::PartialCascade => println!("cascade only: {:.2}ms", timings.cascade_ms),
    PipelineAction::RepaintOnly => println!("repaint only"),
}
```
