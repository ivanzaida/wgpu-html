//! wgpu-html demo.
//!
//! Thin shell over [`wgpu_html_winit::WgpuHtmlWindow`]: parse the
//! HTML document, register a system font, attach a couple of
//! example callbacks, then hand the tree to the harness.
//!
//! Profiling (F9 toggle) is implemented as a [`AppHook`] so all
//! the winit / event-loop plumbing stays inside `wgpu-html-winit`.
//!
//! Built-in shortcuts (provided by the harness):
//!   F12  → save the current frame as `screenshot-<unix>.png`
//!   Esc  → quit
//!   Ctrl+A / Ctrl+C → select all + copy via the system clipboard

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use wgpu_html_tree::Tree;
use wgpu_html_winit::{
    AppHook, EventResponse, FrameTimings, HookContext, WgpuHtmlWindow, create_window,
    register_system_fonts, system_font_variants,
};

const DEFAULT_DOC: &str = include_str!("../html/flex-browser-like.html");
const DEFAULT_DOC_PATH: &str = "crates/wgpu-html-demo/html/flex-browser-like.html";

// ── Demo wiring ─────────────────────────────────────────────────────────────

/// Wire example `on_click` / `on_mouse_enter` callbacks for the
/// known demo IDs. Callbacks are intentionally silent so profiling
/// logs stay readable.
fn install_demo_callbacks(tree: &mut Tree, click_count: &Arc<AtomicUsize>) {
    let counter = click_count.clone();
    if let Some(btn) = tree.get_element_by_id("btn") {
        btn.on_click = Some(Arc::new(move |_| {
            let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
            let _ = n;
        }));
    }
    if let Some(panel) = tree.get_element_by_id("panel") {
        panel.on_mouse_enter = Some(Arc::new(|_| {}));
        panel.on_mouse_leave = Some(Arc::new(|_| {}));
        panel.on_click = Some(Arc::new(|_| {}));
    }
}

// ── Profiling hook ──────────────────────────────────────────────────────────

/// Lightweight per-frame stats: average + max ms across the
/// reporting window.
#[derive(Debug, Clone, Copy, Default)]
struct Stage {
    sum_ms: f64,
    max_ms: f64,
}

impl Stage {
    fn add(&mut self, ms: f64) {
        self.sum_ms += ms;
        self.max_ms = self.max_ms.max(ms);
    }
    fn avg(&self, n: u64) -> f64 {
        if n == 0 {
            0.0
        } else {
            self.sum_ms / n as f64
        }
    }
}

/// One-second rolling profile window.
struct Profiler {
    started_at: Instant,
    frames: u64,
    cascade: Stage,
    layout: Stage,
    paint: Stage,
    render: Stage,
    hover_moves: u64,
    hover_changed: u64,
    pointer_move: Stage,
}

impl Profiler {
    fn new() -> Self {
        Self {
            started_at: Instant::now(),
            frames: 0,
            cascade: Stage::default(),
            layout: Stage::default(),
            paint: Stage::default(),
            render: Stage::default(),
            hover_moves: 0,
            hover_changed: 0,
            pointer_move: Stage::default(),
        }
    }

    fn add_frame(&mut self, t: &FrameTimings) {
        self.frames += 1;
        self.cascade.add(t.cascade_ms);
        self.layout.add(t.layout_ms);
        self.paint.add(t.paint_ms);
        self.render.add(t.render_ms);
    }

    fn add_pointer_move(&mut self, ms: f64, changed: bool) {
        self.hover_moves += 1;
        if changed {
            self.hover_changed += 1;
        }
        self.pointer_move.add(ms);
    }

    fn take_summary_if_due(&mut self) -> Option<String> {
        if self.started_at.elapsed() < Duration::from_secs(1) {
            return None;
        }
        if self.frames == 0 && self.hover_moves == 0 {
            self.reset();
            return None;
        }
        let secs = self.started_at.elapsed().as_secs_f64().max(f64::EPSILON);
        let fps = self.frames as f64 / secs;
        let n = self.frames;
        let line = format!(
            "profile: {:.2}s frames={} fps={:.1}  cascade={:.2}/{:.2}  layout={:.2}/{:.2}  paint={:.2}/{:.2}  render={:.2}/{:.2}  hover[moves={} changed={} ptr={:.3}/{:.3}ms]",
            secs,
            n,
            fps,
            self.cascade.avg(n), self.cascade.max_ms,
            self.layout.avg(n), self.layout.max_ms,
            self.paint.avg(n), self.paint.max_ms,
            self.render.avg(n), self.render.max_ms,
            self.hover_moves,
            self.hover_changed,
            self.pointer_move.avg(self.hover_moves),
            self.pointer_move.max_ms,
        );
        self.reset();
        Some(line)
    }

    fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Per-app hook: F9 toggles profiling; while enabled, frame and
/// pointer-move stats are aggregated and a summary line is printed
/// once per second.
struct DemoHook {
    enabled: bool,
    profiler: Profiler,
}

impl DemoHook {
    fn new(profiling_enabled: bool) -> Self {
        Self {
            enabled: profiling_enabled,
            profiler: Profiler::new(),
        }
    }
}

impl AppHook for DemoHook {
    fn on_key(&mut self, _ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse {
        if event.state == ElementState::Pressed && !event.repeat {
            if let PhysicalKey::Code(KeyCode::F9) = event.physical_key {
                self.enabled = !self.enabled;
                println!(
                    "demo: profiling {}",
                    if self.enabled { "enabled" } else { "disabled" }
                );
                if !self.enabled {
                    self.profiler.reset();
                }
                // We've fully handled F9; suppress the harness's
                // (currently no-op) default for this key.
                return EventResponse::Stop;
            }
        }
        EventResponse::Continue
    }

    fn on_frame(&mut self, _ctx: HookContext<'_>, timings: &FrameTimings) {
        if !self.enabled {
            return;
        }
        self.profiler.add_frame(timings);
        if let Some(line) = self.profiler.take_summary_if_due() {
            println!("{line}");
        }
    }

    fn on_pointer_move(&mut self, _ctx: HookContext<'_>, pointer_move_ms: f64, changed: bool) {
        if !self.enabled {
            return;
        }
        self.profiler.add_pointer_move(pointer_move_ms, changed);
    }
}

// ── CLI ─────────────────────────────────────────────────────────────────────

fn print_usage(program: &str) {
    println!("Usage: {program} [--profile] [HTML_FILE]");
    println!();
    println!("If HTML_FILE is omitted, the built-in demo document is used:");
    println!("  {DEFAULT_DOC_PATH}");
    println!();
    println!("Options:");
    println!("  --profile   enable per-frame profiling logs at startup");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} --profile");
    println!("  {program} crates/wgpu-html-demo/html/flex-browser-like.html");
}

fn resolve_doc_from_args() -> Result<(String, String, bool), ExitCode> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wgpu-html-demo".to_owned());

    let mut profiling_enabled = false;
    let mut doc_arg: Option<std::ffi::OsString> = None;

    for arg in args {
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                print_usage(&program);
                return Err(ExitCode::SUCCESS);
            }
            "--profile" => profiling_enabled = true,
            _ if text.starts_with('-') => {
                eprintln!("demo: unknown flag: {text}\n");
                print_usage(&program);
                return Err(ExitCode::FAILURE);
            }
            _ => {
                if let Some(extra) = doc_arg.replace(arg) {
                    eprintln!(
                        "demo: unexpected extra argument: {}\n",
                        extra.to_string_lossy()
                    );
                    print_usage(&program);
                    return Err(ExitCode::FAILURE);
                }
            }
        }
    }

    let Some(doc_arg) = doc_arg else {
        return Ok((
            DEFAULT_DOC.to_owned(),
            format!("embedded default ({DEFAULT_DOC_PATH})"),
            profiling_enabled,
        ));
    };

    let path = PathBuf::from(doc_arg);
    let html = match std::fs::read_to_string(&path) {
        Ok(html) => html,
        Err(err) => {
            eprintln!(
                "demo: failed to read HTML document '{}': {err}",
                path.display()
            );
            return Err(ExitCode::FAILURE);
        }
    };

    Ok((html, path.display().to_string(), profiling_enabled))
}

// ── main ────────────────────────────────────────────────────────────────────

fn main() -> ExitCode {
    println!("wgpu-html demo:");
    println!("  F12  →  save current frame as screenshot-<unix>.png");
    println!("  F9   →  toggle frame profiling logs");
    println!("  Esc  →  quit");
    println!("  Ctrl+A / Ctrl+C  →  select all + copy");

    let (doc_html, doc_source, profiling_enabled) = match resolve_doc_from_args() {
        Ok(v) => v,
        Err(code) => return code,
    };
    println!("  doc  →  {doc_source}");
    if system_font_variants().is_empty() {
        eprintln!(
            "demo: no system font found at the candidate paths — text \
             will render as zero-size. Update FONT_FAMILIES in \
             wgpu-html-winit/src/fonts.rs to point at a TTF on your \
             machine."
        );
    }
    if profiling_enabled {
        eprintln!("demo: profiling enabled via --profile");
    }

    // Build the tree once, eagerly; the harness owns the borrow
    // for the duration of `run`, so anything that mutates the tree
    // (font registration, callback wiring) has to happen first.
    let click_count = Arc::new(AtomicUsize::new(0));
    let mut tree = wgpu_html::parser::parse(&doc_html);
    register_system_fonts(&mut tree, "DemoSans");
    install_demo_callbacks(&mut tree, &click_count);

    let hook = DemoHook::new(profiling_enabled);

    let result = create_window(&mut tree)
        .with_title(format!("wgpu-html demo: {doc_source}"))
        .with_size(1280, 720)
        .with_hook(hook)
        .run();

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("demo: event loop error: {err}");
            ExitCode::FAILURE
        }
    }
}

// Keep WgpuHtmlWindow in scope so the bound is observable; cargo
// would otherwise warn about an unused import in the rare path
// where create_window happens to be the only thing referenced.
#[allow(dead_code)]
fn _ensure_type_in_scope<'tree>() -> WgpuHtmlWindow<'tree> {
    unreachable!()
}
