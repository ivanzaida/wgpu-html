//! A single node in the devtools element tree.
//!
//! Renders one inspected DOM node as either:
//! - **Expandable**: chevron + open tag, children (if not collapsed), close tag
//! - **Inline**: open tag + text content + close tag on one row
//! - **Text leaf**: quoted text content

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use wgpu_html_tree::{Element, Node};

use crate::tags::*;

/// Emit the rows for one inspected node into `parent`.
///
/// - `path`: the node's index path in the inspected tree (used for
///   selection highlight and collapse tracking).
/// - `collapsed`: set of paths whose children should be hidden.
/// - `toggle_sink`: receives the path when a chevron is clicked.
/// - `click_sink`: receives the path when a row is clicked (selection).
pub fn emit(
    parent: &mut Node,
    node: &Node,
    depth: usize,
    path: &mut Vec<usize>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    toggle_sink: &Arc<Mutex<Option<Vec<usize>>>>,
    collapsed: &HashSet<Vec<usize>>,
) {
    if depth > super::tree_view::MAX_DEPTH {
        return;
    }

    match &node.element {
        Element::Text(t) => {
            let trimmed = t.trim();
            if trimmed.is_empty() {
                return;
            }
            let display = truncate(trimmed, 60);
            let row = tree_row(depth, path, selected_path, click_sink)
                .with_children(vec![span("text-node", &format!("\"{display}\""))]);
            parent.push(row);
        }
        _ => {
            let tag = node.element.tag_name();
            if matches!(tag, "style" | "script" | "meta" | "link" | "title") {
                return;
            }

            let has_vis = has_visible_children(node);

            if has_vis {
                let is_collapsed = collapsed.contains(path.as_slice());
                let icon = if is_collapsed {
                    ICON_CHEVRON_RIGHT
                } else {
                    ICON_CHEVRON_DOWN
                };

                let mut row = tree_row(depth, path, selected_path, click_sink);
                row.push(chevron_button(icon, path, toggle_sink));
                push_open_tag(&mut row, node, tag);

                if is_collapsed {
                    // Show "…" indicator and inline close tag.
                    row.push(span("text-node", "\u{2026}"));
                    row.push(span("bracket", "</"));
                    row.push(span("tag", tag));
                    row.push(span("bracket", ">"));
                    parent.push(row);
                } else {
                    parent.push(row);

                    for (i, child) in node.children.iter().enumerate() {
                        path.push(i);
                        emit(
                            parent,
                            child,
                            depth + 1,
                            path,
                            selected_path,
                            click_sink,
                            toggle_sink,
                            collapsed,
                        );
                        path.pop();
                    }

                    let close = tree_row_plain(depth).with_children(vec![
                        span("bracket", "</"),
                        span("tag", tag),
                        span("bracket", ">"),
                    ]);
                    parent.push(close);
                }
            } else {
                let mut row = tree_row(depth, path, selected_path, click_sink);
                push_open_tag(&mut row, node, tag);

                if let Some(txt) = text_only_content(node) {
                    row.push(span("text-node", &truncate(&txt, 40)));
                }

                row.push(span("bracket", "</"));
                row.push(span("tag", tag));
                row.push(span("bracket", ">"));
                parent.push(row);
            }
        }
    }
}

/// Build a clickable chevron span that toggles collapse on click.
fn chevron_button(
    icon: &str,
    path: &[usize],
    toggle_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> Node {
    let mut node = span("chevron", icon);
    let sink = toggle_sink.clone();
    let path_owned = path.to_vec();
    node.on_click = Some(Arc::new(move |_| {
        *sink.lock().unwrap() = Some(path_owned.clone());
    }));
    node
}
