//! Optional per-frame profiler that lives on [`super::Tree`].
//!
//! When `tree.profiler` is `Some`, the cascade → layout → paint
//! pipeline automatically records and flushes each stage's
//! wall-clock duration — no manual reading at call sites.
//!
//! Interior mutability (`Mutex`) lets pipeline functions that
//! receive `&Tree` write entries without requiring `&mut Tree`.

use std::sync::Mutex;
use std::time::Duration;

/// One named timing sample.
#[derive(Debug, Clone)]
pub struct ProfileEntry {
    pub label: &'static str,
    pub duration: Duration,
}

/// Collects [`ProfileEntry`] samples for one frame and flushes
/// them automatically at the end of each pipeline pass.
///
/// Thread-safe: the inner `Mutex` allows recording from code paths
/// that only have `&Tree` (e.g. cascade, which borrows the tree
/// immutably).
#[derive(Debug, Default)]
pub struct Profiler {
    entries: Mutex<Vec<ProfileEntry>>,
    /// Optional tag prepended to log lines (e.g. `"devtools"`).
    tag: Option<&'static str>,
}

impl Clone for Profiler {
    fn clone(&self) -> Self {
        Self {
            entries: Mutex::new(
                self.entries
                    .lock()
                    .map(|v| v.clone())
                    .unwrap_or_default(),
            ),
            tag: self.tag,
        }
    }
}

impl Profiler {
    /// Create a profiler with no tag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a profiler with a tag prepended to each log line.
    /// e.g. `Profiler::tagged("devtools")` logs `[devtools] cascade: 0.42ms`.
    pub fn tagged(tag: &'static str) -> Self {
        Self {
            entries: Mutex::new(Vec::new()),
            tag: Some(tag),
        }
    }

    /// Remove all entries. Called at the start of each frame.
    pub fn clear(&self) {
        if let Ok(mut v) = self.entries.lock() {
            v.clear();
        }
    }

    /// Append a timing sample.
    pub fn record(&self, label: &'static str, duration: Duration) {
        if let Ok(mut v) = self.entries.lock() {
            v.push(ProfileEntry { label, duration });
        }
    }

    /// Snapshot of all entries recorded since the last `clear`.
    pub fn entries(&self) -> Vec<ProfileEntry> {
        self.entries
            .lock()
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Print all entries to stderr and clear. Called automatically
    /// by the pipeline at the end of each frame.
    pub fn flush(&self) {
        let entries = if let Ok(mut v) = self.entries.lock() {
            std::mem::take(&mut *v)
        } else {
            return;
        };
        if entries.is_empty() {
            return;
        }
        let prefix = match self.tag {
            Some(t) => format!("[{t}] "),
            None => String::new(),
        };
        for e in &entries {
            eprintln!(
                "{prefix}{}: {:.2}ms",
                e.label,
                e.duration.as_secs_f64() * 1000.0,
            );
        }
    }
}
