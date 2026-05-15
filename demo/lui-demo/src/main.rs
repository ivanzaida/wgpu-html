mod examples;

use lui::{
  lui_core::{HtmlElement::Div, HtmlNode}, Lui, WgpuRenderer,
  WinitHarness,
};

use std::sync::{Arc, Mutex};

use crate::examples::ExampleRegistry;

const DEFAULT_HTML: &str = include_str!("../html/shell.html");

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

fn read_example_arg() -> Option<String> {
  let args: Vec<String> = std::env::args().collect();
  args.iter().position(|a| a == "--example")
    .and_then(|pos| args.get(pos + 1).cloned())
}

fn main() {
  let mut lui = Lui::new();
  let html_text = read_html();
  lui.set_html(&html_text);
  let harness = WinitHarness::new(800, 600, "lui v2 demo");
  let renderer = WgpuRenderer::new();

  let registry = Arc::new(Mutex::new(ExampleRegistry::new()));
  let names = registry.lock().unwrap().names();

  if let Some(nav_bar) = lui.doc.root.get_element_by_id_mut("nav-section".into()) {
    for name in &names {
      let mut node = HtmlNode::new(Div);
      node.class_list_mut().add("nav-item");
      node.set_attribute("data-example", name);
      node.set_text_content(&name.to_uppercase());
      nav_bar.append_child(node);
    }
  }

  if let Some(example_name) = read_example_arg() {
    if let Some(res) = registry.lock().unwrap().run(&example_name, &mut lui) {
      if let Some(wrapper) = lui.doc_mut().root.get_element_by_id_mut("content-wrapper".into()) {
        wrapper.set_children(res);
      }
    }
  }

  let reg = Arc::clone(&registry);
  lui.on("click", move |lui, evt| {
    let path = evt.target_path().to_vec();

    let example_name = {
      let Some(target) = lui.doc().root.at_path(&path) else {
        return;
      };
      let Some(name) = target.data_attrs().get("example") else {
        return;
      };
      name.clone()
    };

    let Some(res) = reg.lock().unwrap().run(&example_name, lui) else {
      return;
    };

    if let Some(wrapper) = lui.doc_mut().root.get_element_by_id_mut("content-wrapper".into()) {
      wrapper.set_children(res);
    }

    if let Some(node) = lui.doc_mut().root.at_path_mut(&path) {
      node.class_list_mut().add("active");
    }
  });

  harness.run(lui, renderer);
}
