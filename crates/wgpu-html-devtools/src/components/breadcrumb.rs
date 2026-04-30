//! The breadcrumb bar showing the path to the selected element.

use wgpu_html_tree::Node;

use crate::tags::{span, tag_label, text};

/// Rebuild the `#breadcrumb` container.
pub fn update(
    tree: &mut wgpu_html_tree::Tree,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
) {
    if let Some(container) = tree.get_element_by_id("breadcrumb") {
        container.children.clear();
        populate(container, inspected_root, selected_path);
    }
}

fn populate(
    container: &mut Node,
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
) {
    if let (Some(root), Some(path)) = (inspected_root, selected_path) {
        let mut current = root;
        let len = path.len();

        if len == 0 {
            container.push(span("bc-active", &tag_label(current)));
        } else {
            container.push(span("bracket", &tag_label(current)));
        }

        for (i, &idx) in path.iter().enumerate() {
            container.push(text(" \u{203A} "));
            if let Some(child) = current.children.get(idx) {
                let label = tag_label(child);
                if i == len - 1 {
                    container.push(span("bc-active", &label));
                } else {
                    container.push(span("bracket", &label));
                }
                current = child;
            } else {
                break;
            }
        }
    } else {
        container.push(span("bc-active", "document"));
    }
}
