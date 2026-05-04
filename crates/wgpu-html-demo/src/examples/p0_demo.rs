pub fn build() -> wgpu_html_tree::Tree {
  const HTML: &str = include_str!("../../html/p0-demo.html");
  let tree = wgpu_html::parser::parse(HTML);
  tree
}
