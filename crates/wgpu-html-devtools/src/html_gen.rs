//! Build the devtools UI as a [`Tree`] directly from an inspected
//! tree — no HTML generation or parsing involved.

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
pub fn build(inspected: &Tree) -> Tree {
    let style = Node::new(wgpu_html_models::StyleElement::default())
        .with_children(vec![text(CSS)]);

    let toolbar = build_toolbar();
    let main = build_main(inspected);

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

fn build_main(inspected: &Tree) -> Node {
    let tree_panel = build_tree_panel(inspected);
    let styles_panel = build_styles_panel(inspected);
    div("main").with_children(vec![tree_panel, styles_panel])
}

// ── Tree panel ──────────────────────────────────────────────────

fn build_tree_panel(inspected: &Tree) -> Node {
    let mut rows = div("tree-rows");
    if let Some(root) = &inspected.root {
        emit_node(&mut rows, root, 0);
    }

    let breadcrumb = div("breadcrumb").with_children(vec![
        span("bc-active", "document"),
    ]);

    div("tree-panel").with_children(vec![rows, breadcrumb])
}

fn emit_node(parent: &mut Node, node: &Node, depth: usize) {
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
            let row = tree_row(depth).with_children(vec![
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
                let mut row = tree_row(depth);
                row.push(span("chevron", "\u{25BC}"));
                push_open_tag(&mut row, node, tag);
                parent.push(row);

                // Children
                for child in &node.children {
                    emit_node(parent, child, depth + 1);
                }

                // Close tag row
                let close = tree_row(depth).with_children(vec![
                    span("bracket", "</"),
                    span("tag", tag),
                    span("bracket", ">"),
                ]);
                parent.push(close);
            } else {
                // Leaf / self-closing on one row
                let mut row = tree_row(depth);
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

fn tree_row(depth: usize) -> Node {
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

// ── Styles panel ────────────────────────────────────────────────

fn build_styles_panel(inspected: &Tree) -> Node {
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

    // element.style rule — show inline style of the root element
    let mut element_style = div("rule");
    element_style.push(
        div("rule-header").with_children(vec![
            span("selector-text", "element.style"),
            span("brace", "{"),
        ]),
    );
    if let Some(root) = &inspected.root {
        if let Some(style_str) = root.element.attr("style") {
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
    }
    element_style.push(
        div("rule-end").with_children(vec![text("}")]),
    );
    content.push(element_style);

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
