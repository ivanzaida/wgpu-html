use std::sync::{
  atomic::{AtomicU32, Ordering},
  Arc,
};

use lui::{Lui, WgpuRenderer, WinitHarness};

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
  let mut lui = Lui::new();
  lui.set_html(&read_html());
  let harness = WinitHarness::new(800, 600, "lui v2 demo");
  let renderer = WgpuRenderer::new();

  lui.doc.add_event_listener(
    "click",
    Arc::new(|node, event| {
      println!("click event");
    }),
  );
  let mut counter = AtomicU32::new(0);
  harness.run(lui, renderer, move |ctx| {
    let elem = ctx.lui.doc.root.get_element_by_id_mut("test".into()).unwrap();
    let c = counter.fetch_add(1, Ordering::Relaxed);
    elem.set_text_content(&format!("test {}", c));
  });
}
