//! wgpu-html demo launcher.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

mod egui;
mod winit;

const DEFAULT_DOC: &str = include_str!("../html/flex-browser-like.html");
const DEFAULT_DOC_PATH: &str = "crates/wgpu-html-demo/html/flex-browser-like.html";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RendererKind {
    Winit,
    Egui,
}

fn print_usage(program: &str) {
    println!("Usage: {program} [--renderer=winit|egui] [--profile] [HTML_FILE]");
    println!();
    println!("If HTML_FILE is omitted, the built-in demo document is used:");
    println!("  {DEFAULT_DOC_PATH}");
    println!();
    println!("Options:");
    println!("  --renderer=winit|egui   choose integration backend (default: winit)");
    println!("  --profile               enable per-frame profiling logs at startup");
    println!();
    println!("Examples:");
    println!("  {program}");
    println!("  {program} --renderer=egui");
    println!("  {program} --renderer=winit --profile");
    println!("  {program} --renderer=egui crates/wgpu-html-demo/html/flex-browser-like.html");
}

fn resolve_args() -> Result<(String, String, bool, RendererKind), ExitCode> {
    let mut args = env::args_os();
    let program = args
        .next()
        .map(|arg| arg.to_string_lossy().into_owned())
        .unwrap_or_else(|| "wgpu-html-demo".to_owned());

    let mut profiling_enabled = false;
    let mut renderer = RendererKind::Winit;
    let mut doc_arg: Option<std::ffi::OsString> = None;

    for arg in args {
        let text = arg.to_string_lossy();
        match text.as_ref() {
            "-h" | "--help" => {
                print_usage(&program);
                return Err(ExitCode::SUCCESS);
            }
            "--profile" => profiling_enabled = true,
            "--renderer=winit" => renderer = RendererKind::Winit,
            "--renderer=egui" => renderer = RendererKind::Egui,
            _ if text.starts_with("--renderer=") => {
                eprintln!("demo: unknown renderer: {text}\n");
                print_usage(&program);
                return Err(ExitCode::FAILURE);
            }
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
            renderer,
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

    Ok((
        html,
        path.display().to_string(),
        profiling_enabled,
        renderer,
    ))
}

fn main() -> ExitCode {
    let (doc_html, doc_source, profiling_enabled, renderer) = match resolve_args() {
        Ok(v) => v,
        Err(code) => return code,
    };

    match renderer {
        RendererKind::Winit => winit::run(doc_html, doc_source, profiling_enabled),
        RendererKind::Egui => egui::run(doc_html, doc_source, profiling_enabled),
    }
}
