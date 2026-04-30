//! The element tree panel — renders the full inspected DOM as a
//! list of [`tree_node`](super::tree_node) rows.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use wgpu_html_tree::Node;

/// Maximum tree depth rendered.
pub const MAX_DEPTH: usize = 32;

/// Rebuild the `#tree-rows` container from the inspected root.
pub fn update(
    tree: &mut wgpu_html_tree::Tree,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    toggle_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    collapsed: &HashSet<Vec<usize>>,
) {
    if let Some(container) = tree.get_element_by_id("tree-rows") {
        container.children.clear();
        if let Some(root) = inspected_root {
            let mut path = Vec::new();
            super::tree_node::emit(
                container,
                root,
                0,
                &mut path,
                selected_path,
                click_sink,
                toggle_sink,
                collapsed,
            );
        }
    }
}
