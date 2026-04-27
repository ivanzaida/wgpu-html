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

use std::collections::HashMap;

use wgpu_html_models::Style;
use wgpu_html_parser::{
    CssWideKeyword, Selector, Stylesheet, parse_inline_style_decls, parse_stylesheet,
};
use wgpu_html_tree::{Element, Node, Tree};

mod element_attrs;
mod merge;
mod ua;

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
/// 2. for each element compute its style from the matching rules,
/// 3. layer the inline `style="…"` attribute on top, and
/// 4. inherit the standard inheriting properties from the parent's
///    resolved style (CSS-Cascade-3 §3.3 — `color`, font-related
///    properties, line-height, text-align, etc.).
pub fn cascade(tree: &Tree) -> CascadedTree {
    // UA defaults sit before author rules in the rules list, so on
    // a specificity tie the author rule wins on source order.
    let mut stylesheet = ua::ua_stylesheet().clone();
    stylesheet.append(collect_stylesheet(tree));
    CascadedTree {
        root: tree
            .root
            .as_ref()
            .map(|n| cascade_node(n, &stylesheet, None)),
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

fn cascade_node(
    node: &Node,
    sheet: &Stylesheet,
    parent_style: Option<&Style>,
) -> CascadedNode {
    let (mut style, keywords) = computed_decls(&node.element, sheet);

    // Resolve every CSS-wide keyword override against the parent's
    // already-resolved style. Each entry replaces the matching field
    // with `parent.field` (Inherit / inheritable Unset), `None`
    // (Initial / non-inherit Unset), or no-ops if the parent doesn't
    // carry a value for it.
    for (prop, kw) in &keywords {
        wgpu_html_parser::apply_keyword(&mut style, parent_style, prop, *kw);
    }

    // Implicit inheritance: for each inherited property the cascade
    // didn't already hand a value or keyword, take the parent's.
    if let Some(parent) = parent_style {
        inherit_into(&mut style, parent, &keywords);
    }

    let children = node
        .children
        .iter()
        .map(|c| cascade_node(c, sheet, Some(&style)))
        .collect();
    CascadedNode {
        element: node.element.clone(),
        style,
        children,
    }
}

/// Fill in unset inherited properties on `child` from `parent`'s
/// resolved style. Skips properties already touched by a CSS-wide
/// keyword in this layer — those have been resolved authoritatively
/// (an explicit `initial` shouldn't be implicitly re-inherited).
fn inherit_into(
    child: &mut Style,
    parent: &Style,
    keywords: &HashMap<String, CssWideKeyword>,
) {
    macro_rules! inherit {
        ($(($field:ident, $name:literal)),* $(,)?) => {
            $(
                if child.$field.is_none() && !keywords.contains_key($name) {
                    child.$field = parent.$field.clone();
                }
            )*
        };
    }
    inherit!(
        (color, "color"),
        (font_family, "font-family"),
        (font_size, "font-size"),
        (font_weight, "font-weight"),
        (font_style, "font-style"),
        (line_height, "line-height"),
        (letter_spacing, "letter-spacing"),
        (text_align, "text-align"),
        (text_transform, "text-transform"),
        (white_space, "white-space"),
        (text_decoration, "text-decoration"),
        (visibility, "visibility"),
        (cursor, "cursor"),
    );
}

/// Compute the cascaded style for one element against a stylesheet,
/// dropping any CSS-wide keyword overrides. Kept for callers that
/// don't have a parent style on hand and just want the typed values.
/// Use [`computed_decls`] in the cascade itself.
pub fn computed_style(element: &Element, sheet: &Stylesheet) -> Style {
    computed_decls(element, sheet).0
}

/// Compute the cascaded `(values, keyword overrides)` for one element
/// against a stylesheet.
///
/// CSS-Cascade-3 §6.4 ordering, restricted to author + inline (no UA
/// or user origin layers in this engine):
///
///   1. Author normal rules, ascending specificity (stable on ties)
///   2. Inline `style="…"` normal declarations
///   3. Author !important rules, ascending specificity
///   4. Inline `style="…"` !important declarations
///
/// Each layer's keyword side-car displaces matching values from
/// earlier layers, and a later layer's value displaces an earlier
/// layer's keyword for the same property. The returned keyword map
/// is what's left over for `cascade_node` to resolve against the
/// parent's already-resolved style.
pub fn computed_decls(
    element: &Element,
    sheet: &Stylesheet,
) -> (Style, HashMap<String, CssWideKeyword>) {
    let mut values = Style::default();
    let mut keywords: HashMap<String, CssWideKeyword> = HashMap::new();
    let inline = element_style_attr(element).map(parse_inline_style_decls);

    // 1. Author normal.
    let mut author_normal: Vec<(u32, &Style, &HashMap<String, CssWideKeyword>)> = sheet
        .rules
        .iter()
        .filter_map(|rule| {
            rule.selectors
                .iter()
                .filter(|s| matches_selector(s, element))
                .map(|s| s.specificity())
                .max()
                .map(|spec| (spec, &rule.declarations, &rule.keywords))
        })
        .collect();
    author_normal.sort_by_key(|(spec, _, _)| *spec);
    for (_, v, k) in author_normal {
        apply_layer(&mut values, &mut keywords, v, k);
    }

    // 2. Inline normal.
    if let Some(decls) = &inline {
        apply_layer(
            &mut values,
            &mut keywords,
            &decls.normal,
            &decls.keywords_normal,
        );
    }

    // 3. Author !important.
    let mut author_important: Vec<(u32, &Style, &HashMap<String, CssWideKeyword>)> = sheet
        .rules
        .iter()
        .filter_map(|rule| {
            rule.selectors
                .iter()
                .filter(|s| matches_selector(s, element))
                .map(|s| s.specificity())
                .max()
                .map(|spec| (spec, &rule.important, &rule.important_keywords))
        })
        .collect();
    author_important.sort_by_key(|(spec, _, _)| *spec);
    for (_, v, k) in author_important {
        apply_layer(&mut values, &mut keywords, v, k);
    }

    // 4. Inline !important.
    if let Some(decls) = &inline {
        apply_layer(
            &mut values,
            &mut keywords,
            &decls.important,
            &decls.keywords_important,
        );
    }

    (values, keywords)
}

/// Apply one cascade layer's `(values, keyword overrides)` to the
/// running `(values, keywords)` accumulator. Keyword overrides go
/// first — they clear the matching value field — then the value
/// merge runs and drops any keyword left behind for fields the layer
/// also wrote a value for.
fn apply_layer(
    values: &mut Style,
    keywords: &mut HashMap<String, CssWideKeyword>,
    layer_values: &Style,
    layer_keywords: &HashMap<String, CssWideKeyword>,
) {
    for (prop, kw) in layer_keywords {
        wgpu_html_parser::clear_value_for(prop, values);
        keywords.insert(prop.clone(), *kw);
    }
    wgpu_html_parser::merge_values_clearing_keywords(values, keywords, layer_values);
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
