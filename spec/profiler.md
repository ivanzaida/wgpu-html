# wgpu-html — Profiler Spec

A design for a per-stage, per-frame profiler over the wgpu-html
pipeline. Captures CPU time spent in `parse → cascade → layout →
paint → submit`, GPU time spent in the render pass, and structural
counters (node count, rule count, display-list size, draw calls).

This spec is staged like `devtools.md`: each phase ends in something
the host or a test can use, even before a UI panel exists. Early
phases produce **counters** the host can read; later phases produce
**timeline traces** and an in-process **panel**.

Companion to `roadmap.md` (engine milestones), `status.md`
(implementation snapshot), and `devtools.md` (inspector tooling). The
profiler is a sibling to devtools — it shares the "in-process,
opt-in, zero-cost-when-off" design, but answers _"where did this
frame go?"_ rather than _"what is this element?"_.

---

## 0. Current state (2026-04-29)

A lightweight **inline profiler** exists today without a dedicated
crate. Its pieces:

- **`PipelineTimings`** (`wgpu-html/src/lib.rs`): `{ cascade_ms,
  layout_ms, paint_ms }` + `total_ms()`. Returned by
  `compute_layout_profiled` and `paint_tree_returning_layout_profiled`.
- **`ProfileWindow`** (`wgpu-html-demo/src/main.rs`): rolling per-second
  accumulators for every pipeline stage (`tree`, `cascade`, `layout`,
  `paint`, `postprocess`, `atlas_upload`, `render`) plus a dedicated
  hover-latency breakdown (`hover_pointer_move`, `hover_frame_*` — avg
  and max per pointer-move event, and per hover-triggered frame). Printed
  to stderr once per second via `take_line_if_due()`.
- **Stage coverage**: cascade, layout, and paint timings come from
  `PipelineTimings`; atlas upload, postprocess, and GPU submit/render
  are timed separately in the demo's redraw loop.

This covers the core of §4 stage timing. What is **not yet built**:
the `wgpu-html-profiler` crate, ring-buffer history, GPU timestamp
queries, `summary_string()` as a library method, trace export to
Chrome JSON, the overlay HUD, and the self-hosted panel. P1's crate
infrastructure is the next logical step; it would extract
`PipelineTimings`/`ProfileWindow` into a proper crate and add
`scope()` + `counter()`.

---

## 1. Goals

- **Stage timing**: per frame, how many microseconds did `parse`,
  `cascade`, `layout`, `paint`, `submit` each take.
- **Sub-stage timing**: which selectors / which layout subtree /
  which paint pushers dominate.
- **GPU timing**: how long the render pass takes on the GPU, and
  how long `Queue::submit` blocks (queue stall vs. GPU work).
- **Counters**: nodes parsed, rules matched, layout boxes produced,
  display-list quads, draw calls, vertex / index bytes uploaded.
- **History**: a rolling window of the last N frames (default 240,
  ≈ 4 s at 60 Hz).
- **Export**: dump the rolling buffer to Chrome's
  [Trace Event Format] JSON so traces open in `chrome://tracing` /
  `ui.perfetto.dev`.
- **Cheap when off**: the only cost when disabled is one branch
  per stage entry.

[Trace Event Format]: https://docs.google.com/document/d/1CvAClvFfyA5R-PhYUmn5OOQtYMH4h6I0nSsKchNAySU

## 2. Non-goals

- No sampling profiler (no stack walks, no `perf`-style PMC).
  Pure instrumentation only.
- No allocation tracker — out of scope; can be layered in via a
  global allocator wrapper later if needed.
- No JS profiler — there is no JS engine.
- No remote / wire protocol. The dump format is a file on disk.
- No flamegraph SVG generator in-tree; the user opens the JSON in
  an external viewer.
- No automatic regression-detection ("frame X is 3σ slower than
  baseline"); the profiler emits data, the host / CI decides.

## 3. Architecture

```
                                 ┌────────────────────────────┐
                  host app ────▶ │ wgpu_html_profiler::Profiler│
                                 │   • frame_begin / frame_end │
                                 │   • scope("layout")         │
                                 │   • counter("quads", n)     │
                                 │   • snapshot()              │
                                 │   • dump_trace(path)        │
                                 └──────────────┬─────────────┘
                                                │
              ┌─────────────────────────────────┼─────────────────────────────────┐
              │                                 │                                 │
              ▼                                 ▼                                 ▼
       parse / cascade /                 paint translation                renderer.render
       layout (CPU)                      (CPU)                            (CPU + GPU)
       │ scope guards                    │ scope guards                   │ scope + timestamp
       │ wrap each stage                 │ wrap paint                     │ writes wgpu
       │                                 │                                 │ TimestampQuery
       ▼                                 ▼                                 ▼
                            Profiler::record(span | counter)
                                                │
                                                ▼
                                  RingBuffer<FrameRecord, N>
```

A new crate `wgpu-html-profiler` owns:

- `Profiler` — the host-facing handle. Holds a ring buffer of
  per-frame records, a current-frame builder, and a `wgpu`
  timestamp-query pool when GPU timing is enabled.
- `Scope` (RAII guard) — `let _g = profiler.scope("layout")` records
  start time on construction and end time on drop. Nestable.
- `FrameRecord` — flat list of `Span { name, start_ns, end_ns,
  depth }` plus a `Counters` struct. Designed to serialize directly
  into trace-event JSON.
- Optional `panel` module behind a Cargo feature. The panel is a
  **wgpu-html document** authored by the profiler crate (HTML+CSS
  string templates) and painted by the same engine. No egui. Same
  self-hosting approach as `devtools.md` §3.

### Why a separate crate

- Engine crates (parser / style / layout / renderer) stay free of
  profiler types. They take an `Option<&mut dyn ProfilerSink>` (or
  call through a thin macro) rather than depending on the crate.
- Hosts that don't need profiling pay nothing — the `dyn` is `None`
  and the macro expands to a no-op.
- Tests can link the profiler standalone for benchmarking without
  pulling in the panel module or wgpu timestamp queries.

### Scopes vs. hooks

Two complementary instrumentation styles, both supported:

1. **Scope** (RAII): the engine wraps each stage in
   `let _g = profiler.scope("cascade");`. Zero-cost when the
   profiler is disabled (the macro checks the flag and returns a
   `()` guard).
2. **Counter**: `profiler.counter("rules_matched", n);` records a
   scalar at the current point in the frame. Useful for things that
   aren't a duration (display-list size, dirty-node count).

## 4. What gets measured

Stages, in pipeline order. Each row is a top-level span the
profiler records every frame.

| Stage             | Where                                            | Sub-spans (post-P3)                                   |
|-------------------|--------------------------------------------------|-------------------------------------------------------|
| `parse_html`      | `wgpu_html_parser::parse`                        | tokenize / build_tree                                 |
| `parse_css`       | inside cascade or pre-cascade                    | per `<style>` block                                   |
| `cascade`         | `wgpu_html_style::cascade`                       | gather_rules / match / merge                          |
| `layout`          | `wgpu_html_layout::layout`                       | block / flex / measure / repos                        |
| `paint`           | `wgpu_html::paint::paint_tree`                   | bg / borders / radii                                  |
| `build_buffers`   | renderer vertex/index upload                     | —                                                     |
| `submit_cpu`      | `Queue::submit` CPU time                         | —                                                     |
| `gpu_render_pass` | wgpu timestamp queries around the pass           | —                                                     |
| `present`         | surface present time                             | —                                                     |

Counters captured per frame:

| Counter             | Source                                       |
|---------------------|----------------------------------------------|
| `nodes`             | `Tree::node_count()` (add if missing)        |
| `rules`             | cascade — total declarations matched         |
| `layout_boxes`      | `LayoutBox` count via post-order walk        |
| `quads`             | `DisplayList::quads.len()`                   |
| `draw_calls`        | renderer-internal (currently 1)              |
| `vertex_bytes`      | renderer-internal                            |
| `index_bytes`       | renderer-internal                            |
| `surface_size`      | `width × height`                             |

Engine-side gaps to fix as part of P1 / P2:

1. `Tree` and `LayoutBox` need `node_count() / box_count()` (cheap
   recursive walk, can be cached).
2. `Renderer` already exposes `last_frame_ms` / `quad_count` (per
   `devtools.md` D2). Profiler reads through that surface; no
   duplication.
3. Renderer needs an opt-in `wgpu::QuerySet` for timestamps. Off by
   default — initialised only when the profiler asks for GPU
   timing.

## 5. Public API (sketch)

```rust
// crates/wgpu-html-profiler/src/lib.rs

pub struct Profiler {
    enabled:      bool,
    gpu_enabled:  bool,
    history:      RingBuffer<FrameRecord>,   // last N frames
    current:      Option<FrameBuilder>,      // in-progress frame
    gpu:          Option<GpuTimer>,          // wgpu timestamp pool
}

impl Profiler {
    pub fn new(capacity: usize) -> Self;          // disabled
    pub fn enable(&mut self, on: bool);
    pub fn enable_gpu(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    // Frame lifecycle ---------------------------------------------------
    pub fn frame_begin(&mut self);
    pub fn frame_end(&mut self);

    // Scopes ------------------------------------------------------------
    /// RAII guard. When the profiler is disabled this is a no-op `()`.
    pub fn scope(&mut self, name: &'static str) -> Scope<'_>;

    /// For async / non-RAII paths.
    pub fn span_begin(&mut self, name: &'static str) -> SpanId;
    pub fn span_end(&mut self, id: SpanId);

    // Counters / events -------------------------------------------------
    pub fn counter(&mut self, name: &'static str, value: i64);
    pub fn event(&mut self, name: &'static str);   // zero-duration marker

    // GPU --------------------------------------------------------------
    /// Allocate a timestamp pair for the next render pass. The renderer
    /// calls write_timestamp() at begin/end and resolve_gpu_timings()
    /// next frame (one-frame latency to avoid stalling).
    pub fn gpu_pass_begin<'a>(&'a mut self, encoder: &mut wgpu::CommandEncoder)
        -> Option<GpuPassToken<'a>>;
    pub fn gpu_pass_end(&mut self, token: GpuPassToken<'_>,
                        encoder: &mut wgpu::CommandEncoder);
    pub fn resolve_gpu_timings(&mut self, queue: &wgpu::Queue);

    // Read-out ---------------------------------------------------------
    pub fn last_frame(&self) -> Option<&FrameRecord>;
    pub fn snapshot(&self) -> ProfilerSnapshot<'_>;   // borrows ring buffer
    pub fn frame_history(&self) -> impl Iterator<Item = &FrameRecord>;

    // Export -----------------------------------------------------------
    /// Chrome trace-event JSON of the current ring buffer.
    pub fn dump_trace(&self, path: &Path) -> std::io::Result<()>;
    pub fn write_trace<W: Write>(&self, w: &mut W) -> std::io::Result<()>;
}

pub struct Scope<'a> { /* records end_ns on drop */ }
pub struct SpanId(u32);
pub struct GpuPassToken<'a> { /* indices into the timestamp set */ }

pub struct FrameRecord {
    pub frame_index:  u64,
    pub cpu_total_ns: u64,
    pub gpu_total_ns: Option<u64>,
    pub spans:        Vec<Span>,        // flat, depth-tagged
    pub counters:     Vec<(&'static str, i64)>,
    pub events:       Vec<Event>,
}

pub struct Span {
    pub name:     &'static str,
    pub start_ns: u64,
    pub end_ns:   u64,
    pub depth:    u8,
    pub category: SpanCategory,         // CPU | GPU
}
```

A convenience macro hides the disabled-case branch:

```rust
#[macro_export]
macro_rules! prof_scope {
    ($prof:expr, $name:literal) => {
        let _scope = $prof.as_mut().map(|p| p.scope($name));
    };
}
```

so call sites read:

```rust
fn run_frame(&mut self) {
    prof_scope!(self.profiler, "frame");
    {
        prof_scope!(self.profiler, "parse");
        let tree = parse_html(self.html);
    }
    // …
}
```

When `self.profiler` is `None`, every macro invocation collapses
to a single `Option::map` returning `None` — the optimiser strips
it.

## 6. GPU timing details

`wgpu::QuerySet { ty: Timestamp, count: 2 * MAX_INFLIGHT }` allocated
once per session. Each frame:

1. Encoder records `write_timestamp(qset, 2*frame%MAX)` before the
   render pass begin and `2*frame%MAX + 1` after end.
2. After submit, schedule a buffer copy of the resolved timestamps
   into a mappable buffer.
3. **Next** frame, map the buffer for the previous frame's pair,
   compute `(end - begin) * device.timestamp_period()` ns, store
   into `FrameRecord::gpu_total_ns`.

The one-frame lag avoids stalling on `Queue::on_submitted_work_done`.
`MAX_INFLIGHT` = 3 covers double + triple buffering.

GPU timing is **opt-in** because:
- It requires the `TIMESTAMP_QUERY` wgpu feature, which not all
  adapters support (some Vulkan mobile, WebGPU).
- It costs a query set + readback buffer and a one-frame latency.

If the feature is missing, `enable_gpu` returns `false` and CPU-only
profiling continues to work.

## 7. Integration with the pipeline

Per-frame flow once the profiler is wired in (additions in **bold**):

```
frame_begin                                          ── prof.frame_begin()
  parse                                              ── prof.scope("parse")
  cascade                                            ── prof.scope("cascade")
  layout                                             ── prof.scope("layout")
  paint                                              ── prof.scope("paint")
  build_buffers                                      ── prof.scope("build_buffers")
  encoder = device.create_command_encoder()
  ── prof.gpu_pass_begin(encoder)  (timestamp #2N)
  render_pass(...)                                   ── prof.scope("render_pass_cpu")
  ── prof.gpu_pass_end(encoder)    (timestamp #2N+1)
  submit                                             ── prof.scope("submit_cpu")
  prof.resolve_gpu_timings(queue)
frame_end                                            ── prof.frame_end()
```

The renderer takes an `Option<&mut Profiler>` parameter on its
`render()` entry (or a `&mut dyn ProfilerSink` if we want to keep
crate boundaries clean — TBD in P2).

## 8. UI surface

### 8.1 Console summary (no text rendering needed)

`Profiler::summary_string()` returns a fixed-width text block:

```
frame 1234   total 4.21 ms   cpu 3.10 ms   gpu 1.04 ms
  parse           0.12 ms   ████
  cascade         0.34 ms   ███████████
  layout          0.21 ms   ███████
  paint           0.18 ms   ██████
  build_buffers   0.05 ms   ██
  submit_cpu      0.02 ms   █
  gpu_render      1.04 ms   ██████████████████████████████
counters: nodes=412 rules=88 boxes=412 quads=903 dc=1
```

Useful from a host app's stdout, from CI, or from tests asserting
budgets ("layout for 1000-node fixture < 2 ms").

### 8.2 Overlay HUD (no text needed — phase P3)

Mirror the devtools overlay style: a stack of coloured bars in a
corner of the surface, one bar per stage, length proportional to
duration. Single-quad-per-bar — works on the existing renderer.

A 1-pixel-wide column per historical frame builds a tiny stripe-
chart of the last 240 frames. Solid red if `cpu_total_ms > 16.6`.

### 8.3 Self-hosted panel (gated on engine M5 — text)

Same approach as `devtools.md` §5.3: the panel is a wgpu-html
document authored by the profiler crate. The host docks it next
to the page (splitter), and the engine paints both into the same
display list.

- **Timeline**: per-frame stacked bar of stage durations, last N
  frames horizontally, rendered as a row of `<div>`s with widths
  driven by the ring buffer. Hover a bar → tooltip with exact ns
  values and counters.
- **Hot stages**: top-K spans by self-time (descending) for the
  current frame, as a `<table>`.
- **Counters**: tiny inline sparkline per counter, drawn as a row
  of fixed-height bars (each a `<div>` with a computed width and
  background).
- **Capture**: a button that calls `dump_trace()` to a chosen
  `.json` path.

Engine prerequisites the panel forces (shared with devtools D5):
text rendering (M5) and vertical overflow scrolling on a
containing block.

### 8.4 Trace export

`dump_trace()` writes Chrome trace-event JSON:

```json
{
  "traceEvents": [
    {"name":"frame","cat":"cpu","ph":"X","ts":0,"dur":4210,"pid":1,"tid":1},
    {"name":"parse","cat":"cpu","ph":"X","ts":12,"dur":120,"pid":1,"tid":1},
    {"name":"cascade","cat":"cpu","ph":"X","ts":140,"dur":340,"pid":1,"tid":1},
    {"name":"render_pass","cat":"gpu","ph":"X","ts":3200,"dur":1040,"pid":1,"tid":2},
    {"name":"quads","ph":"C","ts":4210,"args":{"count":903},"pid":1,"tid":1}
  ]
}
```

CPU spans live on `tid:1`, GPU spans on `tid:2`. Counters use
`ph:"C"` so they appear as line graphs in the trace viewer.

## 9. Performance / threading

- Per-scope cost: one `Instant::now()` (≈ 20–80 ns on Windows) plus
  a `Vec::push` of `Span`. With ~20 spans/frame, < 5 µs total.
- When disabled: a single `Option::is_some` branch per scope macro;
  the body is dead-code-eliminated. Measured target: < 50 ns/frame
  overhead when off.
- Ring buffer is one heap allocation (size = `capacity *
  sizeof::<FrameRecord>()`). With `capacity=240`, ~1–2 MB.
- All profiler state lives on the host thread that owns the
  renderer. No locks. No background thread (trace dump is
  synchronous and called explicitly).

## 10. Phases

Each phase ends in something the demo / a test can use.

### P1 — Skeleton + CPU stage timing

> **Note:** The bare functionality of P1 already exists inline in
> `wgpu-html/src/lib.rs` (`PipelineTimings`) and
> `wgpu-html-demo/src/main.rs` (`ProfileWindow`). P1 is about
> extracting this into a proper library crate so any host can use it.

- New crate `wgpu-html-profiler`. `Profiler::new`, `enable`,
  `frame_begin / frame_end`, `scope`, `last_frame`,
  `summary_string`.
- Engine crates take `Option<&mut Profiler>` on the entry points
  used by `wgpu-html-demo`: `parse`, `cascade`, `layout`, `paint`.
- Demo binding: `Ctrl+P` toggles profiling; when on, prints
  `summary_string()` to stderr once a second (matches current behaviour).
- No GPU timing, no counters, no panel.

### P2 — Counters + display-list / node stats

- `Tree::node_count`, `LayoutBox::box_count` (cached).
- `Profiler::counter` records each per-frame.
- Renderer reports `quads`, `vertex_bytes`, `index_bytes`,
  `draw_calls` via the same `last_frame_ms` / stat surface added in
  devtools D2.
- `summary_string` includes the counter line.

### P3 — Overlay HUD

- `Profiler::paint_overlay(&LayoutBox /* viewport */, &mut DisplayList)`.
- Stage bars + last-240-frames stripe in a corner.
- Demo binding: `Ctrl+Shift+P` toggles HUD vs. console-only.

### P4 — GPU timing

- Renderer wires `wgpu::QuerySet` behind a feature check.
- `Profiler::enable_gpu`, `gpu_pass_begin / _end`,
  `resolve_gpu_timings`.
- One-frame-lag readback. Falls back gracefully when the feature
  is unsupported.
- HUD gains a separate GPU bar.

### P5 — Trace export

- `Profiler::dump_trace(path)` and `write_trace(W)`.
- Demo binding: `Ctrl+Shift+T` dumps the current ring buffer to
  `./trace-{timestamp}.json`.
- Test: load the JSON, parse, assert it has the expected stage
  names and at least one frame.

### P6 — Sub-stage breakdowns

- `parse`: `tokenize`, `build_tree`.
- `cascade`: `gather_rules`, `match`, `merge`.
- `layout`: `block`, `flex`, `measure`, `reposition`.
- `paint`: `bg`, `borders`, `radii`.
- Each is a nested `prof_scope!` inside the corresponding crate.
- Trace JSON nests via the standard `ph:"X"` overlap rules.

### P7 — Self-hosted panel (depends on engine M5)

- `wgpu-html-profiler/panel` feature.
- Panel ships as static HTML+CSS templates inside the crate; the
  profiler patches dynamic regions per frame using the same
  `Element` mutation path the inspector uses (see `devtools.md`
  §3 "Self-hosted UI").
- Timeline + hot-stages + counter sparklines, all built from
  `<div>` / `<table>` nodes.
- Capture button → `dump_trace`.
- Blocked on engine M5 because the panel needs text. Until then,
  the console summary (8.1) and HUD (8.2) cover the same ground
  with no text glyphs.

### P8 — Budget assertions for tests

- `Profiler::assert_under(stage, max_ns)` panics if the last frame
  exceeds the budget. Useful in `cargo test` to lock in
  regressions.
- Optional `--features wgpu-html-profiler/strict` flips selected
  budgets to hard failures in CI.

### P9 — Headless capture mode

- `Profiler::run_for(frames, callback)` — drives the supplied
  closure exactly N times with profiling on, then returns the
  ring buffer. Lets a benchmark binary collect a clean trace
  without a window.

## 11. Open questions

- **Sampling vs. instrumentation crossover**: a true sampling
  profiler (e.g., `tracy`) might eventually be a better fit for
  hot inner loops. The current spec is instrumentation-only —
  if we cross into "we need stack samples", we wire in tracy
  alongside, not replace.
- **Multi-threading**: parser / cascade are single-threaded today.
  When we parallelise, `Profiler` needs a `tid` per `Span` and a
  thread-local current-frame builder. Out of scope here.
- **Allocation counters**: a global allocator wrapper that
  `counter("alloc_bytes", …)` per frame would be useful but
  introduces a global cost. Punt to a follow-up spec.
- **Long-frame capture**: should `dump_trace` rotate, keeping only
  the worst N frames instead of the most recent? Not in v1; a
  later phase can add a `keep_worst(n)` mode.
- **Wallclock vs. monotonic**: spans use `Instant` (monotonic). The
  trace JSON expects µs since some epoch — we use
  `frame_begin_ns` of the first captured frame as zero.

---

## Summary

P1–P3 are buildable today: the engine just needs `Option<&mut
Profiler>` plumbed through the four CPU stages, plus a
`node_count` / `box_count` helper. P4 (GPU timing) sits on the
existing `wgpu::Queue` / `Device` the renderer already owns, and
P5 (trace export) is a pure `serde_json::to_writer` over the ring
buffer. The panel (P7) and budget asserts (P8) are independent
follow-ons. The profiler is **the** answer for "the last frame
took 30 ms — where did it go?", in the same way devtools answers
"why does this `<div>` look wrong?".
