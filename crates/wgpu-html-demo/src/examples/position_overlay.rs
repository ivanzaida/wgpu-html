pub fn build() -> wgpu_html_tree::Tree {
  const HTML: &str = include_str!("../../html/position-overlay.html");
  let tree = wgpu_html::parser::parse(HTML);
  tree
}
