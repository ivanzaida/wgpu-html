pub fn build() -> lui_tree::Tree {
  const HTML: &str = include_str!("../../html/map-editor-new-project-modal.html");
  let tree = lui::parser::parse(HTML);
  tree
}
