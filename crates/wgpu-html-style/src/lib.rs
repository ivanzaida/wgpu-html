//! Selector matching and CSS cascade.
//!
//! Inputs:
//! - a parsed `Tree` of typed elements (with their inline `style` attributes
//!   and `id` / `class` attributes already populated by the HTML parser)
//! - the contents of any `<style>` blocks found in the tree
//!
//! Output: a parallel `CascadedTree` where every element has a fully
//! computed `Style`. Layout consumes this and never re-parses CSS.
//!
//! Cascade order, lowest specificity first, last writer wins on ties:
//! 1. Stylesheet rules (sorted by specificity ascending, then source order)
//! 2. The element's inline `style="…"` attribute (treated as specificity
//!    higher than any selector, per CSS)

use wgpu_html_models::Style;
use wgpu_html_parser::{Selector, Stylesheet, parse_inline_style, parse_stylesheet};
use wgpu_html_tree::{Element, Node, Tree};

mod element_attrs;
mod merge;

pub use element_attrs::{element_class, element_id, element_style_attr, element_tag};
pub use merge::merge;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CascadedTree {
    pub root: Option<CascadedNode>,
}

#[derive(Debug, Clone)]
pub struct CascadedNode {
    pub element: Element,
    pub style: Style,
    pub children: Vec<CascadedNode>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Cascade a tree end-to-end:
///
/// 1. collect every `<style>` block's text content into one stylesheet,
/// 2. for each element compute its style from the matching rules, then
/// 3. layer the inline `style="…"` attribute on top.
pub fn cascade(tree: &Tree) -> CascadedTree {
    let stylesheet = collect_stylesheet(tree);
    CascadedTree {
        root: tree.root.as_ref().map(|n| cascade_node(n, &stylesheet)),
    }
}

/// Walk the tree, gather text content of all `<style>` blocks, and parse it.
pub fn collect_stylesheet(tree: &Tree) -> Stylesheet {
    let mut sheet = Stylesheet::default();
    if let Some(root) = &tree.root {
        gather(root, &mut sheet);
    }
    sheet
}

fn gather(node: &Node, out: &mut Stylesheet) {
    if matches!(&node.element, Element::StyleElement(_)) {
        let mut css = String::new();
        for child in &node.children {
            if let Element::Text(t) = &child.element {
                css.push_str(t);
            }
        }
        out.append(parse_stylesheet(&css));
    }
    for child in &node.children {
        gather(child, out);
    }
}

fn cascade_node(node: &Node, sheet: &Stylesheet) -> CascadedNode {
    let style = computed_style(&node.element, sheet);
    let children = node
        .children
        .iter()
        .map(|c| cascade_node(c, sheet))
        .collect();
    CascadedNode {
        element: node.element.clone(),
        style,
        children,
    }
}

/// Compute the cascaded style for one element against a stylesheet.
pub fn computed_style(element: &Element, sheet: &Stylesheet) -> Style {
    let mut out = Style::default();

    // 1. matching rules in order of ascending specificity (stable sort keeps source order on ties)
    let mut matched: Vec<(u32, &Style)> = sheet
        .rules
        .iter()
        .filter_map(|rule| {
            rule.selectors
                .iter()
                .filter(|s| matches_selector(s, element))
                .map(|s| s.specificity())
                .max()
                .map(|spec| (spec, &rule.declarations))
        })
        .collect();
    matched.sort_by_key(|(spec, _)| *spec);
    for (_, decls) in matched {
        merge(&mut out, decls);
    }

    // 2. inline style overrides everything
    if let Some(s) = element_style_attr(element) {
        let inline = parse_inline_style(s);
        merge(&mut out, &inline);
    }

    out
}

// ---------------------------------------------------------------------------
// Selector matching
// ---------------------------------------------------------------------------

pub fn matches_selector(sel: &Selector, element: &Element) -> bool {
    if let Some(tag) = &sel.tag {
        match element_tag(element) {
            Some(t) if t == tag => {}
            _ => return false,
        }
    }
    if let Some(id) = &sel.id {
        match element_id(element) {
            Some(eid) if eid == id => {}
            _ => return false,
        }
    }
    if !sel.classes.is_empty() {
        let class_attr = element_class(element).unwrap_or("");
        for needed in &sel.classes {
            if !class_attr.split_ascii_whitespace().any(|c| c == needed) {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests;
