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

use std::collections::VecDeque;
use std::io::BufRead;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use wgpu_html_devtools::Devtools;
use wgpu_html_tree::{FontFace, Tree};
use wgpu_html_winit::{
    AppHook, EventResponse, FrameTimings, HookContext, WgpuHtmlWindow, create_window,
    register_system_fonts, system_font_variants,
};

/// The Lucide icon font, embedded at compile time (ISC license).
/// Glyphs are mapped to PUA codepoints (U+E000+); use `&#xe151;`
/// in HTML to reference them.
static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Demo wiring ─────────────────────────────────────────────────────────────

/// Wire example `on_click` / `on_mouse_enter` callbacks for the
/// known demo IDs. Callbacks are intentionally silent so profiling
/// logs stay readable.
pub(crate) fn install_demo_callbacks(tree: &mut Tree, click_count: &Arc<AtomicUsize>) {
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
        if n == 0 { 0.0 } else { self.sum_ms / n as f64 }
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
            self.cascade.avg(n),
            self.cascade.max_ms,
            self.layout.avg(n),
            self.layout.max_ms,
            self.paint.avg(n),
            self.paint.max_ms,
            self.render.avg(n),
            self.render.max_ms,
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

/// Seconds since UNIX epoch; used to make screenshot filenames
/// unique without coordinating a counter between threads.
fn timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// ── Stdin command listener ──────────────────────────────────────────────────

/// Commands the user can type into the demo's stdin. The reader
/// thread pushes parsed values onto a shared queue and wakes the
/// event loop; the queue is drained inside the per-frame hook so
/// every command runs against an up-to-date layout.
#[derive(Debug)]
enum DemoCommand {
    /// `make_screenshot [selector]`. With no selector → full
    /// viewport (the existing F12 behaviour). With one → match by
    /// CSS-style compound selector (id / tag / class) and capture
    /// just that node, even if it's outside the visible viewport.
    Screenshot { selector: Option<String> },
    /// `dump_tree [selector]`. With no selector → dump the entire
    /// tree. With one → dump the subtree rooted at the first
    /// matching element. Writes JSON to a file.
    DumpTree { selector: Option<String> },
}

type CommandQueue = Arc<Mutex<VecDeque<DemoCommand>>>;

/// Detached reader: line-buffered, blocks on stdin, parses
/// `make_screenshot [selector]` and pushes the result onto
/// `commands`. Calls `window.request_redraw()` so the harness's
/// `Wait` control flow wakes up and the next `on_frame` drains the
/// queue. Unrecognised lines are reported on stderr and ignored.
fn spawn_stdin_listener(commands: CommandQueue, window: Arc<Window>) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let lock = stdin.lock();
        for line in lock.lines() {
            let Ok(line) = line else {
                break; // pipe closed / EOF / IO error
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Split on the first whitespace → command + optional argument.
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let cmd = parts.next().unwrap_or("");
            let arg = parts
                .next()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned);
            match cmd {
                "make_screenshot" => {
                    if let Ok(mut q) = commands.lock() {
                        q.push_back(DemoCommand::Screenshot { selector: arg });
                    }
                    window.request_redraw();
                }
                "dump_tree" => {
                    if let Ok(mut q) = commands.lock() {
                        q.push_back(DemoCommand::DumpTree { selector: arg });
                    }
                    window.request_redraw();
                }
                "help" | "?" => {
                    println!("commands:");
                    println!("  make_screenshot              capture the full viewport");
                    println!(
                        "  make_screenshot <selector>   capture the node matching the selector"
                    );
                    println!("  dump_tree                    dump the full DOM tree as JSON");
                    println!(
                        "  dump_tree <selector>         dump the subtree matching the selector"
                    );
                }
                _ => {
                    eprintln!("demo: unknown command `{cmd}` (try `help` for a list)");
                }
            }
        }
    });
}

/// Sanitise a selector for use as a filename fragment: keep
/// alphanumerics + dashes / underscores, drop everything else.
/// Empty input becomes `"node"`.
fn sanitise_for_filename(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "node".to_owned()
    } else {
        cleaned
    }
}

/// Per-app hook: F9 toggles profiling; while enabled, frame and
/// pointer-move stats are aggregated and a summary line is printed
/// once per second. Also drains the stdin command queue every
/// frame and dispatches `make_screenshot` commands.
struct DemoHook {
    enabled: bool,
    profiler: Profiler,
    commands: CommandQueue,
    devtools: Devtools,
    /// 2nd-level devtools inspecting the 1st devtools' tree.
    devtools_meta: Devtools,
    stdin_started: bool,
}

impl DemoHook {
    fn new(profiling_enabled: bool, devtools: Devtools) -> Self {
        let devtools_meta = Devtools::new();
        Self {
            enabled: profiling_enabled,
            profiler: Profiler::new(),
            commands: Arc::new(Mutex::new(VecDeque::new())),
            devtools,
            devtools_meta,
            stdin_started: false,
        }
    }

    fn drain_commands(&mut self, ctx: &mut HookContext<'_>) {
        let cmds: Vec<DemoCommand> = match self.commands.lock() {
            Ok(mut q) => q.drain(..).collect(),
            Err(_) => return,
        };
        for cmd in cmds {
            self.run_command(ctx, cmd);
        }
    }

    fn run_command(&mut self, ctx: &mut HookContext<'_>, cmd: DemoCommand) {
        match cmd {
            DemoCommand::Screenshot { selector: None } => {
                let path: PathBuf = format!("screenshot-viewport-{}.png", timestamp()).into();
                ctx.renderer.capture_next_frame_to(path.clone());
                ctx.window.request_redraw();
                println!("demo: queued viewport screenshot → {}", path.display());
            }
            DemoCommand::DumpTree { selector } => {
                self.run_dump_tree(ctx, selector);
            }
            DemoCommand::Screenshot {
                selector: Some(sel),
            } => {
                // Resolve the DOM path first so we can fail fast
                // with a useful diagnostic before doing any GPU work.
                let dom_path = ctx.tree.query_selector_path(sel.as_str());
                let Some(path_indices) = dom_path else {
                    eprintln!("demo: selector `{sel}` matched no element");
                    return;
                };
                let size = ctx.window.inner_size();
                let out_path: PathBuf = format!(
                    "screenshot-{}-{}.png",
                    sanitise_for_filename(&sel),
                    timestamp()
                )
                .into();
                let result = wgpu_html::screenshot_node_to(
                    ctx.tree,
                    ctx.text_ctx,
                    ctx.image_cache,
                    ctx.renderer,
                    &path_indices,
                    size.width as f32,
                    size.height as f32,
                    1.0,
                    &out_path,
                );
                match result {
                    Ok(()) => println!(
                        "demo: saved screenshot of `{sel}` at path {path_indices:?} → {}",
                        out_path.display()
                    ),
                    Err(e) => eprintln!("demo: screenshot for `{sel}` failed: {e}"),
                }
            }
        }
    }

    fn run_dump_tree(&self, ctx: &mut HookContext<'_>, selector: Option<String>) {
        let cascaded = wgpu_html_style::cascade(ctx.tree);
        let Some(cascaded_root) = &cascaded.root else {
            eprintln!("demo: tree has no root");
            return;
        };
        let (cnode, label) = match &selector {
            None => (cascaded_root, "tree".to_owned()),
            Some(sel) => {
                let Some(path_indices) = ctx.tree.query_selector_path(sel.as_str()) else {
                    eprintln!("demo: selector `{sel}` matched no element");
                    return;
                };
                let Some(node) = cascaded_at_path(cascaded_root, &path_indices) else {
                    eprintln!("demo: path {path_indices:?} out of bounds");
                    return;
                };
                (node, sanitise_for_filename(sel))
            }
        };
        let out_path: PathBuf = format!("dump-{}-{}.json", label, timestamp()).into();
        let mut buf = String::with_capacity(8192);
        write_cascaded_json(&mut buf, cnode, 0);
        buf.push('\n');
        match std::fs::write(&out_path, &buf) {
            Ok(()) => println!(
                "demo: dumped tree → {} ({} bytes)",
                out_path.display(),
                buf.len()
            ),
            Err(e) => eprintln!("demo: dump_tree failed: {e}"),
        }
    }
}

// ── JSON tree serialiser (zero external deps) ───────────────────────────────

fn cascaded_at_path<'a>(
    root: &'a wgpu_html_style::CascadedNode,
    path: &[usize],
) -> Option<&'a wgpu_html_style::CascadedNode> {
    let mut cur = root;
    for &i in path {
        cur = cur.children.get(i)?;
    }
    Some(cur)
}

fn write_cascaded_json(out: &mut String, node: &wgpu_html_style::CascadedNode, depth: usize) {
    use std::fmt::Write;
    let indent = "  ".repeat(depth);
    let inner = "  ".repeat(depth + 1);
    out.push_str(&indent);
    out.push_str("{\n");

    let tag = node.element.tag_name();
    let _ = write!(out, "{inner}\"tag\": {}", json_str(tag));
    if let Some(id) = node.element.id() {
        let _ = write!(out, ",\n{inner}\"id\": {}", json_str(id));
    }
    if let Some(cls) = node.element.class() {
        let _ = write!(out, ",\n{inner}\"class\": {}", json_str(cls));
    }
    if let wgpu_html_tree::Element::Text(txt) = &node.element {
        let _ = write!(out, ",\n{inner}\"text\": {}", json_str(txt));
    }
    write_attrs(out, &node.element, &inner);
    write_computed_style(out, &node.style, &inner);
    if !node.children.is_empty() {
        let _ = write!(out, ",\n{inner}\"children\": [\n");
        for (i, child) in node.children.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
            }
            write_cascaded_json(out, child, depth + 2);
        }
        let _ = write!(out, "\n{inner}]");
    }
    out.push('\n');
    out.push_str(&indent);
    out.push('}');
}

fn write_attrs(out: &mut String, el: &wgpu_html_tree::Element, indent: &str) {
    use std::fmt::Write;
    let names = [
        "type",
        "name",
        "value",
        "placeholder",
        "href",
        "src",
        "alt",
        "disabled",
        "checked",
        "required",
        "readonly",
        "hidden",
        "tabindex",
        "lang",
        "dir",
        "role",
        "style",
    ];
    let mut attrs: Vec<(&str, String)> = Vec::new();
    for n in &names {
        if let Some(v) = el.attr(n) {
            attrs.push((n, v));
        }
    }
    if !attrs.is_empty() {
        let _ = write!(out, ",\n{indent}\"attrs\": {{");
        for (i, (k, v)) in attrs.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "\n{}  {}: {}", indent, json_str(k), json_str(v));
        }
        let _ = write!(out, "\n{indent}}}");
    }
}

fn write_computed_style(out: &mut String, s: &wgpu_html_models::Style, indent: &str) {
    use std::fmt::Write;
    let mut props: Vec<(&str, String)> = Vec::new();
    macro_rules! p {
        ($n:literal, $f:expr) => {
            if let Some(v) = &$f {
                props.push(($n, format!("{v:?}")));
            }
        };
    }
    p!("display", s.display);
    p!("position", s.position);
    p!("top", s.top);
    p!("right", s.right);
    p!("bottom", s.bottom);
    p!("left", s.left);
    p!("width", s.width);
    p!("height", s.height);
    p!("min-width", s.min_width);
    p!("min-height", s.min_height);
    p!("max-width", s.max_width);
    p!("max-height", s.max_height);
    p!("margin", s.margin);
    p!("margin-top", s.margin_top);
    p!("margin-right", s.margin_right);
    p!("margin-bottom", s.margin_bottom);
    p!("margin-left", s.margin_left);
    p!("padding", s.padding);
    p!("padding-top", s.padding_top);
    p!("padding-right", s.padding_right);
    p!("padding-bottom", s.padding_bottom);
    p!("padding-left", s.padding_left);
    p!("box-sizing", s.box_sizing);
    p!("color", s.color);
    p!("background", s.background);
    p!("background-color", s.background_color);
    p!("background-image", s.background_image);
    p!("background-size", s.background_size);
    p!("background-position", s.background_position);
    p!("background-repeat", s.background_repeat);
    p!("background-clip", s.background_clip);
    p!("border", s.border);
    p!("border-top-width", s.border_top_width);
    p!("border-right-width", s.border_right_width);
    p!("border-bottom-width", s.border_bottom_width);
    p!("border-left-width", s.border_left_width);
    p!("border-top-style", s.border_top_style);
    p!("border-right-style", s.border_right_style);
    p!("border-bottom-style", s.border_bottom_style);
    p!("border-left-style", s.border_left_style);
    p!("border-top-color", s.border_top_color);
    p!("border-right-color", s.border_right_color);
    p!("border-bottom-color", s.border_bottom_color);
    p!("border-left-color", s.border_left_color);
    p!("border-top-left-radius", s.border_top_left_radius);
    p!("border-top-right-radius", s.border_top_right_radius);
    p!("border-bottom-right-radius", s.border_bottom_right_radius);
    p!("border-bottom-left-radius", s.border_bottom_left_radius);
    p!("font-family", s.font_family);
    p!("font-size", s.font_size);
    p!("font-weight", s.font_weight);
    p!("font-style", s.font_style);
    p!("line-height", s.line_height);
    p!("letter-spacing", s.letter_spacing);
    p!("text-align", s.text_align);
    p!("text-decoration", s.text_decoration);
    p!("text-transform", s.text_transform);
    p!("white-space", s.white_space);
    p!("overflow", s.overflow);
    p!("overflow-x", s.overflow_x);
    p!("overflow-y", s.overflow_y);
    p!("opacity", s.opacity);
    p!("visibility", s.visibility);
    p!("z-index", s.z_index);
    p!("box-shadow", s.box_shadow);
    p!("cursor", s.cursor);
    p!("pointer-events", s.pointer_events);
    p!("user-select", s.user_select);
    p!("flex-direction", s.flex_direction);
    p!("flex-wrap", s.flex_wrap);
    p!("justify-content", s.justify_content);
    p!("align-items", s.align_items);
    p!("align-content", s.align_content);
    p!("align-self", s.align_self);
    p!("order", s.order);
    p!("gap", s.gap);
    p!("row-gap", s.row_gap);
    p!("column-gap", s.column_gap);
    p!("flex", s.flex);
    p!("flex-grow", s.flex_grow);
    p!("flex-shrink", s.flex_shrink);
    p!("flex-basis", s.flex_basis);
    p!("grid-template-columns", s.grid_template_columns);
    p!("grid-template-rows", s.grid_template_rows);
    p!("grid-auto-columns", s.grid_auto_columns);
    p!("grid-auto-rows", s.grid_auto_rows);
    p!("grid-auto-flow", s.grid_auto_flow);
    p!("grid-column", s.grid_column);
    p!("grid-column-start", s.grid_column_start);
    p!("grid-column-end", s.grid_column_end);
    p!("grid-row", s.grid_row);
    p!("grid-row-start", s.grid_row_start);
    p!("grid-row-end", s.grid_row_end);
    p!("justify-items", s.justify_items);
    p!("justify-self", s.justify_self);
    p!("transform", s.transform);
    p!("transform-origin", s.transform_origin);
    p!("transition", s.transition);
    p!("animation", s.animation);
    for (k, v) in &s.custom_properties {
        props.push((k, v.clone()));
    }
    for (k, v) in &s.deferred_longhands {
        props.push((k, v.clone()));
    }
    if !props.is_empty() {
        let _ = write!(out, ",\n{indent}\"computedStyle\": {{");
        for (i, (k, v)) in props.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "\n{}  {}: {}", indent, json_str(k), json_str(v));
        }
        let _ = write!(out, "\n{indent}}}");
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl AppHook for DemoHook {
    fn on_key(&mut self, ctx: HookContext<'_>, event: &KeyEvent) -> EventResponse {
        if event.state == ElementState::Pressed && !event.repeat {
            if let PhysicalKey::Code(code) = event.physical_key {
                match code {
                    KeyCode::F9 => {
                        self.enabled = !self.enabled;
                        println!(
                            "demo: profiling {}",
                            if self.enabled { "enabled" } else { "disabled" }
                        );
                        if !self.enabled {
                            self.profiler.reset();
                        }
                        return EventResponse::Stop;
                    }
                    KeyCode::F11 => {
                        self.devtools.toggle(ctx.event_loop);
                        return EventResponse::Stop;
                    }
                    _ => {}
                }
            }
        }
        EventResponse::Continue
    }

    fn on_frame(&mut self, mut ctx: HookContext<'_>, timings: &FrameTimings) {
        if !self.stdin_started {
            self.stdin_started = true;
            spawn_stdin_listener(self.commands.clone(), Arc::clone(ctx.window));
        }
        self.drain_commands(&mut ctx);

        // Devtools: poll for inspected tree changes, re-render.
        if self.devtools.is_enabled() {
            self.devtools.poll_and_redraw();
            // Feed the devtools tree into the meta-devtools.
            if self.devtools_meta.is_enabled() {
                self.devtools_meta
                    .update_inspected_tree(self.devtools.tree());
                self.devtools_meta.poll_and_redraw();
            }
        }

        if !self.enabled {
            return;
        }
        self.profiler.add_frame(timings);
        if let Some(line) = self.profiler.take_summary_if_due() {
            println!("{line}");
        }
    }

    fn on_idle(&mut self) {
        self.devtools.flush();
        self.devtools_meta.flush();
    }

    fn on_pointer_move(&mut self, _ctx: HookContext<'_>, pointer_move_ms: f64, changed: bool) {
        if !self.enabled {
            return;
        }
        self.profiler.add_pointer_move(pointer_move_ms, changed);
    }

    fn on_window_event(
        &mut self,
        _ctx: HookContext<'_>,
        window_id: WindowId,
        event: &WindowEvent,
    ) -> bool {
        // Meta-devtools (no further nesting).
        if self.devtools_meta.owns_window(window_id) {
            self.devtools_meta.handle_window_event(event);
            return true;
        }
        // Primary devtools — F11 toggles meta-devtools.
        if self.devtools.owns_window(window_id) {
            if let WindowEvent::KeyboardInput { event: key_ev, .. } = event {
                if key_ev.state == ElementState::Pressed
                    && !key_ev.repeat
                    && key_ev.physical_key == PhysicalKey::Code(KeyCode::F11)
                {
                    // Copy fonts before first open so text renders.
                    if !self.devtools_meta.is_enabled() {
                        for (_h, face) in self.devtools.tree().fonts.iter() {
                            self.devtools_meta.register_font(face.clone());
                        }
                        self.devtools_meta
                            .update_inspected_tree(self.devtools.tree());
                    }
                    self.devtools_meta.toggle(_ctx.event_loop);
                    return true;
                }
            }
            self.devtools.handle_window_event(event);
            return true;
        }
        false
    }
}

// ── Runner ──────────────────────────────────────────────────────────────────

pub(crate) fn run(doc_html: String, doc_source: String, profiling_enabled: bool) -> ExitCode {
    println!("wgpu-html demo:");
    println!("  renderer  →  winit");
    println!("  F12  →  save current frame as screenshot-<unix>.png");
    println!("  F9   →  toggle frame profiling logs");
    println!("  F11  →  toggle devtools window");
    println!("  Esc  →  quit");
    println!("  Ctrl+A / Ctrl+C  →  select all + copy");
    println!("  stdin →  `make_screenshot [selector]` (e.g. `make_screenshot #panel`)");
    println!("            no selector → full viewport; selector → just that node");
    println!("            (works even when the node is below the fold)");
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
    tree.register_font(FontFace::regular("lucide", Arc::from(LUCIDE_FONT)));
    install_demo_callbacks(&mut tree, &click_count);

    let devtools = Devtools::attach(&mut tree);
    let hook = DemoHook::new(profiling_enabled, devtools);

    let result = create_window(&mut tree)
        .with_title(format!("wgpu-html demo: {doc_source}"))
        .with_size(1920, 1080)
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
