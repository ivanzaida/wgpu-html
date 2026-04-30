//! The styles panel showing computed styles of the selected element.

use wgpu_html_tree::Node;

use crate::tags::{div, div_style, make_decl, span, text};

/// Styles panel component. Stateless — renders from the selected
/// node each time.
pub struct StylesPanel;

impl StylesPanel {
    pub fn new() -> Self {
        Self
    }

    /// Rebuild the `#styles-content` container.
    pub fn update(
        &self,
        tree: &mut wgpu_html_tree::Tree,
        inspected_root: Option<&Node>,
        selected_path: Option<&[usize]>,
    ) {
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
            Self::populate(container, selected_node);
        }
    }

    fn populate(container: &mut Node, selected_node: Option<&Node>) {
        if let Some(node) = selected_node {
            // element.style rule
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

            // Element info
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
            info_rule
                .push(div("rule-header").with_children(vec![span("selector-text", &info_text)]));
            container.push(info_rule);
        } else {
            let placeholder = div_style("rule", "padding: 12px;").with_children(vec![span(
                "text-node",
                "Select an element to inspect its styles",
            )]);
            container.push(placeholder);
        }
    }
}
