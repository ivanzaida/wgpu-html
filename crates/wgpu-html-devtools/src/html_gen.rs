//! Build the devtools UI shell and delegate dynamic content to
//! [`crate::components`].

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use wgpu_html_tree::Node;

use crate::components;

const SHELL_HTML: &str = include_str!("../html/devtools.html");
const CSS: &str = include_str!("../html/devtools.css");

/// Parse the static devtools shell with empty containers.
pub fn build_shell() -> wgpu_html_tree::Tree {
    let mut tree = wgpu_html_parser::parse(SHELL_HTML);
    tree.register_linked_stylesheet("devtools.css", CSS);
    tree
}

/// Full build (shell + all containers).
pub fn build(
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    toggle_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    collapsed: &HashSet<Vec<usize>>,
) -> wgpu_html_tree::Tree {
    let mut tree = build_shell();
    update_tree_rows(&mut tree, inspected_root, selected_path, click_sink, toggle_sink, collapsed);
    update_breadcrumb(&mut tree, inspected_root, selected_path);
    update_styles(&mut tree, inspected_root, selected_path);
    tree
}

pub fn update_tree_rows(
    tree: &mut wgpu_html_tree::Tree,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    toggle_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    collapsed: &HashSet<Vec<usize>>,
) {
    components::tree_view::update(tree, inspected_root, selected_path, click_sink, toggle_sink, collapsed);
}

pub fn update_breadcrumb(
    tree: &mut wgpu_html_tree::Tree,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
) {
    components::breadcrumb::update(tree, inspected_root, selected_path);
}

pub fn update_styles(
    tree: &mut wgpu_html_tree::Tree,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
) {
    components::styles_panel::update(tree, inspected_root, selected_path);
}
