//! wgpu-html winit demo.
//!
//! Uses [`wgpu_html_driver_winit`] for a manual event loop.

use std::{
  collections::VecDeque,
  io::BufRead,
  path::PathBuf,
  process::ExitCode,
  sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use wgpu_html_devtools::Devtools;
use wgpu_html_driver_winit::{WgpuHtml, WinitRuntime, dispatch, register_system_fonts, system_font_variants};
use wgpu_html_tree::{FontFace, Tree};
use winit::{
  application::ApplicationHandler,
  event::{ElementState, KeyEvent, WindowEvent},
  event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
  keyboard::{KeyCode, PhysicalKey},
  window::{Window, WindowId},
};

static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── Demo wiring ─────────────────────────────────────────────────────────────

pub(crate) fn install_demo_callbacks(tree: &mut Tree, click_count: &Arc<AtomicUsize>) {
  let counter = click_count.clone();
  if let Some(btn) = tree.get_element_by_id("btn") {
    btn.on_click.push(Arc::new(move |_| {
      let n = counter.fetch_add(1, Ordering::Relaxed) + 1;
      let _ = n;
    }));
  }
  if let Some(panel) = tree.get_element_by_id("panel") {
    panel.on_mouse_enter.push(Arc::new(|_| {}));
    panel.on_mouse_leave.push(Arc::new(|_| {}));
    panel.on_click.push(Arc::new(|_| {}));
  }

  let log_ids = [
    "zone-click",
    "zone-mousedown",
    "zone-mouseup",
    "zone-dblclick",
    "zone-auxclick",
    "zone-contextmenu",
    "zone-enterleave",
    "zone-mousemove",
    "zone-buttons",
    "zone-wheel",
    "zone-keydown",
    "zone-focus-input",
    "zone-input",
    "zone-checkbox",
    "zone-form",
    "zone-clipboard",
    "zone-dragsource",
    "zone-droptarget",
    "zone-activate-btn",
    "zone-activate-link",
    "zone-submit-btn",
    "zone-form-input",
  ];
  for id in log_ids {
    if let Some(el) = tree.get_element_by_id(id) {
      el.on_event.push(Arc::new(move |_ev| {}));
    }
  }

  if let Some(el) = tree.get_element_by_id("zone-dragsource") {
    el.draggable = true;
  }
}

// ── Profiling ────────────────────────────────────────────────────────────────

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

struct Stats {
  started_at: Instant,
  frames: u64,
  cascade: Stage,
  layout: Stage,
  paint: Stage,
}

impl Stats {
  fn new() -> Self {
    Self {
      started_at: Instant::now(),
      frames: 0,
      cascade: Stage::default(),
      layout: Stage::default(),
      paint: Stage::default(),
    }
  }

  fn add_frame(&mut self, cascade_ms: f64, layout_ms: f64, paint_ms: f64) {
    self.frames += 1;
    self.cascade.add(cascade_ms);
    self.layout.add(layout_ms);
    self.paint.add(paint_ms);
  }

  fn take_summary_if_due(&mut self) -> Option<String> {
    if self.started_at.elapsed() < Duration::from_secs(1) {
      return None;
    }
    if self.frames == 0 {
      self.reset();
      return None;
    }
    let secs = self.started_at.elapsed().as_secs_f64().max(f64::EPSILON);
    let fps = self.frames as f64 / secs;
    let n = self.frames;
    let line = format!(
      "profile: {:.2}s frames={} fps={:.1}  cascade={:.2}/{:.2}  layout={:.2}/{:.2}  paint={:.2}/{:.2}",
      secs,
      n,
      fps,
      self.cascade.avg(n),
      self.cascade.max_ms,
      self.layout.avg(n),
      self.layout.max_ms,
      self.paint.avg(n),
      self.paint.max_ms,
    );
    self.reset();
    Some(line)
  }

  fn reset(&mut self) {
    *self = Self::new();
  }
}

// ── Stdin commands ──────────────────────────────────────────────────────────

#[derive(Debug)]
enum DemoCommand {
  Screenshot { selector: Option<String> },
  DumpTree { selector: Option<String> },
}

type CommandQueue = Arc<Mutex<VecDeque<DemoCommand>>>;

fn spawn_stdin_listener(commands: CommandQueue, window: Arc<Window>) {
  std::thread::spawn(move || {
    let stdin = std::io::stdin();
    let lock = stdin.lock();
    for line in lock.lines() {
      let Ok(line) = line else { break };
      let trimmed = line.trim();
      if trimmed.is_empty() {
        continue;
      }
      let mut parts = trimmed.splitn(2, char::is_whitespace);
      let cmd = parts.next().unwrap_or("");
      let arg = parts.next().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned);
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
          println!("  make_screenshot [selector]");
          println!("  dump_tree [selector]");
        }
        _ => eprintln!("demo: unknown command `{cmd}` (try `help`)"),
      }
    }
  });
}

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
  if cleaned.is_empty() { "node".to_owned() } else { cleaned }
}

fn timestamp() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::SystemTime::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0)
}

// ── JSON dump ───────────────────────────────────────────────────────────────

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
  p!("width", s.width);
  p!("height", s.height);
  p!("margin", s.margin);
  p!("padding", s.padding);
  p!("color", s.color);
  p!("background-color", s.background_color);
  p!("border", s.border);
  p!("font-family", s.font_family);
  p!("font-size", s.font_size);
  p!("font-weight", s.font_weight);
  p!("overflow", s.overflow);
  p!("opacity", s.opacity);
  p!("flex-direction", s.flex_direction);
  p!("justify-content", s.justify_content);
  p!("align-items", s.align_items);
  p!("gap", s.gap);
  p!("cursor", s.cursor);
  p!("z-index", s.z_index);
  for (k, v) in &s.custom_properties {
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

// ── Runner ──────────────────────────────────────────────────────────────────

struct DemoApp {
  tree: Tree,
  rt: WinitRuntime,
  profiling: bool,
  stats: Stats,
  commands: CommandQueue,
  devtools: Option<Devtools>,
  stdin_started: bool,
  screenshot_key: Option<KeyCode>,
  exit_on_escape: bool,
  clipboard: Option<arboard::Clipboard>,
}

impl DemoApp {
  fn new(tree: Tree, rt: WinitRuntime, profiling_enabled: bool, devtools: Devtools) -> Self {
    Self {
      tree,
      rt,
      profiling: profiling_enabled,
      stats: Stats::new(),
      commands: Arc::new(Mutex::new(VecDeque::new())),
      devtools: Some(devtools),
      stdin_started: false,
      screenshot_key: Some(KeyCode::F12),
      exit_on_escape: true,
      clipboard: None,
    }
  }

  fn drain_commands(&mut self) {
    let cmds: Vec<DemoCommand> = match self.commands.lock() {
      Ok(mut q) => q.drain(..).collect(),
      Err(_) => return,
    };
    for cmd in cmds {
      self.run_command(cmd);
    }
  }

  fn run_command(&mut self, cmd: DemoCommand) {
    match cmd {
      DemoCommand::Screenshot { selector: None } => {
        let path: PathBuf = format!("screenshot-viewport-{}.png", timestamp()).into();
        self.rt.renderer.capture_next_frame_to(path.clone());
        self.rt.driver.window.request_redraw();
        println!("demo: queued viewport screenshot → {}", path.display());
      }
      DemoCommand::Screenshot { selector: Some(sel) } => {
        let dom_path = self.tree.query_selector_path(sel.as_str());
        let Some(path_indices) = dom_path else {
          eprintln!("demo: selector `{sel}` matched no element");
          return;
        };
        let size = self.rt.driver.window.inner_size();
        let out_path: PathBuf = format!("screenshot-{}-{}.png", sanitise_for_filename(&sel), timestamp()).into();
        let result = wgpu_html::screenshot_node_to(
          &self.tree,
          &mut self.rt.text_ctx,
          &mut self.rt.image_cache,
          &mut self.rt.renderer,
          &path_indices,
          size.width as f32,
          size.height as f32,
          1.0,
          &out_path,
        );
        match result {
          Ok(()) => println!("demo: saved screenshot of `{sel}` → {}", out_path.display()),
          Err(e) => eprintln!("demo: screenshot for `{sel}` failed: {e}"),
        }
      }
      DemoCommand::DumpTree { selector } => {
        let cascaded = wgpu_html_style::cascade(&self.tree);
        let Some(root) = &cascaded.root else {
          eprintln!("demo: tree has no root");
          return;
        };
        let (cnode, label) = match &selector {
          None => (root, "tree".to_owned()),
          Some(sel) => {
            let Some(path_indices) = self.tree.query_selector_path(sel.as_str()) else {
              eprintln!("demo: selector `{sel}` matched no element");
              return;
            };
            let Some(node) = cascaded_at_path(root, &path_indices) else {
              eprintln!("demo: path out of bounds");
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
          Ok(()) => println!("demo: dumped tree → {} ({} bytes)", out_path.display(), buf.len()),
          Err(e) => eprintln!("demo: dump_tree failed: {e}"),
        }
      }
    }
  }

  fn handle_key(&mut self, event_loop: &ActiveEventLoop, event: &KeyEvent) {
    if event.state != ElementState::Pressed || event.repeat {
      return;
    }
    let PhysicalKey::Code(code) = event.physical_key else {
      return;
    };

    // F9 profiling toggle
    if code == KeyCode::F9 {
      self.profiling = !self.profiling;
      println!(
        "demo: profiling {}",
        if self.profiling { "enabled" } else { "disabled" }
      );
      if !self.profiling {
        self.stats.reset();
      }
      return;
    }

    // Escape exit
    if self.exit_on_escape && code == KeyCode::Escape {
      event_loop.exit();
      return;
    }

    // Screenshot
    if self.screenshot_key == Some(code) {
      let path: PathBuf = format!("screenshot-{}.png", timestamp()).into();
      self.rt.renderer.capture_next_frame_to(path.clone());
      self.rt.driver.window.request_redraw();
      return;
    }

    // Clipboard shortcuts
    if self.tree.modifiers().ctrl {
      match code {
        KeyCode::KeyA => {
          if let Some(layout) = self.rt.layout() {
            if wgpu_html::select_all_text(&mut self.tree, layout) {
              self.rt.driver.window.request_redraw();
            }
          }
        }
        KeyCode::KeyC => {
          let _ = self.clipboard.get_or_insert_with(|| arboard::Clipboard::new().unwrap());
          if let Some(layout) = self.rt.layout() {
            if let Some(text) = wgpu_html::selected_text(&self.tree, layout) {
              if let Some(cb) = &mut self.clipboard {
                let _ = cb.set_text(text);
              }
            }
          }
        }
        _ => {}
      }
    }
  }
}

impl ApplicationHandler for DemoApp {
  fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

  fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
    // Route secondary window events to devtools.
    if window_id != self.rt.driver.window.id() {
      if let Some(devtools) = &mut self.devtools {
        if devtools.owns_window(window_id) {
          devtools.handle_window_event(&self.tree, &event);
        }
      }
      return;
    }

    match event {
      WindowEvent::CloseRequested => event_loop.exit(),

      WindowEvent::RedrawRequested => {
        // Drain stdin commands first.
        if !self.stdin_started {
          self.stdin_started = true;
          spawn_stdin_listener(self.commands.clone(), self.rt.driver.window.clone());
        }
        self.drain_commands();

        // Sync devtools.
        if let Some(devtools) = &mut self.devtools {
          devtools.poll(&self.tree, event_loop);
        }

        let timings = self.rt.render_frame(&mut self.tree);

        // Profiling.
        if self.profiling {
          self
            .stats
            .add_frame(timings.cascade_ms, timings.layout_ms, timings.paint_ms);
          if let Some(line) = self.stats.take_summary_if_due() {
            println!("{line}");
          }
          if let Some(prof) = &self.tree.profiler {
            if prof.is_enabled() {
              if let Some(summary) = prof.summary_string() {
                eprintln!("{summary}");
              }
            }
          }
        }

        // Schedule next wake-up.
        if self.rt.image_cache.has_pending() {
          event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100)));
        } else if self.rt.image_cache.has_animated() {
          event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50)));
        } else if self.tree.interaction.edit_cursor.is_some() {
          let elapsed = self.tree.interaction.caret_blink_epoch.elapsed().as_millis() as u64;
          let next = 500u64.saturating_sub(elapsed % 500).max(16);
          event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(next)));
        } else {
          event_loop.set_control_flow(ControlFlow::Wait);
        }
      }

      // Keyboard is handled before dispatch to intercept F9/Esc/etc.
      WindowEvent::KeyboardInput {
        event: ref key_event, ..
      } => {
        self.handle_key(event_loop, key_event);
        if dispatch(&event, &mut self.rt, &mut self.tree) {
          self.rt.driver.window.request_redraw();
        }
      }

      other => {
        if dispatch(&other, &mut self.rt, &mut self.tree) {
          self.rt.driver.window.request_redraw();
        }
      }
    }
  }

  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    // Caret blink wake-up.
    if self.tree.interaction.edit_cursor.is_some() {
      let elapsed = self.tree.interaction.caret_blink_epoch.elapsed().as_millis() as u64;
      if 500u64.saturating_sub(elapsed % 500) == 0 {
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now()));
      }
    }
  }
}

// ── Entry point ─────────────────────────────────────────────────────────────

pub(crate) fn run(doc_html: String, doc_source: String, profiling_enabled: bool) -> ExitCode {
  println!("wgpu-html demo:");
  println!("  renderer  →  winit");
  println!("  F12  →  screenshot");
  println!("  F9   →  toggle profiling");
  println!("  F11  →  toggle devtools");
  println!("  Esc  →  quit");
  println!("  doc  →  {doc_source}");
  if system_font_variants().is_empty() {
    eprintln!("demo: no system font found — text will render as zero-size");
  }
  if profiling_enabled {
    eprintln!("demo: profiling enabled via --profile");
  }

  let click_count = Arc::new(AtomicUsize::new(0));
  let mut tree = wgpu_html::parser::parse(&doc_html);
  register_system_fonts(&mut tree, "DemoSans");
  tree.register_font(FontFace::regular("lucide", Arc::from(LUCIDE_FONT)));
  install_demo_callbacks(&mut tree, &click_count);

  if doc_source.ends_with("devtools.html") {
    tree.register_linked_stylesheet("devtools.css", include_str!("../html/devtools.css"));
  }
  if profiling_enabled {
    tree.profiler = Some(wgpu_html_tree::Profiler::tagged("demo app"));
    tree.profiler.as_ref().map(|p| p.enable());
  }

  let devtools = Devtools::attach(&mut tree, false);

  let event_loop = match EventLoop::new() {
    Ok(el) => el,
    Err(e) => {
      eprintln!("demo: failed to create event loop: {e}");
      return ExitCode::FAILURE;
    }
  };

  let attrs = winit::window::Window::default_attributes()
    .with_title(format!("wgpu-html demo: {doc_source}"))
    .with_inner_size(winit::dpi::PhysicalSize::new(1920u32, 1080u32));
  let window = Arc::new(event_loop.create_window(attrs).expect("failed to create window"));

  let driver = WgpuHtml { window };
  let rt = WinitRuntime::new(driver, 1920, 1080);

  let mut app = DemoApp::new(tree, rt, profiling_enabled, devtools);
  event_loop.set_control_flow(ControlFlow::Wait);

  match event_loop.run_app(&mut app) {
    Ok(()) => ExitCode::SUCCESS,
    Err(_) => ExitCode::FAILURE,
  }
}
