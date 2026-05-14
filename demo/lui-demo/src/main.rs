use lui::{Lui, WgpuRenderer, WinitDriver};

const DEFAULT_HTML: &str = include_str!("../html/test.html");

fn read_html() -> String {
  use std::io::Read;
  let args: Vec<String> = std::env::args().collect();
  if let Some(pos) = args.iter().position(|a| a == "--html") {
    if let Some(path) = args.get(pos + 1) {
      return std::fs::read_to_string(path).expect("failed to read HTML file");
    }
  }
  if !atty::is(atty::Stream::Stdin) {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).expect("failed to read stdin");
    if !buf.trim().is_empty() {
      return buf;
    }
  }
  DEFAULT_HTML.to_string()
}

fn main() {
  let driver = WinitDriver::new(800, 600, "lui v2 demo");
  let renderer = WgpuRenderer::new();
  let mut lui = Lui::new(Box::new(driver), Box::new(renderer));
  lui.set_html(&read_html());
  lui.run();
}
