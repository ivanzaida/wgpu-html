//! lui demo launcher.

mod egui;
mod examples;
mod parse_cmd;
mod winit;

use std::{path::PathBuf, process::ExitCode, sync::Arc};

use lui_tree::FontFace;

const DEFAULT_EXAMPLE: &str = "flex_browser_like";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RendererKind {
  Winit,
  Egui,
}

static LUCIDE_FONT: &[u8] = include_bytes!("../fonts/lucide.ttf");

// ── CLI dispatch ──────────────────────────────────────────────────────────

fn print_usage(program: &str) {
  println!("Usage: {program} [--renderer=winit|egui] [--profile] [--example=NAME | HTML_FILE]");
  println!();
  println!("If no argument is given, the built-in demo runs ({DEFAULT_EXAMPLE}).");
  println!();
  println!("Options:");
  println!("  --renderer=winit|egui   choose integration backend (default: winit)");
  println!("  --profile               enable per-frame profiling logs at startup");
  println!("  --example=NAME          run a named example (see --examples)");
  println!("  --examples              list available examples");
  println!();
  println!("You can also pass an HTML file path directly:");
  println!("  {program} crates/lui-demo/html/flex-browser-like.html");
}

fn print_examples() {
  println!("Available examples:");
  for name in examples::list_examples() {
    println!("  {name}");
  }
}

fn run() -> ExitCode {
  let program = std::env::args().next().unwrap_or_else(|| "lui-demo".into());
  let cmd = parse_cmd::parse_command_line();

  if cmd.flags.contains_key("-h") || cmd.flags.contains_key("--help") {
    print_usage(&program);
    return ExitCode::SUCCESS;
  }
  if cmd.flags.contains_key("--examples") {
    print_examples();
    return ExitCode::SUCCESS;
  }

  let profiling = cmd.flags.contains_key("--profile");

  let renderer = match cmd.flag_str("--renderer").as_deref() {
    Some("winit") | None => RendererKind::Winit,
    Some("egui") => RendererKind::Egui,
    Some(other) => {
      eprintln!("demo: unknown renderer: {other}\n");
      print_usage(&program);
      return ExitCode::FAILURE;
    }
  };

  if let Some(name) = cmd.flag_str("--example") {
    return run_example(&name, profiling, renderer);
  }

  if let Some(pos) = cmd.positional.first() {
    if examples::get_example_tree(pos).is_some() {
      return run_example(pos, profiling, renderer);
    }
    return run_file(PathBuf::from(pos), profiling, renderer);
  }

  run_example(DEFAULT_EXAMPLE, profiling, renderer)
}

fn run_example(name: &str, profiling: bool, renderer: RendererKind) -> ExitCode {
  match examples::get_example_tree(name) {
    Some(tree) => launch(tree, format!("example:{name}"), profiling, renderer),
    None => {
      eprintln!("demo: unknown example: {name}");
      println!();
      print_examples();
      ExitCode::FAILURE
    }
  }
}

fn run_file(path: PathBuf, profiling: bool, renderer: RendererKind) -> ExitCode {
  match std::fs::read_to_string(&path) {
    Ok(html) => launch(
      lui::parser::parse(&html),
      path.display().to_string(),
      profiling,
      renderer,
    ),
    Err(err) => {
      eprintln!("demo: failed to read HTML document '{}': {err}", path.display());
      ExitCode::FAILURE
    }
  }
}

fn launch(mut tree: lui_tree::Tree, source: String, profiling: bool, renderer: RendererKind) -> ExitCode {
  tree.register_system_fonts("DemoSans");
  tree.register_font(FontFace::regular("lucide", Arc::from(LUCIDE_FONT)));

  if source.ends_with("devtools.html") {
    tree.register_linked_stylesheet("devtools.css", include_str!("../html/devtools.css"));
  }

  match renderer {
    RendererKind::Winit => winit::run(tree, source, profiling),
    RendererKind::Egui => egui::run(tree, source, profiling),
  }
}

fn main() -> ExitCode {
  run()
}
