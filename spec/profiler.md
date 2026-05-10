# lui — Profiler Design & Roadmap

> **Date:** 2026-05-04
> **Status:** Design phase. No profiler crate exists yet — infrastructure lives inline.
> **Companion documents:** `spec/profiler.md` (original spec), `docs/full-status.md` (engine status).

---

## 0. Current state

Three profiler pieces exist inline across the codebase:

| Piece | Location | What it does |
|-------|----------|-------------|
| `Profiler` | `crates/lui-tree/src/profiler.rs` | `Mutex<Vec<ProfileEntry>>`, `clear()` / `record()` / `flush()` — prints per-stage ms to stderr each frame |
| `PipelineTimings` | `crates/lui/src/lib.rs` | `{cascade_ms, layout_ms, paint_ms}` struct returned by `_profiled` entry points |
| `Profiler` (demo) | `crates/lui-demo/src/winit.rs` | Rolling 1-second accumulators per stage (avg+max), printed once/sec. Toggled with F9. |

**What works today:** per-frame stage timing (cascade, layout, paint, render) with
rolling stats via F9 in the demo. The `tree::Profiler` is already plumbed through
`compute_layout_profiled` and `paint_tree_returning_layout_profiled` — every
pipeline entrypoint checks `tree.profiler.is_some()`.

**What doesn't exist:** ring-buffer history, sub-stage spans with parent nesting,
counters, GPU timestamp queries, trace export, overlay HUD, or budget assertions.

---

## 1. Goals

- **Stage timing**: per frame, how long did cascade, layout, paint, and render take
  (CPU wall-clock), both top-level and with nested sub-spans.
- **GPU timing**: wall-clock duration of the render pass *on the GPU* (requires
  `TIMESTAMP_QUERY` feature — opt-in, graceful fallback).
- **Counters**: nodes, layout boxes, quads, glyphs, draw calls, vertex bytes per frame.
- **History**: a fixed-capacity ring buffer of the last N frames (default 240, ≈ 4 s at 60 Hz).
- **Console summary**: a human-readable text block per frame or per interval,
  printable to stderr/stdout from any host (no GPU dependency).
- **Overlay HUD**: coloured bars + frame-stripe chart painted as quads into the
  DisplayList — zero text dependency.
- **Trace export**: dump the ring buffer to Chrome Trace Event Format JSON for
  `chrome://tracing` / `ui.perfetto.dev`.
- **Self-hosted panel**: a lui document (HTML+CSS templates) rendered by the
  same engine, docked alongside the page. Timeline + hot-stages table + counter
  sparklines. (Gated on engine text maturity — Phase 7.)
- **Budget assertions**: `assert_under(stage, max_us)` in tests to catch regressions
  in CI.
- **Zero cost when off**: a single `Option::is_some` branch per instrumented site;
  dead-code-eliminated by the compiler. Target: < 50 ns/frame overhead when disabled.

## 2. Non-goals

- No sampling profiler (no stack walks, no `perf`-style PMC). Instrumentation only.
- No allocation tracker — punt to a follow-up spec.
- No JavaScript profiler — there is no JS engine.
- No remote / wire protocol. The dump format is a file on disk.
- No flamegraph SVG generator in-tree; the user opens the JSON in an external viewer.
- No automatic regression detection beyond budget assertions (`assert_under`). The
  profiler emits data; CI decides policy.
- No multi-threaded profiling. Parser / cascade / layout are single-threaded today.
  When we parallelise, a thread-local frame builder can be added — out of scope now.

---

## 3. Architecture

### 3.1 Crate placement strategy

**Phase 1–3 (CPU timing, counters, HUD): upgrade `tree::Profiler` in-place.**

Rationale:
- The existing `tree.profiler: Option<Profiler>` field is already plumbed through
  every pipeline entry point (`compute_layout_profiled`, `paint_tree_returning_layout_profiled`).
  Cascade, layout, and paint all call `prof.record()` / `prof.flush()` through `&Tree`
  with zero changes to call-site signatures.
- Engine crates (parser, style, layout) have zero new dependencies — they continue
  instrumenting through the same `tree.profiler` field.
- `lui-tree` has no `wgpu` dependency, which is correct for CPU-only profiling.

**Phase 4 (GPU timing): introduce `lui-profiler` crate.**

This is the natural boundary. `lui-tree` cannot depend on `wgpu` types
(`Device`, `Queue`, `QuerySet`). The new crate wraps the upgraded tree profiler:

```
lui-profiler::Profiler {
    inner: lui_tree::Profiler,   // ring buffer, scopes, counters
    gpu: Option<GpuTimer>,             // wgpu::QuerySet + readback buffer
}
```

Hosts that don't need GPU timing continue using the tree profiler directly
(no `wgpu` dep pulled in). Hosts that do use the wrapper crate.

### 3.2 Three instrumentation layers

```
Layer 1: Pipeline stages (top-level spans)        ← lui/src/lib.rs
  cascade, layout, paint
  Already instrumented via tree.profiler

Layer 2: Sub-stage spans (nested, per-crate)       ← parser/style/layout crates
  tokenize, gather_rules, match, merge, block, flex, measure, repos
  Added via prof_scope!() macros

Layer 3: Frame lifecycle (harness-level)            ← lui-winit / demo
  build_buffers, submit_cpu, gpu_render_pass, present
  Ring buffer ownership + summary_string() + trace export
```

Layers 1 and 2 share the same sink — a flat `Vec<Span>` per frame in the profiler.
Layer 3 reads the completed `FrameRecord`.

### 3.3 Ring buffer — the core data structure

```
RingBuffer<FrameRecord, N=240>       // heap-allocated, fixed capacity
├─ head: u16                         // next write slot
├─ len: u16                          // frames currently stored (≤ N)
├─ total_frames: u64                 // monotonic counter

FrameRecord {
    frame_index: u64,
    frame_start_ns: u64,             // Instant::now() at frame_begin
    spans: Vec<Span>,                // flat, sorted by start_ns
    counters: Vec<(LabelId, i64)>,
    events: Vec<(LabelId, u64)>,
}

Span {
    name: LabelId,                   // interned label index
    start_ns: u64,
    end_ns: u64,
    parent: Option<u16>,             // index into FrameRecord::spans
    category: Category,              // Cpu | Gpu
}

Category: Cpu | Gpu
```

**Why `parent: Option<u16>` instead of `depth: u8`:** a depth counter cannot
reconstruct the span tree — you cannot tell whether a depth-2 span belongs to
the most recent depth-1 or some earlier one. The parent index gives exact tree
reconstruction for timeline visualisation and indented `summary_string()` output.
Cost: 2 extra bytes per span — negligible.

**Why a fixed `RingBuffer` instead of `VecDeque`:** no internal fragmentation,
each `FrameRecord` is contiguous, and the structure is `'static`-friendly (no
lifetimes on items). The buffer is a single heap allocation:
`Box<[MaybeUninit<FrameRecord>; N]>`.

### 3.4 String interner for span names

Top-level stages use `&'static str` (zero cost). Sub-spans may need dynamic
names (e.g. `format!("flex_item_{i}")` — unacceptable to lose this detail).

```
Profiler {
    // ...
    interner: HashMap<String, LabelId>,
    labels: Vec<String>,
}

LabelId(u16)   // 65535 unique labels per session — plenty

fn intern(&mut self, name: &str) -> LabelId {
    *self.interner.entry(name.to_owned())
        .or_insert_with(|| { let id = LabelId(self.labels.len() as u16);
                             self.labels.push(name.to_owned()); id })
}
```

Static strings get interned once on first use (cheap). Dynamic strings pay one
hash lookup per creation. The `scope()` method accepts both `&'static str` and
`&str` via a trait (`IntoLabel`), so call sites keep their natural form.

Cost: one hash table lookup per unique dynamic span name per session. Since
sub-span names are mostly static in a given build (< 50 unique strings), this
is < 2 µs total across the session.

### 3.5 Scope guard design

Two forms, both zero-cost when profiling is off:

```rust
// Form 1: RAII guard on an active Profiler (crate-internal use)
let _s = profiler.scope("cascade");
// records start_ns on construction, end_ns on drop

// Form 2: Macro for Option<Profiler> boundaries (pipeline entry points)
scope!(&tree.profiler, "cascade");
// expands to:
// let __scope = tree.profiler.as_ref().map(|p| p.scope("cascade"));
```

The `scope()` method returns a `ScopeGuard` with a `Drop` impl. When the profiler
is disabled, `scope()` returns `()` (unit type) and the compiler eliminates the
entire guard construction. The macro form collapses to a single `Option::map`
that returns `None` — the body is dead-code-eliminated.

For non-RAII paths (async, callbacks):

```rust
let span_id = profiler.span_begin("upload_atlas");
// ... work ...
profiler.span_end(span_id);
```

### 3.6 Integration flow per frame

```
frame_begin()                             ← prof.frame_begin()
  scope "parse"                           ← prof.scope("parse")
    scope "tokenize"
    scope "build_tree"
  scope "cascade"                         ← prof.scope("cascade")
    scope "gather_rules"
    scope "match"
    scope "merge"
  scope "layout"                          ← prof.scope("layout")
    scope "block"
    scope "flex"
    scope "grid"
    scope "measure"
  scope "paint"                           ← prof.scope("paint")
    scope "backgrounds"
    scope "borders"
    scope "glyphs"
  ── counter("nodes", n)                 ← prof.counter("nodes", node_count)
  ── counter("quads", n)                 ← prof.counter("quads", quad_count)
  scope "build_buffers"                   ← prof.scope("build_buffers")
  scope "submit_cpu"                      ← prof.scope("submit_cpu")
  ── prof.gpu_pass_begin(encoder)        ← GPU timestamp #2N   (P4)
  scope "render_pass_cpu"
  ── prof.gpu_pass_end(encoder)          ← GPU timestamp #2N+1 (P4)
  scope "present"
frame_end()                               ← prof.frame_end()
  ── prof.resolve_gpu_timings(queue)     ← read back prev frame (P4)
```

Top-level stages (`parse`, `cascade`, `layout`, `paint`) are emitted by
`crates/lui/src/lib.rs` (the facade entry points). Per-frame lifecycle
stages (`build_buffers`, `submit_cpu`, `present`) are emitted by the winit
harness. Sub-spans are emitted inside each engine crate.

## 4. What gets measured — stages

| Stage | Where emitted | Sub-spans (Phase 6) |
|-------|--------------|---------------------|
| `parse` | `lui/src/lib.rs` | `tokenize`, `build_tree` |
| `cascade` | `lui/src/lib.rs` | `gather_rules`, `match`, `merge` |
| `layout` | `lui/src/lib.rs` | `block`, `flex`, `grid`, `measure`, `repos` |
| `paint` | `lui/src/lib.rs` | `backgrounds`, `borders`, `glyphs`, `images` |
| `build_buffers` | `lui-winit` harness | — |
| `submit_cpu` | `lui-winit` harness | — |
| `render_pass_cpu` | renderer `record_ordered_commands` | — |
| `gpu_render_pass` | GPU timestamp query (P4) | — |
| `present` | `lui-winit` harness | — |

## 5. What gets measured — counters

| Counter | Source | When emitted |
|---------|--------|-------------|
| `nodes` | `Tree::node_count()` (new method) | After parse |
| `layout_boxes` | `LayoutBox` recursive walk | After layout |
| `quads` | `DisplayList::commands.len()` | After paint |
| `glyphs` | `GlyphQuad` count in display list | After paint |
| `images` | `ImageQuad` count in display list | After paint |
| `draw_calls` | Render pass count in `record_ordered_commands` | After render |
| `vertex_bytes` | GPU buffer upload size | After render |
| `surface_size` | `width × height` | Every frame |

## 6. UI surface

### 6.1 Console summary (Phase 1)

`summary_string()` returns a fixed-width text block — no text rendering needed:

```
frame 4123  total 3.82 ms  cpu 3.82 ms  fps 60.0
  cascade         0.42 ms  ████████
  └ selector      0.28 ms  █████
    └ merge       0.08 ms  ██
  layout          2.12 ms  ██████████████████████████████████████
  └ flex          1.34 ms  ████████████████████████
  paint           0.28 ms  █████
  └ glyphs        0.11 ms  ██
counters: nodes=412  boxes=388  quads=903  glyphs=218  images=3  dc=5
```

Bar widths: `fraction_of_total * 40` characters. Sub-spans indented with `└`.
When GPU timing is enabled (Phase 4), a separate `gpu_render_pass` span appears
at the bottom with its own bar and indentation.

Can be printed to stderr/stdout, piped to a log file, or asserted against in tests.

### 6.2 Overlay HUD (Phase 3)

Painted directly into the `DisplayList` — zero text dependency, works on the
existing quad pipeline:

- **Stage bars:** one coloured rectangle per top-level stage in the top-right
  corner. Width = proportion of frame budget. Heights stack vertically (8 px each,
  1 px gap). Colours: cascade=blue, layout=green, paint=yellow, gpu=red.
- **Frame stripe chart:** below the bars, one 1-pixel-wide column per historical
  frame (up to 240 columns). Each column's height = total frame time, capped at
  33 ms (two vsyncs). Columns turn red when > 16.6 ms. A horizontal line marks
  the 16.6 ms threshold.
- **Layout:** anchored to viewport top-right, 10 px inset. Translucent dark
  background behind the bars for readability.
- **Toggle:** `Ctrl+Shift+P` in the demo (F9 remains for console mode).

All geometry is single `push_quad()` calls — no text shaping, no atlas upload.

### 6.3 Self-hosted panel (Phase 7)

Gated on engine text maturity (the panel needs rendered text for labels and
tables). Modeled on `devtools.md` §3: the panel is a lui document (static
HTML+CSS templates inside the crate), the profiler patches dynamic regions per
frame via element mutation.

- **Timeline:** per-frame stacked bar of stage durations, last N frames
  horizontally, rendered as a row of `<div>`s with computed widths.
  Hover a bar → tooltip with exact ns values and counters.
- **Hot stages:** top-K spans by self-time (descending) for the current frame
  as a `<table>`.
- **Counters:** inline sparkline per counter, drawn as a row of fixed-height
  bars (each a `<div>` with computed width and background).
- **Capture button:** calls `dump_trace()` to a chosen `.json` path.

### 6.4 Trace export (Phase 5)

`dump_trace(path)` writes Chrome Trace Event Format JSON:

```json
{
  "traceEvents": [
    {"name":"frame","cat":"cpu","ph":"X","ts":0,"dur":3820,"pid":1,"tid":1},
    {"name":"cascade","cat":"cpu","ph":"X","ts":12,"dur":420,"pid":1,"tid":1},
    {"name":"selector","cat":"cpu","ph":"X","ts":12,"dur":280,"pid":1,"tid":1},
    {"name":"merge","cat":"cpu","ph":"X","ts":300,"dur":80,"pid":1,"tid":1},
    {"name":"layout","cat":"cpu","ph":"X","ts":440,"dur":2120,"pid":1,"tid":1},
    {"name":"flex","cat":"cpu","ph":"X","ts":440,"dur":1340,"pid":1,"tid":1},
    {"name":"paint","cat":"cpu","ph":"X","ts":2570,"dur":280,"pid":1,"tid":1},
    {"name":"glyphs","cat":"cpu","ph":"X","ts":2570,"dur":110,"pid":1,"tid":1},
    {"name":"render_pass","cat":"gpu","ph":"X","ts":3200,"dur":1040,"pid":1,"tid":2},
    {"name":"quads","ph":"C","ts":3820,"args":{"count":"903"},"pid":1,"tid":1},
    {"name":"glyphs","ph":"C","ts":3820,"args":{"count":"218"},"pid":1,"tid":1}
  ]
}
```

CPU spans on `tid:1`, GPU spans on `tid:2`. Nested spans use `ph:"X"` overlap
rules (parent fully contains child in time). Counters use `ph:"C"` — appear as
line graphs in the trace viewer. Timestamps use `frame_begin_ns` of the first
captured frame as epoch zero.

Demo binding: `Ctrl+Shift+T` dumps the current ring buffer to
`./trace-{timestamp}.json`.

## 7. on_when_slow mode

A mode the spec doesn't mention but real renderers implement (Chrome's "long
frame" detection, Firefox's frame delay reporter). The profiler runs silently
(no stderr output, no HUD) unless a frame exceeds a configurable threshold.

```rust
profiler.set_alert_threshold(Duration::from_millis(33)); // two vsyncs
// Profiler now:
//   - always records spans + counters (for trace export)
//   - but only prints summary_string() to stderr when total > 33 ms
//   - and only paints the HUD red-border alert when threshold exceeded
```

The alert highlights the offending spans (e.g. `» layout 2.12 ms` with `»` prefix).
Useful for catching intermittent jank without log spam during normal operation.

Threshold is `None` by default (profiler always outputs when enabled). The demo
binds `Ctrl+Shift+L` to toggle the threshold to 33 ms.

## 8. Performance budget

- **Per-scope overhead (enabled):** one `Instant::now()` (≈ 20–80 ns on Windows)
  + a `Vec::push` of `Span`. With ~20 spans/frame: < 5 µs total.
- **Per-scope overhead (disabled):** a single `Option::is_some` branch; the guard
  body is dead-code-eliminated. Target: < 50 ns/frame total.
- **Ring buffer memory:** one heap allocation, size = `capacity * sizeof(FrameRecord)`.
  With capacity=240: ~80 KB for span data (assuming 20 spans × 240 frames × 24 bytes)
  + ~20 KB for counters and events. Total ~100 KB — negligible.
- **String interner memory:** ~50 unique labels per session × avg 15 bytes = < 1 KB.
- **No locks** (after the `Mutex` in the current `tree::Profiler` is removed).
  Ring buffer access is single-threaded (the host thread that owns the renderer).
- **Trace export:** synchronous, `serde_json::to_writer` over the ring buffer.
  For 240 frames: ~200 KB of JSON — < 5 ms to write. Called explicitly, not per-frame.

## 9. Phases

Each phase ends in something the demo / a test can use. Phases 1 and 2 are
delivered together because counters are trivial once the `FrameRecord` exists.

### Phase 1 — Ring buffer + scopes + summary_string

**Crate:** Upgrade `lui-tree::Profiler` in-place.

Deliverables:
- `RingBuffer<FrameRecord, 240>` with `push()`, `iter()`, `len()`.
- `Profiler::scope(name) -> ScopeGuard` — RAII span recording with parent tracking.
- `scope!(tree.profiler, "name")` macro for `Option<Profiler>` boundaries.
- `Profiler::frame_begin()`, `frame_end()` — lifecycle management.
- `Profiler::summary_string() -> String` — indented text block with bar chart.
- String interner (`intern()`, `LabelId`) for dynamic span names.
- `Span::parent: Option<u16>` for tree reconstruction.
- Remove the `Mutex` from `Profiler` — single-threaded, no contention.
- Keep `clear()`/`record()`/`flush()` as deprecated compat shims that map to
  the new API internally.

Changes to existing code:
- `crates/lui/src/lib.rs`: wrap cascade/layout/paint in `scope!()` (replaces
  manual `Instant::now()` + `prof.record()` calls).
- `crates/lui-winit/src/window.rs`: call `prof.frame_begin()` / `frame_end()`
  around the redraw loop. Print `summary_string()` in the F9 path.

No changes to parser, style, or layout crates in Phase 1.

### Phase 2 — Counters

**Crate:** `lui-tree` (same Profiler).

Deliverables:
- `Profiler::counter(name, value: i64)` — records a scalar on the current frame.
- `Profiler::event(name)` — zero-duration marker (e.g. "viewport_resize").
- `Tree::node_count()` — recursive walk, returns `usize`.
- Helper to count `LayoutBox` instances from the layout root (free function,
  not a `LayoutBox` method — avoids pulling the profiler dependency into layout).
- `DisplayList::command_counts()` — returns `(quads, glyphs, images)`.
- `summary_string()` appends the counters line.

Counter emission points:
- After parse: `prof.counter("nodes", tree.node_count())`
- After layout: `prof.counter("layout_boxes", count_boxes(&layout))`
- After paint: `prof.counter("quads", ...)`, `prof.counter("glyphs", ...)`
- After render: `prof.counter("draw_calls", ...)` (set by harness)

### Phase 3 — Overlay HUD

**Crate:** `lui-tree` (same Profiler).

Deliverables:
- `Profiler::paint_overlay(&self, viewport: Rect, &mut DisplayList)`.
- Draws stage bars + frame-stripe chart in the viewport top-right corner.
- All geometry via `push_quad()` — zero text dependency.
- Reads from the ring buffer's last-frame record for current bars, and the
  full history for the stripe chart.
- Demo binding: `Ctrl+Shift+P` toggles HUD vs. console-only. F9 continues to
  toggle console mode independently.

### Phase 4 — GPU timing

**Crate:** New `lui-profiler` (wraps the upgraded tree profiler, adds `wgpu` dep).

Deliverables:
- `lui-profiler::Profiler` wraps `lui_tree::Profiler`.
- `GpuTimer` struct: manages `wgpu::QuerySet` + readback buffer + timestamp
  resolution calculation.
- `Profiler::enable_gpu(device, queue)` — attempts to enable GPU timing,
  returns `false` if `TIMESTAMP_QUERY` is unsupported.
- `gpu_pass_begin(encoder)` / `gpu_pass_end(encoder)` — writes timestamps into
  the query set around the render pass.
- `resolve_gpu_timings(queue)` — called one frame later; maps the readback
  buffer and stores GPU ns into the previous `FrameRecord`.
- GPU span appears in `summary_string()` and trace export on `tid:2`.
- HUD gains a separate red GPU bar.
- Graceful fallback: CPU-only profiling continues unchanged when GPU timing
  is unavailable.

The query set has `2 * MAX_INFLIGHT` slots (default 6). The one-frame lag
avoids stalling on `Queue::on_submitted_work_done`.

### Phase 5 — Trace export

**Crate:** `lui-tree` (no new deps — pure serialisation).

Deliverables:
- `Profiler::write_trace(&self, writer: impl Write) -> io::Result<()>`.
- `Profiler::dump_trace(&self, path: impl AsRef<Path>) -> io::Result<()>`.
- Chrome Trace Event Format JSON output:
  - Spans → `ph:"X"` complete events, nested via parent timestamps.
  - Counters → `ph:"C"` counter events with `args: {name: value}`.
  - GPU spans → `tid:2`, CPU spans → `tid:1`.
- Timestamps use the first captured frame's `frame_begin_ns` as epoch zero.
- Demo binding: `Ctrl+Shift+T` → `./trace-{timestamp}.json`.
- Test: parse the output JSON, assert expected stage names present, at least
  one frame with non-zero duration.

### Phase 6 — Sub-stage breakdowns

**Crates:** `lui-parser`, `lui-style`, `lui-layout`,
  `crates/lui/src/paint.rs`.

Deliverables:
- `parser`: `prof_scope!("tokenize")` around tokenizer, `prof_scope!("build_tree")`
  around tree builder.
- `style`: `prof_scope!("gather_rules")`, `prof_scope!("match")`, `prof_scope!("merge")`
  inside `cascade()`.
- `layout`: `prof_scope!("block")`, `prof_scope!("flex")`, `prof_scope!("grid")`,
  `prof_scope!("measure")`, `prof_scope!("repos")` in the layout pass.
- `paint`: `prof_scope!("backgrounds")`, `prof_scope!("borders")`,
  `prof_scope!("glyphs")`, `prof_scope!("images")` in `paint_box_in_clip()`.

Each sub-span is a `scope!(&tree.profiler, "name")` call — no new function
signatures, no trait bounds. These crates already have access to `tree.profiler`
through the `&Tree` reference they receive.

Trace export correctly nests these via the `parent: Option<u16>` field —
the JSON `ph:"X"` events for children are fully contained in the parent's
time range.

### Phase 7 — Self-hosted panel

**Crate:** `lui-profiler` with `panel` feature flag.

Gated on engine text maturity. Same architecture as `devtools` §3: the panel
is a `Tree` built from static HTML+CSS templates, then the profiler mutates
dynamic regions per frame.

- **Timeline panel:** a horizontal row of stacked divs, each representing one
  frame. Width = 1–2 px per frame. Hover triggers a tooltip overlay.
- **Hot stages panel:** a `<table>` of span names and durations, sorted by
  self-time descending. Updated every frame.
- **Counter sparklines:** a row of tiny bar charts, each bar computed from the
  ring buffer history.
- **Capture button:** an `<input type="button">` or `<button>` that calls
  `dump_trace()` on click (wired via `on_click` callback on the tree).

The panel renders into a dedicated `Tree` that lives alongside the page tree.
The host composites both `DisplayList`s into the same frame:

```rust
let page_list = paint_tree(&page_tree, ...);
let panel_list = paint_tree(&panel_tree, ...);
// Merge: panel_list is translated to the dock position (e.g. right side).
page_list.append(panel_list.translated(panel_x, panel_y));
```

### Phase 8 — Budget assertions for tests

**Crate:** `lui-tree` (the profiler is already a dev-dependency of
  `lui` tests).

Deliverables:
- `Profiler::assert_under(stage: &str, max_us: u64)` — panics if the named
  span in the last frame exceeded the budget.
- `Profiler::assert_total_under(max_us: u64)` — panics if total frame time
  exceeded the budget.
- Used in `lui` integration tests:

```rust
#[test]
fn layout_1000_nodes_under_2ms() {
    let mut prof = Profiler::new();
    prof.enable();
    tree.profiler = Some(prof.clone());
    // ... run pipeline ...
    prof.assert_under("layout", 2000); // µs
    prof.assert_total_under(5000);
}
```

- Optional `"strict"` feature on `lui` flips selected budgets to hard
  CI failures.

### Phase 9 — Headless capture mode

**Crate:** `lui-tree` (no new deps).

Deliverables:
- `Profiler::run_for(frames: usize, f: impl FnMut(usize)) -> RingBuffer<FrameRecord>`.
- Drives the closure exactly N times with profiling enabled.
- Returns the populated ring buffer — caller can then `dump_trace()` or iterate
  to extract measurements.
- Useful for benchmarks without a window:

```rust
let ring = Profiler::run_for(120, |i| {
    // ... run one frame of the pipeline ...
});
ring.write_trace(&mut File::create("bench.json")?)?;
```

## 10. Open questions

- **Sampling crossover:** a true sampling profiler (e.g. `tracy`) might
  eventually be a better fit for hot inner loops. The current design is
  instrumentation-only. If the need arises, we wire in `tracy` alongside,
  not instead of this profiler.
- **Multi-threading:** when parser/cascade/layout become parallel, the
  profiler needs thread-local frame builders and a `tid` per span. The
  current `parent: Option<u16>` model extends naturally — add a `tid: u8`
  field to `Span`. Out of scope for now.
- **Allocation tracking:** a global allocator wrapper that calls
  `prof.counter("alloc_bytes", n)` per frame. Introduces a global cost.
  Punt to a follow-up spec.
- **Long-frame capture:** should `dump_trace` rotate, keeping only the worst
  N frames instead of the most recent? Not in v1. A later phase can add
  `keep_worst(n)` mode.
- **Wall-clock vs monotonic:** spans use `Instant` (monotonic). Trace JSON
  expects µs since some epoch — we use `frame_begin_ns` of the first captured
  frame as epoch zero. Restarting the profiler resets the epoch.
- **Render pass count:** the renderer's `record_ordered_commands()` creates
  one render pass per command-kind group (quads → images → glyphs → quads → …).
  Should GPU timing measure the whole encoder span (one timestamp pair) or
  per-pass (N pairs)? The whole-encoder approach is simpler and more useful
  for "where did the frame go?" questions. If per-pass detail is needed later,
  add sub-spans within the GPU category.

## 11. Summary — what to build first

```
Phase 1 + 2 (now):    Ring buffer, scopes, counters, summary_string
                      → upgrade tree::Profiler in-place
                      → 0 new crates, 0 new deps

Phase 3 (next):       Overlay HUD bars + stripe chart
                      → 8-10 push_quad calls in paint_overlay()

Phase 4 (later):      GPU timing
                      → new lui-profiler crate (wgpu dep)
                      → QuerySet + one-frame-lag readback

Phase 5:              Chrome trace JSON export

Phase 6:              Sub-span instrumentation in parser/style/layout/paint

Phase 7:              Self-hosted panel (gated on engine text)

Phase 8:              Budget assertions for CI

Phase 9:              Headless capture for benchmarks
```
