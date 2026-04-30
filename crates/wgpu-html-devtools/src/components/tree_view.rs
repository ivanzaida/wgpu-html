//! The element tree panel — renders the full inspected DOM as a
//! list of [`tree_node`](super::tree_node) rows.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use wgpu_html_tree::Node;

/// Maximum tree depth rendered.
pub const MAX_DEPTH: usize = 32;

/// Owns the tree-view state: which rows are selected, which are
/// collapsed, and the sinks that receive user interactions.
pub struct TreeView {
    /// Path of the currently selected row.
    pub selected_path: Option<Vec<usize>>,
    /// Receives the path when a row is clicked (selection).
    pub click_sink: Arc<Mutex<Option<Vec<usize>>>>,
    /// Receives the path when a chevron is clicked (toggle).
    pub toggle_sink: Arc<Mutex<Option<Vec<usize>>>>,
    /// Paths whose children are collapsed (hidden).
    pub collapsed: HashSet<Vec<usize>>,
}

impl TreeView {
    pub fn new() -> Self {
        Self {
            selected_path: None,
            click_sink: Arc::new(Mutex::new(None)),
            toggle_sink: Arc::new(Mutex::new(None)),
            collapsed: HashSet::new(),
        }
    }

    /// Rebuild the `#tree-rows` container from the inspected root.
    pub fn update(&self, tree: &mut wgpu_html_tree::Tree, inspected_root: Option<&Node>) {
        if let Some(container) = tree.get_element_by_id("tree-rows") {
            container.children.clear();
            if let Some(root) = inspected_root {
                let mut path = Vec::new();
                super::tree_node::emit(
                    container,
                    root,
                    0,
                    &mut path,
                    self.selected_path.as_deref(),
                    &self.click_sink,
                    &self.toggle_sink,
                    &self.collapsed,
                );
            }
        }
    }

    /// Drain the toggle and click sinks. Returns `(toggled, clicked)`
    /// — `true` if either sink had a pending event.
    pub fn drain(&mut self) -> (bool, bool) {
        let mut toggled = false;
        if let Some(path) = self.toggle_sink.lock().unwrap().take() {
            if !self.collapsed.remove(&path) {
                self.collapsed.insert(path);
            }
            toggled = true;
        }

        let mut clicked = false;
        if let Some(path) = self.click_sink.lock().unwrap().take() {
            self.selected_path = Some(path);
            clicked = true;
        }

        (toggled, clicked)
    }
}
