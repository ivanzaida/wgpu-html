//! egui demo shell over `lui-driver-egui`.

use std::{
  process::ExitCode,
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, Instant},
};

use eframe::egui;
use lui_driver_egui::EguiRunner;
use lui_tree::Tree;

pub(crate) fn run(tree: Tree, doc_source: String, profiling_enabled: bool) -> ExitCode {
  println!("lui demo:");
  println!("  renderer  ->  egui");
  println!("  doc       ->  {doc_source}");
  if tree.fonts.is_empty() {
    eprintln!("demo: no system font found — text will render as zero-size");
  }
  if profiling_enabled {
    eprintln!("demo: profiling enabled via --profile");
  }

  let native_options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
      .with_title(format!("lui egui demo: {doc_source}"))
      .with_inner_size([1280.0, 720.0]),
    ..Default::default()
  };

  let app_doc_source = doc_source.clone();
  let result = eframe::run_native(
    "lui egui demo",
    native_options,
    Box::new(move |_cc| Ok(Box::new(EguiDemoApp::new(tree, app_doc_source, profiling_enabled)))),
  );

  match result {
    Ok(()) => ExitCode::SUCCESS,
    Err(err) => {
      eprintln!("demo: egui event loop error: {err}");
      ExitCode::FAILURE
    }
  }
}

struct EguiDemoApp {
  tree: Tree,
  html: EguiRunner<winit::window::Window>,
  doc_source: String,
  profiling_enabled: bool,
  profiler: Profiler,
  click_count: Arc<AtomicUsize>,
}

impl EguiDemoApp {
  fn new(mut tree: Tree, doc_source: String, profiling_enabled: bool) -> Self {
    let click_count = Arc::new(AtomicUsize::new(0));
    tree.register_system_fonts("DemoSans");

    // Create a hidden window for the wgpu surface.
    // The EguiRunner needs a surface for Renderer init, but GPU
    // rendering is handled externally by eframe/egui.
    #[allow(deprecated)]
    let (window, _event_loop) = {
      let el = winit::event_loop::EventLoop::new().unwrap();
      let w = Arc::new(
        el.create_window(winit::window::Window::default_attributes().with_title("_hidden"))
          .unwrap(),
      );
      w.set_visible(false);
      (w, el)
    };

    Self {
      tree,
      html: EguiRunner::new(window, 1280, 720),
      doc_source,
      profiling_enabled,
      profiler: Profiler::new(),
      click_count,
    }
  }
}

impl eframe::App for EguiDemoApp {
  fn update(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    egui::Panel::top("demo-toolbar").show_inside(ui, |ui| {
      ui.horizontal(|ui| {
        ui.label("lui egui");
        ui.separator();
        ui.label(&self.doc_source);
        ui.separator();
        ui.label(format!("clicks: {}", self.click_count.load(Ordering::Relaxed)));
        if self.profiling_enabled {
          ui.separator();
          ui.label("profile: on");
        }
      });
    });

    egui::CentralPanel::default().show_inside(ui, |ui| {
      let size = ui.available_size().max(egui::vec2(1.0, 1.0));
      let out = self.html.show(ui, &mut self.tree, size);
      if self.profiling_enabled {
        self.profiler.add_frame(out.timings);
        if let Some(line) = self.profiler.take_summary_if_due() {
          println!("{line}");
        }
      }
    });
  }
}

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
  fn avg(self, n: u64) -> f64 {
    if n == 0 { 0.0 } else { self.sum_ms / n as f64 }
  }
}

struct Profiler {
  started_at: Instant,
  frames: u64,
  cascade: Stage,
  layout: Stage,
  paint: Stage,
}

impl Profiler {
  fn new() -> Self {
    Self {
      started_at: Instant::now(),
      frames: 0,
      cascade: Stage::default(),
      layout: Stage::default(),
      paint: Stage::default(),
    }
  }

  fn add_frame(&mut self, timings: lui::PipelineTimings) {
    self.frames += 1;
    self.cascade.add(timings.cascade_ms);
    self.layout.add(timings.layout_ms);
    self.paint.add(timings.paint_ms);
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
      "egui profile: {:.2}s frames={} fps={:.1}  cascade={:.2}/{:.2}  layout={:.2}/{:.2}  paint={:.2}/{:.2}",
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
