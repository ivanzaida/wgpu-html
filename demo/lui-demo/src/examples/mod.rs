use std::collections::HashMap;

use lui_core::HtmlNode;

pub mod flex;
pub mod grid;

pub enum ExampleOutput {
  Node(HtmlNode),
  Nodes(Vec<HtmlNode>),
}

pub trait Example: Send {
  fn get_name(&self) -> &'static str;
  fn render(&mut self, lui: &mut lui::Lui) -> ExampleOutput;
  fn cleanup(&mut self, lui: &mut lui::Lui);
}

pub struct ExampleRegistry {
  factories: Vec<(&'static str, fn() -> Box<dyn Example>)>,
  active: HashMap<String, Box<dyn Example>>,
}

impl ExampleRegistry {
  pub fn new() -> Self {
    Self {
      factories: vec![
        ("flex", || Box::new(flex::FlexExample::default())),
        ("grid", || Box::new(grid::GridExample::default())),
      ],
      active: HashMap::new(),
    }
  }

  pub fn names(&self) -> Vec<&'static str> {
    self.factories.iter().map(|(n, _)| *n).collect()
  }

  pub fn run(&mut self, name: &str, lui: &mut lui::Lui) -> Option<Vec<HtmlNode>> {
    if let Some(ex) = self.active.get_mut(name) {
      let out = ex.render(lui);
      return Some(into_nodes(out));
    }

    let factory = self.factories.iter().find(|(n, _)| *n == name)?.1;
    let mut ex = factory();
    let out = ex.render(lui);
    self.active.insert(name.to_string(), ex);
    Some(into_nodes(out))
  }

  pub fn cleanup(&mut self, name: &str, lui: &mut lui::Lui) {
    if let Some(mut ex) = self.active.remove(name) {
      ex.cleanup(lui);
    }
  }

  pub fn cleanup_all(&mut self, lui: &mut lui::Lui) {
    for (_, mut ex) in self.active.drain() {
      ex.cleanup(lui);
    }
  }
}

fn into_nodes(out: ExampleOutput) -> Vec<HtmlNode> {
  match out {
    ExampleOutput::Node(n) => vec![n],
    ExampleOutput::Nodes(ns) => ns,
  }
}
