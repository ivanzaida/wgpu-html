pub fn build() -> wgpu_html_tree::Tree {
  const HTML: &str = include_str!("../../html/text-wrapping.html");
  let tree = wgpu_html::parser::parse(HTML);
  tree
}
