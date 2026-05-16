use lui::{Lui, StylesheetHandle};
use lui_parse::html::parser::parse_nodes;

use crate::examples::Example;

const GRID_HTML: &'static str = include_str!("../../html/grid.html");

#[derive(Default)]
pub struct GridExample {
  handles: Vec<StylesheetHandle>,
}

impl Example for GridExample {
  fn get_name(&self) -> &'static str {
    "Grid"
  }

  fn render(&mut self, lui: &mut Lui) -> super::ExampleOutput {
    let (nodes, sheets) = parse_nodes(GRID_HTML);
    self.handles = lui.add_stylesheets(sheets);
    super::ExampleOutput::Nodes(nodes)
  }

  fn cleanup(&mut self, lui: &mut Lui) {
    let handles = std::mem::take(&mut self.handles);
    lui.remove_stylesheets(handles);
  }
}
