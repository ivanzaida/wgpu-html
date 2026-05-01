//! Shared utility functions for the devtools UI.

use wgpu_html_tree::Node;

// ── Lucide icon codepoints (PUA) ────────────────────────────────

pub const ICON_CHEVRON_DOWN: &str = "\u{e06d}";
pub const ICON_CHEVRON_RIGHT: &str = "\u{e06f}";

// ── Utilities ───────────────────────────────────────────────────

/// Format a node as a breadcrumb label: `tag#id.class1.class2`.
pub fn tag_label(node: &Node) -> String {
    let tag = node.element.tag_name();
    let mut label = tag.to_string();
    if let Some(id) = node.element.id() {
        label.push('#');
        label.push_str(id);
    }
    if let Some(cls) = node.element.class() {
        for c in cls.split_whitespace().take(2) {
            label.push('.');
            label.push_str(c);
        }
    }
    label
}

/// Whether the node has visible *element* children that warrant an
/// expanded (multi-row) tree display.
pub fn has_visible_children(node: &Node) -> bool {
    node.children.iter().any(|c| match &c.element {
        wgpu_html_tree::Element::Text(_) => false,
        _ => !matches!(
            c.element.tag_name(),
            "style" | "script" | "meta" | "link" | "title"
        ),
    })
}

/// Return the combined text content when a node's children are all
/// text nodes.
pub fn text_only_content(node: &Node) -> Option<String> {
    if node.children.is_empty() {
        return None;
    }
    let mut buf = String::new();
    for c in &node.children {
        if let wgpu_html_tree::Element::Text(t) = &c.element {
            buf.push_str(t);
        } else {
            return None;
        }
    }
    let trimmed = buf.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

pub fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_owned()
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

pub fn encode_path(path: &[usize]) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}
