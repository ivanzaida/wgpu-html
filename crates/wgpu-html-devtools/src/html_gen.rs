//! Build the devtools UI as a [`Tree`] directly from an inspected
//! tree — no HTML generation or parsing involved.

use std::sync::{Arc, Mutex};

use wgpu_html_tree::{Element, Node, Tree};

const CSS: &str = include_str!("../html/devtools.css");

/// Maximum tree depth rendered.
const MAX_DEPTH: usize = 32;

// ── Helpers ─────────────────────────────────────────────────────

fn div(class: &str) -> Node {
    Node::new(wgpu_html_models::Div {
        class: Some(class.to_owned()),
        ..Default::default()
    })
}

fn div_style(class: &str, style: &str) -> Node {
    Node::new(wgpu_html_models::Div {
        class: Some(class.to_owned()),
        style: Some(style.to_owned()),
        ..Default::default()
    })
}

fn span(class: &str, text: &str) -> Node {
    Node::new(wgpu_html_models::Span {
        class: Some(class.to_owned()),
        ..Default::default()
    })
    .with_children(vec![Node::new(text)])
}

fn text(s: &str) -> Node {
    Node::new(s)
}

// ── Public API ──────────────────────────────────────────────────

/// Build a complete devtools UI tree from the inspected tree.
pub fn build(
    inspected: &Tree,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> Tree {
    let style = Node::new(wgpu_html_models::StyleElement::default())
        .with_children(vec![text(CSS)]);

    let toolbar = build_toolbar();
    let main = build_main(inspected, selected_path, click_sink);

    let body = Node::new(wgpu_html_models::Body::default())
        .with_children(vec![style, toolbar, main]);

    Tree::new(body)
}

// ── Toolbar ─────────────────────────────────────────────────────

fn build_toolbar() -> Node {
    div("toolbar").with_children(vec![
        span("pick-btn", "\u{25B6}"),
        div("tb-divider"),
        div("filter").with_children(vec![span("filter-text", "Filter")]),
    ])
}

// ── Main area ───────────────────────────────────────────────────

fn build_main(
    inspected: &Tree,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> Node {
    let tree_panel = build_tree_panel(inspected, selected_path, click_sink);

    // Look up the selected node for the styles panel.
    let selected_node = selected_path.and_then(|path| {
        let root = inspected.root.as_ref()?;
        if path.is_empty() {
            Some(root)
        } else {
            root.at_path(path)
        }
    });
    let styles_panel = build_styles_panel(selected_node);

    div("main").with_children(vec![tree_panel, styles_panel])
}

// ── Tree panel ──────────────────────────────────────────────────

fn build_tree_panel(
    inspected: &Tree,
    selected_path: Option<&[usize]>,
    click_sink: &Arc<Mutex<Option<Vec<usize>>>>,
) -> Node {
    let mut rows = div("tree-rows");
    if let Some(root) = &inspected.root {
        let mut path = Vec::new();
        emit_node(&mut rows, root, 0, &mut path, selected_path, click_sink);
    }

    let breadcrumb = build_breadcrumb(inspected, selected_path);

    div("tree-panel").with_children(vec![rows, breadcrumb])
}

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
                .with_children(vec![
                    span("text-node", &format!("\"{display}\"")),
                ]);
            parent.push(row);
        }
        _ => {
            let tag = node.element.tag_name();
            if matches!(tag, "style" | "script" | "meta" | "link" | "title") {
                return;
            }

            let has_vis = has_visible_children(node);

            if has_vis {
                // Open tag row
                let mut row = tree_row(depth, path, selected_path, click_sink);
                row.push(span("chevron", "\u{25BC}"));
                push_open_tag(&mut row, node, tag);
                parent.push(row);

                // Children
                for (i, child) in node.children.iter().enumerate() {
                    path.push(i);
                    emit_node(parent, child, depth + 1, path, selected_path, click_sink);
                    path.pop();
                }

                // Close tag row
                let close = tree_row_plain(depth).with_children(vec![
                    span("bracket", "</"),
                    span("tag", tag),
                    span("bracket", ">"),
                ]);
                parent.push(close);
            } else {
                // Leaf / self-closing on one row
                let mut row = tree_row(depth, path, selected_path, click_sink);
                push_open_tag(&mut row, node, tag);

                if let Some(txt) = single_text_child(node) {
                    row.push(span("text-node", &truncate(txt, 40)));
                }

                row.push(span("bracket", "</"));
                row.push(span("tag", tag));
                row.push(span("bracket", ">"));
                parent.push(row);
            }
        }
    }
}

/// Build an interactive tree row with click-to-select and selection
/// highlight.
fn tree_row(
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
    // Store the inspected-tree path for debugging / future use.
    div_model
        .data_attrs
        .insert("path".to_string(), encode_path(path));
    let mut node = Node::new(div_model);

    // Click callback: write the inspected path to the shared sink.
    let sink = click_sink.clone();
    let path_owned = path.to_vec();
    node.on_click = Some(Arc::new(move |_| {
        *sink.lock().unwrap() = Some(path_owned.clone());
    }));

    node
}

/// Plain (non-interactive) tree row used for closing tags.
fn tree_row_plain(depth: usize) -> Node {
    let px = 12 + depth * 16;
    div_style("tree-row", &format!("padding-left: {px}px;"))
}

fn push_open_tag(row: &mut Node, node: &Node, tag: &str) {
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

fn build_breadcrumb(inspected: &Tree, selected_path: Option<&[usize]>) -> Node {
    let mut items: Vec<Node> = Vec::new();

    if let (Some(root), Some(path)) = (&inspected.root, selected_path) {
        let mut current = root;
        let len = path.len();

        // Root element
        if len == 0 {
            items.push(span("bc-active", &tag_label(current)));
        } else {
            items.push(span("bracket", &tag_label(current)));
        }

        for (i, &idx) in path.iter().enumerate() {
            items.push(text(" \u{203A} ")); // ›
            if let Some(child) = current.children.get(idx) {
                let label = tag_label(child);
                if i == len - 1 {
                    items.push(span("bc-active", &label));
                } else {
                    items.push(span("bracket", &label));
                }
                current = child;
            } else {
                break;
            }
        }
    } else {
        items.push(span("bc-active", "document"));
    }

    div("breadcrumb").with_children(items)
}

fn tag_label(node: &Node) -> String {
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

fn build_styles_panel(selected_node: Option<&Node>) -> Node {
    let tab_bar = div("tab-bar").with_children(vec![
        div_style("tab tab-active", "height: 100%;").with_children(vec![text("Styles")]),
        div_style("tab", "height: 100%;").with_children(vec![text("Computed")]),
        div_style("tab", "height: 100%;").with_children(vec![text("Layout")]),
        div_style("tab", "height: 100%;").with_children(vec![text("Event Listeners")]),
    ]);

    let style_search = div("style-search").with_children(vec![
        span("ss-label", "Filter"),
        div("ss-spacer"),
        span("ss-btn ss-btn-active", ":hov"),
        span("ss-btn", ".cls"),
        span("ss-btn", "+"),
    ]);

    let mut content = div("styles-content");

    if let Some(node) = selected_node {
        // ── element.style rule ──
        let mut element_style = div("rule");
        element_style.push(
            div("rule-header").with_children(vec![
                span("selector-text", "element.style"),
                span("brace", " {"),
            ]),
        );
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
        element_style.push(
            div("rule-end").with_children(vec![text("}")]),
        );
        content.push(element_style);

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
        info_rule.push(
            div("rule-header").with_children(vec![
                span("selector-text", &info_text),
            ]),
        );
        content.push(info_rule);
    } else {
        // No selection — placeholder.
        let placeholder = div_style("rule", "padding: 12px;").with_children(vec![
            span("text-node", "Select an element to inspect its styles"),
        ]);
        content.push(placeholder);
    }

    div("styles-panel").with_children(vec![tab_bar, style_search, content])
}

fn make_decl(prop: &str, value: &str) -> Node {
    div("decl").with_children(vec![
        div("cb"),
        span("prop", prop),
        span("colon", ": "),
        span("val", value),
        span("semi", ";"),
    ])
}

// ── Utilities ───────────────────────────────────────────────────

fn has_visible_children(node: &Node) -> bool {
    node.children.iter().any(|c| match &c.element {
        Element::Text(t) => !t.trim().is_empty(),
        _ => !matches!(
            c.element.tag_name(),
            "style" | "script" | "meta" | "link" | "title"
        ),
    })
}

fn single_text_child(node: &Node) -> Option<&str> {
    if node.children.len() == 1 {
        if let Element::Text(t) = &node.children[0].element {
            let trimmed = t.trim();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
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

fn encode_path(path: &[usize]) -> String {
    path.iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}
