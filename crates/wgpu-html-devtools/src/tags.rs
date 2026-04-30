//! Reusable node-building helpers for the devtools UI.
//!
//! These produce [`Node`] subtrees that match the CSS classes in
//! `devtools.css`. Used by [`crate::html_gen`] to populate the
//! dynamic containers (tree rows, breadcrumb, styles).

use std::sync::{Arc, Mutex};

use wgpu_html_tree::Node;

// ── Lucide icon codepoints (PUA) ────────────────────────────────

pub const ICON_CHEVRON_DOWN: &str = "\u{e06d}";
#[allow(dead_code)]
pub const ICON_CHEVRON_RIGHT: &str = "\u{e06f}";

// ── Primitive helpers ───────────────────────────────────────────

pub fn div(class: &str) -> Node {
    Node::new(wgpu_html_models::Div {
        class: Some(class.to_owned()),
        ..Default::default()
    })
}

pub fn div_style(class: &str, style: &str) -> Node {
    Node::new(wgpu_html_models::Div {
        class: Some(class.to_owned()),
        style: Some(style.to_owned()),
        ..Default::default()
    })
}

pub fn span(class: &str, text: &str) -> Node {
    Node::new(wgpu_html_models::Span {
        class: Some(class.to_owned()),
        ..Default::default()
    })
    .with_children(vec![Node::new(text)])
}

pub fn text(s: &str) -> Node {
    Node::new(s)
}

// ── Tree rows ───────────────────────────────────────────────────

/// Build an interactive tree row with click-to-select and selection
/// highlight. `depth` controls indentation via `padding-left`.
pub fn tree_row(
    depth: usize,
    path: &[usize],
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> Node {
    let px = 12 + depth * 16;
    let is_selected = selected_path.is_some_and(|sp| sp == path);
    let class = if is_selected {
        "tree-row tree-row-selected"
    } else {
        "tree-row"
    };
    let mut div_model = wgpu_html_models::Div {
        class: Some(class.to_owned()),
        style: Some(format!("padding-left: {px}px;")),
        ..Default::default()
    };
    div_model
        .data_attrs
        .insert("path".to_string(), encode_path(path));
    let mut node = Node::new(div_model);

    let sink = click_sink.clone();
    let path_owned = path.to_vec();
    node.on_click = Some(Arc::new(move |_| {
        *sink.lock().unwrap() = Some(path_owned.clone());
    }));

    node
}

/// Plain (non-interactive) tree row used for closing tags.
pub fn tree_row_plain(depth: usize) -> Node {
    let px = 12 + depth * 16;
    div_style("tree-row", &format!("padding-left: {px}px;"))
}

/// Append `<tag id="…" class="…">` spans to `row`.
pub fn push_open_tag(row: &mut Node, node: &Node, tag: &str) {
    row.push(span("bracket", "<"));
    row.push(span("tag", tag));

    if let Some(id) = node.element.id() {
        row.push(span("attr-n", " id"));
        row.push(span("bracket", "="));
        row.push(span("attr-v", &format!("\"{id}\"")));
    }
    if let Some(cls) = node.element.class() {
        row.push(span("attr-n", " class"));
        row.push(span("bracket", "="));
        row.push(span("attr-v", &format!("\"{cls}\"")));
    }

    row.push(span("bracket", ">"));
}

// ── Breadcrumb ──────────────────────────────────────────────────

/// Format a node as a breadcrumb label: `tag#id.class1.class2`.
pub fn tag_label(node: &Node) -> String {
    let tag = node.element.tag_name();
    let mut label = tag.to_string();
    if let Some(id) = node.element.id() {
        label.push('#');
        label.push_str(&id);
    }
    if let Some(cls) = node.element.class() {
        for c in cls.split_whitespace().take(2) {
            label.push('.');
            label.push_str(c);
        }
    }
    label
}

// ── Styles panel ────────────────────────────────────────────────

/// Build a single CSS declaration row: `prop: value;`
pub fn make_decl(prop: &str, value: &str) -> Node {
    div("decl").with_children(vec![
        div("cb"),
        span("prop", prop),
        span("colon", ": "),
        span("val", value),
        span("semi", ";"),
    ])
}

// ── Utilities ───────────────────────────────────────────────────

/// Whether the node has visible *element* children that warrant an
/// expanded (multi-row) tree display. Nodes whose only children
/// are text nodes are rendered inline on one row, matching browser
/// devtools behaviour (`<h1>My App</h1>` on a single line).
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
/// text nodes. Used to display the content inline on the same row
/// as the tag (`<h1>My App</h1>`).
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
    if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) }
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
