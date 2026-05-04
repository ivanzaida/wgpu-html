//! Optional per-frame profiler that lives on [`super::Tree`].
//!
//! When `tree.profiler` is `Some`, the cascade → layout → paint
//! pipeline records each stage's wall-clock duration into a
//! fixed-capacity ring buffer. The host can read per-frame
//! summaries, dump Chrome trace-event JSON, or paint an overlay
//! HUD — all through the same profiler handle.
//!
//! Interior mutability (`Mutex`) lets pipeline functions that
//! receive `&Tree` write entries without requiring `&mut Tree`.

use std::{
  collections::{HashMap, VecDeque},
  fmt::Write,
  sync::Mutex,
  time::{Duration, Instant},
};

// ── Core types ──────────────────────────────────────────────────────────

/// Interned label identifier. Index into the profiler's label table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LabelId(u16);

/// Whether a span was measured on the CPU or GPU.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanCategory {
  Cpu,
  Gpu,
}

/// One measured time span within a frame.
#[derive(Debug, Clone)]
pub struct Span {
  pub name: LabelId,
  /// Nanoseconds since the profiler's epoch (first `frame_begin`).
  pub start_ns: u64,
  /// Nanoseconds since the profiler's epoch (first `frame_begin`).
  pub end_ns: u64,
  /// Index into `FrameRecord::spans` of the parent span, if nested.
  pub parent: Option<u16>,
  pub category: SpanCategory,
}

impl Span {
  fn duration_ns(&self) -> u64 {
    self.end_ns.saturating_sub(self.start_ns)
  }
}

/// A zero-duration marker (e.g. "viewport_resize", "font_reload").
#[derive(Debug, Clone)]
pub struct Event {
  pub name: LabelId,
  pub timestamp_ns: u64,
}

/// All profiling data for a single frame.
#[derive(Debug, Clone)]
pub struct FrameRecord {
  pub frame_index: u64,
  /// Nanoseconds since epoch when this frame started.
  pub frame_start_ns: u64,
  /// Spans in creation order. Nested via [`Span::parent`].
  pub spans: Vec<Span>,
  /// Counter samples: (label, value).
  pub counters: Vec<(LabelId, i64)>,
  /// Zero-duration event markers.
  pub events: Vec<Event>,
}

impl FrameRecord {
  /// Total CPU time across all spans (self-time, no double-counting children).
  fn cpu_self_time_ns(&self) -> u64 {
    self.spans.iter().map(|s| s.duration_ns()).sum::<u64>()
      - self
        .spans
        .iter()
        .filter_map(|s| s.parent.map(|p| self.spans[p as usize].duration_ns()))
        .sum::<u64>()
  }
}

// ── Ring buffer ─────────────────────────────────────────────────────────

/// Fixed-capacity ring buffer storing the last N frames.
#[derive(Debug, Clone)]
pub struct RingBuffer<const N: usize> {
  buf: VecDeque<FrameRecord>,
  total_frames: u64,
}

impl<const N: usize> RingBuffer<N> {
  pub fn new() -> Self {
    Self {
      buf: VecDeque::with_capacity(N),
      total_frames: 0,
    }
  }

  pub fn push(&mut self, record: FrameRecord) {
    if self.buf.len() >= N {
      self.buf.pop_front();
    }
    self.buf.push_back(record);
    self.total_frames += 1;
  }

  pub fn last(&self) -> Option<&FrameRecord> {
    self.buf.back()
  }

  pub fn iter(&self) -> impl Iterator<Item = &FrameRecord> {
    self.buf.iter()
  }

  pub fn len(&self) -> usize {
    self.buf.len()
  }

  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.buf.is_empty()
  }

  pub fn total_frames(&self) -> u64 {
    self.total_frames
  }
}

impl<const N: usize> Default for RingBuffer<N> {
  fn default() -> Self {
    Self::new()
  }
}

// ── ProfileEntry (backward compat) ──────────────────────────────────────

/// One named timing sample. Retained for backward compatibility with
/// the pre-ring-buffer API. New code should use [`Span`] through
/// [`Profiler::scope`].
#[derive(Debug, Clone)]
pub struct ProfileEntry {
  pub label: String,
  pub duration: Duration,
}

// ── Profiler ────────────────────────────────────────────────────────────

const DEFAULT_CAPACITY: usize = 240;

/// Profiler inner state behind a `Mutex` — all methods use `&self`
/// so the profiler can sit on [`super::Tree`] and be written to
/// from `&Tree` borrows.
#[derive(Debug)]
struct Inner {
  enabled: bool,
  history: RingBuffer<DEFAULT_CAPACITY>,
  /// Spans being built for the current frame.
  current_spans: Vec<Span>,
  /// Stack of span indices currently open (for parent tracking).
  span_stack: Vec<u16>,
  /// Counter values for the current frame.
  current_counters: Vec<(LabelId, i64)>,
  /// Events for the current frame.
  current_events: Vec<Event>,
  /// [`Instant`] when the current frame started, if inside a frame.
  frame_start: Option<Instant>,
  /// Monotonic frame counter.
  frame_index: u64,
  /// Epoch for nanosecond timestamps. Set on first `frame_begin`.
  epoch: Option<Instant>,
  /// String interner.
  labels: Vec<String>,
  interner: HashMap<String, LabelId>,
  /// Optional tag prepended to log lines.
  tag: Option<&'static str>,
  /// When set, `summary_string()` and `flush()` only print frames
  /// whose total CPU time exceeds this threshold (in ns).
  alert_threshold_ns: Option<u64>,
}

impl Inner {
  fn now_ns(&self) -> u64 {
    let now = Instant::now();
    self.epoch.map(|e| now.duration_since(e).as_nanos() as u64).unwrap_or(0)
  }

  fn intern(&mut self, name: &str) -> LabelId {
    if let Some(&id) = self.interner.get(name) {
      return id;
    }
    let id = LabelId(self.labels.len() as u16);
    self.labels.push(name.to_owned());
    self.interner.insert(name.to_owned(), id);
    id
  }

  fn label_str(&self, id: LabelId) -> &str {
    self.labels.get(id.0 as usize).map(|s| s.as_str()).unwrap_or("?")
  }
}

/// Collects per-frame profile spans, counters, and events into a
/// fixed-capacity ring buffer. All methods take `&self` — interior
/// mutability is via a `Mutex`.
///
/// Thread-safe but single-threaded in practice (the host thread
/// owns the renderer and the tree).
#[derive(Debug, Clone)]
pub struct Profiler {
  inner: std::sync::Arc<Mutex<Inner>>,
}

// Manual Clone uses Arc clone (shallow, shares the same inner state).
// This is needed so `tree.profiler = Some(Profiler::tagged(...))` works
// and the profiler can be cheaply passed around.

impl Profiler {
  /// Create a disabled profiler with default capacity (240 frames).
  pub fn new() -> Self {
    Self {
      inner: std::sync::Arc::new(Mutex::new(Inner {
        enabled: false,
        history: RingBuffer::new(),
        current_spans: Vec::new(),
        span_stack: Vec::new(),
        current_counters: Vec::new(),
        current_events: Vec::new(),
        frame_start: None,
        frame_index: 0,
        epoch: None,
        labels: Vec::new(),
        interner: HashMap::new(),
        tag: None,
        alert_threshold_ns: None,
      })),
    }
  }

  /// Create a profiler with a tag prepended to each log line.
  /// e.g. `Profiler::tagged("devtools")` logs `[devtools] cascade: 0.42ms`.
  pub fn tagged(tag: &'static str) -> Self {
    let p = Self::new();
    if let Ok(mut inner) = p.inner.lock() {
      inner.tag = Some(tag);
    }
    p
  }

  // ── Enable / disable ────────────────────────────────────────────────

  /// Enable or disable profiling. When disabled, all recording methods
  /// are no-ops and the ring buffer is not updated.
  pub fn set_enabled(&self, on: bool) {
    if let Ok(mut inner) = self.inner.lock() {
      inner.enabled = on;
    }
  }

  /// Shortcut to enable profiling.
  pub fn enable(&self) {
    self.set_enabled(true);
  }

  /// Shortcut to disable profiling.
  pub fn disable(&self) {
    self.set_enabled(false);
  }

  /// Returns `true` when profiling is active.
  pub fn is_enabled(&self) -> bool {
    self.inner.lock().map(|i| i.enabled).unwrap_or(false)
  }

  /// Call [`frame_begin`](Self::frame_begin) only if we are not
  /// already inside a frame. Safe to call at the start of nested
  /// pipeline functions (e.g. `compute_layout_profiled` called
  /// from `paint_tree_cached`).
  pub fn ensure_frame_begin(&self) {
    let already_in_frame = self.inner.lock().map(|i| i.frame_start.is_some()).unwrap_or(false);
    if !already_in_frame {
      self.frame_begin();
    }
  }

  // ── Alert threshold ─────────────────────────────────────────────────

  /// Set a CPU-time threshold (in nanoseconds). When set,
  /// [`summary_string`](Self::summary_string) and [`flush`](Self::flush)
  /// only produce output for frames exceeding this threshold.
  /// Pass `None` to always output (default).
  pub fn set_alert_threshold(&self, threshold_ns: Option<u64>) {
    if let Ok(mut inner) = self.inner.lock() {
      inner.alert_threshold_ns = threshold_ns;
    }
  }

  // ── Frame lifecycle ─────────────────────────────────────────────────

  /// Begin a new frame. Must be called once per frame before any
  /// [`scope`](Self::scope) or [`counter`](Self::counter) calls.
  /// Finalises the previous frame's record into the ring buffer.
  pub fn frame_begin(&self) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    if !inner.enabled {
      return;
    }

    // Finalise the previous frame.
    if inner.frame_start.is_some() {
      let record = FrameRecord {
        frame_index: inner.frame_index,
        frame_start_ns: inner.now_ns(),
        spans: std::mem::take(&mut inner.current_spans),
        counters: std::mem::take(&mut inner.current_counters),
        events: std::mem::take(&mut inner.current_events),
      };
      inner.history.push(record);
    }

    // Set the epoch on the first frame.
    if inner.epoch.is_none() {
      inner.epoch = Some(Instant::now());
    }

    inner.frame_start = Some(Instant::now());
    inner.frame_index = inner.history.total_frames();
    inner.span_stack.clear();
  }

  /// End the current frame. Finalises the record into the ring buffer.
  /// Typically called after the render pass completes.
  pub fn frame_end(&self) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    if !inner.enabled {
      return;
    }
    if inner.frame_start.is_some() {
      let record = FrameRecord {
        frame_index: inner.frame_index,
        frame_start_ns: inner.now_ns(),
        spans: std::mem::take(&mut inner.current_spans),
        counters: std::mem::take(&mut inner.current_counters),
        events: std::mem::take(&mut inner.current_events),
      };
      inner.history.push(record);
      inner.frame_start = None;
      inner.span_stack.clear();
    }
  }

  // ── Scopes (RAII spans) ─────────────────────────────────────────────

  /// Begin a named span. Returns a guard that records the end time on
  /// drop. Nested scopes are tracked via a parent stack — the trace
  /// tree is reconstructed from `Span::parent`.
  ///
  /// When the profiler is disabled, returns a no-op `()` guard.
  pub fn scope(&self, name: &str) -> ScopeGuard<'_> {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return ScopeGuard::disabled(),
    };
    if !inner.enabled {
      return ScopeGuard::disabled();
    }
    let span_idx = inner.current_spans.len() as u16;
    let parent = inner.span_stack.last().copied();
    let label_id = inner.intern(name);
    let start_ns = inner.now_ns();
    inner.current_spans.push(Span {
      name: label_id,
      start_ns,
      end_ns: 0, // filled on drop
      parent,
      category: SpanCategory::Cpu,
    });
    inner.span_stack.push(span_idx);
    ScopeGuard::active(self)
  }

  fn scope_end(&self) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    let now = inner.now_ns();
    if let Some(idx) = inner.span_stack.pop() {
      if let Some(span) = inner.current_spans.get_mut(idx as usize) {
        span.end_ns = now;
      }
    }
  }

  /// Begin a named span manually (for non-RAII paths). Returns a
  /// [`SpanId`] that must be passed to [`end_span`](Self::end_span).
  pub fn begin_span(&self, name: &str) -> SpanId {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return SpanId(u16::MAX),
    };
    if !inner.enabled {
      return SpanId(u16::MAX);
    }
    let span_idx = inner.current_spans.len() as u16;
    let parent = inner.span_stack.last().copied();
    let label_id = inner.intern(name);
    let start_ns = inner.now_ns();
    inner.current_spans.push(Span {
      name: label_id,
      start_ns,
      end_ns: 0,
      parent,
      category: SpanCategory::Cpu,
    });
    inner.span_stack.push(span_idx);
    SpanId(span_idx)
  }

  /// End the span identified by `id`. Pair with [`begin_span`](Self::begin_span).
  pub fn end_span(&self, id: SpanId) {
    if id.0 == u16::MAX {
      return;
    }
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    let now = inner.now_ns();
    // Pop from stack — the top should match `id`.
    if let Some(top) = inner.span_stack.last().copied() {
      if top == id.0 {
        inner.span_stack.pop();
      }
    }
    if let Some(span) = inner.current_spans.get_mut(id.0 as usize) {
      span.end_ns = now;
    }
  }

  // ── Counters & events ───────────────────────────────────────────────

  /// Record a scalar counter value for the current frame.
  pub fn counter(&self, name: &str, value: i64) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    if !inner.enabled {
      return;
    }
    let id = inner.intern(name);
    inner.current_counters.push((id, value));
  }

  /// Record a zero-duration event marker at the current timestamp.
  pub fn event(&self, name: &str) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    if !inner.enabled {
      return;
    }
    let id = inner.intern(name);
    let ts = inner.now_ns();
    inner.current_events.push(Event {
      name: id,
      timestamp_ns: ts,
    });
  }

  // ── Read-out ────────────────────────────────────────────────────────

  /// The most recently completed frame record, if any.
  pub fn last_frame(&self) -> Option<FrameRecord> {
    self.inner.lock().ok()?.history.last().cloned()
  }

  /// Iterator over all stored frame records (oldest first).
  pub fn frame_history(&self) -> Vec<FrameRecord> {
    self
      .inner
      .lock()
      .map(|i| i.history.iter().cloned().collect())
      .unwrap_or_default()
  }

  /// Total frames stored in the ring buffer.
  pub fn history_len(&self) -> usize {
    self.inner.lock().map(|i| i.history.len()).unwrap_or(0)
  }

  /// Total frames ever recorded (including overwritten ones).
  pub fn total_frames(&self) -> u64 {
    self.inner.lock().map(|i| i.history.total_frames()).unwrap_or(0)
  }

  /// Look up a label string by its interned ID.
  pub fn label(&self, id: LabelId) -> String {
    self
      .inner
      .lock()
      .ok()
      .and_then(|i| i.labels.get(id.0 as usize).cloned())
      .unwrap_or_else(|| "?".to_owned())
  }

  // ── Console summary ─────────────────────────────────────────────────

  /// Build a human-readable text block summarising the last frame
  /// (or returns `None` if the ring buffer is empty). Includes:
  ///
  /// - Frame index, total CPU time, FPS estimate
  /// - Per-span durations with proportional bar chart, indented by parent
  /// - Counter line
  ///
  /// When an alert threshold is set via [`set_alert_threshold`],
  /// returns `None` for frames below the threshold.
  pub fn summary_string(&self) -> Option<String> {
    let inner = self.inner.lock().ok()?;
    let frame = inner.history.last()?;
    let total_ns = frame.cpu_self_time_ns();

    // Alert threshold check.
    if let Some(threshold) = inner.alert_threshold_ns {
      if total_ns < threshold {
        return None;
      }
    }

    let total_ms = total_ns as f64 / 1_000_000.0;
    let max_ns = frame.spans.iter().map(|s| s.duration_ns()).max().unwrap_or(1).max(1);

    // Estimate FPS from the last two frames.
    let fps = if inner.history.len() >= 2 {
      let prev = &inner.history.buf[inner.history.len() - 2];
      let delta_ns = frame.frame_start_ns.saturating_sub(prev.frame_start_ns);
      if delta_ns > 0 {
        1_000_000_000.0 / delta_ns as f64
      } else {
        0.0
      }
    } else {
      0.0
    };

    let prefix = match inner.tag {
      Some(t) => format!("[{t}] "),
      None => String::new(),
    };

    let mut out = String::new();
    let _ = writeln!(
      out,
      "{}frame {}  total {:.2} ms  fps {:.1}",
      prefix, frame.frame_index, total_ms, fps
    );

    // Top-level spans first, then nested.
    let top_spans: Vec<usize> = frame
      .spans
      .iter()
      .enumerate()
      .filter(|(_, s)| s.parent.is_none())
      .map(|(i, _)| i)
      .collect();
    for idx in &top_spans {
      write_span_tree(&frame.spans, *idx, &inner.labels, max_ns, 0, &mut out);
    }

    // Counters.
    if !frame.counters.is_empty() {
      out.push_str("counters: ");
      for (i, (id, val)) in frame.counters.iter().enumerate() {
        if i > 0 {
          out.push_str("  ");
        }
        let name = inner.label_str(*id);
        let _ = write!(out, "{name}={val}");
      }
      out.push('\n');
    }

    Some(out)
  }
}

// ── Backward-compat shims ───────────────────────────────────────────────

impl Profiler {
  /// Remove all entries. Called at the start of each frame.
  /// **Deprecated:** use [`frame_begin`](Self::frame_begin) instead.
  pub fn clear(&self) {
    self.frame_begin();
  }

  /// Append a timing sample.
  /// **Deprecated:** use [`scope`](Self::scope) instead.
  pub fn record(&self, label: &'static str, duration: Duration) {
    let mut inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };
    if !inner.enabled {
      return;
    }
    let label_id = inner.intern(label);
    let end_ns = inner.now_ns();
    let start_ns = end_ns.saturating_sub(duration.as_nanos() as u64);
    inner.current_spans.push(Span {
      name: label_id,
      start_ns,
      end_ns,
      parent: None,
      category: SpanCategory::Cpu,
    });
  }

  /// Snapshot of all entries recorded since the last [`clear`](Self::clear).
  pub fn entries(&self) -> Vec<ProfileEntry> {
    let inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return Vec::new(),
    };
    inner
      .current_spans
      .iter()
      .map(|s| ProfileEntry {
        label: inner.label_str(s.name).to_owned(),
        duration: Duration::from_nanos(s.duration_ns()),
      })
      .collect()
  }

  /// Print all entries to stderr and finalise the frame.
  /// **Deprecated:** use [`frame_end`](Self::frame_end) + [`summary_string`](Self::summary_string) instead.
  pub fn flush(&self) {
    // Print current entries.
    let inner = match self.inner.lock() {
      Ok(i) => i,
      Err(_) => return,
    };

    // Check alert threshold on the partial frame.
    let total_ns: u64 = inner.current_spans.iter().map(|s| s.duration_ns()).sum();
    if let Some(threshold) = inner.alert_threshold_ns {
      if total_ns < threshold {
        return;
      }
    }

    let prefix = match inner.tag {
      Some(t) => format!("[{t}] "),
      None => String::new(),
    };
    for s in &inner.current_spans {
      eprintln!(
        "{prefix}{}: {:.2}ms",
        inner.label_str(s.name),
        s.duration_ns() as f64 / 1_000_000.0,
      );
    }
  }
}

impl Default for Profiler {
  fn default() -> Self {
    Self::new()
  }
}

// ── ScopeGuard ──────────────────────────────────────────────────────────

/// Opaque handle for a manually-opened span.
#[derive(Debug, Clone, Copy)]
pub struct SpanId(u16);

/// RAII guard returned by [`Profiler::scope`]. Records the span's
/// end time on drop.
///
/// When the profiler is disabled, this is a zero-size no-op.
pub struct ScopeGuard<'a> {
  profiler: Option<&'a Profiler>,
}

impl<'a> ScopeGuard<'a> {
  fn active(profiler: &'a Profiler) -> Self {
    Self {
      profiler: Some(profiler),
    }
  }

  fn disabled() -> Self {
    Self { profiler: None }
  }
}

impl Drop for ScopeGuard<'_> {
  fn drop(&mut self) {
    if let Some(p) = self.profiler {
      p.scope_end();
    }
  }
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Write one span and its children to `out`, indented and with a
/// proportional bar.
fn write_span_tree(spans: &[Span], idx: usize, labels: &[String], max_ns: u64, depth: usize, out: &mut String) {
  let span = &spans[idx];
  let dur_ns = span.duration_ns();
  let dur_ms = dur_ns as f64 / 1_000_000.0;
  let bar_width = ((dur_ns as f64 / max_ns.max(1) as f64) * 40.0) as usize;

  let indent = "  ".repeat(depth);
  let connector = if depth > 0 { "└ " } else { "" };
  let name = labels.get(span.name.0 as usize).map(|s| s.as_str()).unwrap_or("?");
  let _ = writeln!(
    out,
    "{indent}{connector}{name:<16} {dur_ms:>6.2} ms  {}",
    "█".repeat(bar_width.min(40))
  );

  // Recursively emit children (spans whose parent == idx).
  for (child_idx, child) in spans.iter().enumerate() {
    if child.parent == Some(idx as u16) {
      write_span_tree(spans, child_idx, labels, max_ns, depth + 1, out);
    }
  }
}

// ── Convenience macro ───────────────────────────────────────────────────

/// Begin a profiler scope on an `Option<Profiler>`. Expands to a
/// no-op when the profiler is `None`.
///
/// ```ignore
/// prof_scope!(&tree.profiler, "cascade");
/// // ... cascade work ...
/// // guard dropped here, timing automatically recorded
/// ```
#[macro_export]
macro_rules! prof_scope {
  ($prof_opt:expr, $name:expr) => {
    let __prof_scope_guard = $prof_opt.as_ref().map(|p| p.scope($name));
  };
}
