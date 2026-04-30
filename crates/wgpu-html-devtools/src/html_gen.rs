//! Build the devtools UI shell.

const SHELL_HTML: &str = include_str!("../html/devtools.html");
const CSS: &str = include_str!("../html/devtools.css");

/// Parse the static devtools shell with empty containers.
pub fn build_shell() -> wgpu_html_tree::Tree {
    let mut tree = wgpu_html_parser::parse(SHELL_HTML);
    tree.register_linked_stylesheet("devtools.css", CSS);
    tree
}
