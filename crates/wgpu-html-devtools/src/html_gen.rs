//! Build the devtools UI by parsing the static HTML shell and
//! injecting dynamic content (tree rows, breadcrumb, styles)
//! into placeholder containers identified by `id`.

use std::sync::{Arc, Mutex};

use wgpu_html_tree::{Element, Node};

use crate::tags::*;

const SHELL_HTML: &str = include_str!("../html/devtools.html");
const CSS: &str = include_str!("../html/devtools.css");

/// Maximum tree depth rendered.
const MAX_DEPTH: usize = 32;

// ── Public API ──────────────────────────────────────────────────

/// Parse the static devtools shell and populate the dynamic
/// containers with content derived from the inspected tree.
pub fn build(
    inspected_root: Option<&Node>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> wgpu_html_tree::Tree {
    let mut tree = wgpu_html_parser::parse(SHELL_HTML);
    tree.register_linked_stylesheet("devtools.css", CSS);

    // ── Tree rows ───────────────────────────────────────────
    if let Some(container) = tree.get_element_by_id("tree-rows") {
        container.children.clear();
        if let Some(root) = inspected_root {
            let mut path = Vec::new();
            emit_node(container, root, 0, &mut path, selected_path, click_sink);
        }
    }

    // ── Breadcrumb ──────────────────────────────────────────
    if let Some(container) = tree.get_element_by_id("breadcrumb") {
        container.children.clear();
        populate_breadcrumb(container, inspected_root, selected_path);
    }

    // ── Styles content ──────────────────────────────────────
    if let Some(container) = tree.get_element_by_id("styles-content") {
        container.children.clear();
        let selected_node = selected_path.and_then(|path| {
            let root = inspected_root?;
            if path.is_empty() {
                Some(root)
            } else {
                root.at_path(path)
            }
        });
        populate_styles(container, selected_node);
    }

    tree
}

// ── Tree rows ───────────────────────────────────────────────────

fn emit_node(
    parent: &mut Node,
    node: &Node,
    depth: usize,
    path: &mut Vec<usize>,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) {
    if depth > MAX_DEPTH {
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
                let mut row = tree_row(depth, path, selected_path, click_sink);
                row.push(span("chevron", ICON_CHEVRON_DOWN));
                push_open_tag(&mut row, node, tag);
                parent.push(row);

                for (i, child) in node.children.iter().enumerate() {
                    path.push(i);
                    emit_node(parent, child, depth + 1, path, selected_path, click_sink);
                    path.pop();
                }

                let close = tree_row_plain(depth).with_children(vec![
                    span("bracket", "</"),
                    span("tag", tag),
                    span("bracket", ">"),
                ]);
                parent.push(close);
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

// ── Breadcrumb ──────────────────────────────────────────────────

fn populate_breadcrumb(
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

// ── Styles content ──────────────────────────────────────────────

fn populate_styles(container: &mut Node, selected_node: Option<&Node>) {
    if let Some(node) = selected_node {
        // ── element.style rule ──
        let mut element_style = div("rule");
        element_style.push(div("rule-header").with_children(vec![
            span("selector-text", "element.style"),
            span("brace", " {"),
        ]));
        if let Some(style_str) = node.element.attr("style") {
            for decl in style_str.split(';') {
                let decl = decl.trim();
                if decl.is_empty() {
                    continue;
                }
                if let Some((prop, value)) = decl.split_once(':') {
                    element_style.push(make_decl(prop.trim(), value.trim()));
                }
            }
        }
        element_style.push(div("rule-end").with_children(vec![text("}")]));
        container.push(element_style);

        // ── Element info ──
        let tag = node.element.tag_name();
        let mut info_parts: Vec<String> = Vec::new();
        info_parts.push(format!("<{tag}>"));
        if let Some(id) = node.element.id() {
            info_parts.push(format!("id=\"{id}\""));
        }
        if let Some(cls) = node.element.class() {
            info_parts.push(format!("class=\"{cls}\""));
        }
        let info_text = info_parts.join("  ");

        let mut info_rule = div("rule");
        info_rule.push(div("rule-header").with_children(vec![span("selector-text", &info_text)]));
        container.push(info_rule);
    } else {
        let placeholder = div_style("rule", "padding: 12px;").with_children(vec![span(
            "text-node",
            "Select an element to inspect its styles",
        )]);
        container.push(placeholder);
    }
}
