pub fn build() -> wgpu_html::tree::Tree {
  const HTML: &str = include_str!("../../html/resize.html");
  let tree = wgpu_html::parser::parse(HTML);


  tree
}
