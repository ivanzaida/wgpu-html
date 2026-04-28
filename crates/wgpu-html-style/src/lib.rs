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
    CssWideKeyword, PseudoClass, Selector, Stylesheet, parse_inline_style_decls, parse_stylesheet,
};
use wgpu_html_tree::{Element, InteractionState, Node, Tree};

/// Per-element state consulted by the matcher when resolving dynamic
/// pseudo-classes (`:hover`, `:active`, …). Default is "nothing on";
/// rules with pseudo-classes never match a default context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MatchContext {
    pub is_hover: bool,
    pub is_active: bool,
    pub is_focus: bool,
}

impl MatchContext {
    /// Compute the context for the element at `path` given the
    /// document's `InteractionState`. An element is "in the hover
    /// chain" iff its path is a prefix of `state.hover_path` (i.e. it
    /// is, or is an ancestor of, the deepest hovered element).
    pub fn for_path(path: &[usize], state: &InteractionState) -> Self {
        Self {
            is_hover: path_is_prefix(path, state.hover_path.as_deref()),
            is_active: path_is_prefix(path, state.active_path.as_deref()),
            is_focus: false,
        }
    }
}

fn path_is_prefix(path: &[usize], target: Option<&[usize]>) -> bool {
    match target {
        Some(t) => t.len() >= path.len() && t[..path.len()] == *path,
        None => false,
    }
}

mod element_attrs;
mod merge;
mod ua;

pub use element_attrs::{element_attr, element_class, element_id, element_style_attr, element_tag};
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
/// 2. for each element compute its style from the matching rules
///    (consulting `tree.interaction` so dynamic pseudo-classes like
///    `:hover` / `:active` resolve correctly),
/// 3. layer the inline `style="…"` attribute on top, and
/// 4. inherit the standard inheriting properties from the parent's
///    resolved style (CSS-Cascade-3 §3.3 — `color`, font-related
///    properties, line-height, text-align, etc.).
pub fn cascade(tree: &Tree) -> CascadedTree {
    // UA defaults sit before author rules in the rules list, so on
    // a specificity tie the author rule wins on source order.
    let mut stylesheet = ua::ua_stylesheet().clone();
    stylesheet.append(collect_stylesheet(tree));
    let interaction = &tree.interaction;
    let mut path: Vec<usize> = Vec::new();
    CascadedTree {
        root: tree
            .root
            .as_ref()
            .map(|n| cascade_node(n, &stylesheet, None, &[], &mut path, interaction)),
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

/// Recursive cascade. `ancestors[0]` is the immediate parent element
/// (with its `MatchContext`), deeper indices going further up — used
/// by the selector matcher so descendant-combinator rules
/// (`.row .item`, `div:hover .child`) fire correctly.
///
/// `path` is the current element's child-index path from the root;
/// it's used to compute the per-element `MatchContext` against the
/// document's `InteractionState`.
fn cascade_node(
    node: &Node,
    sheet: &Stylesheet,
    parent_style: Option<&Style>,
    ancestors: &[(&Element, MatchContext)],
    path: &mut Vec<usize>,
    interaction: &InteractionState,
) -> CascadedNode {
    let element_ctx = MatchContext::for_path(path, interaction);
    let (mut style, keywords) =
        computed_decls_in_tree_with_context(&node.element, &element_ctx, sheet, ancestors);

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

    // Build the child ancestor chain by prepending this element.
    let mut child_ancestors: Vec<(&Element, MatchContext)> =
        Vec::with_capacity(ancestors.len() + 1);
    child_ancestors.push((&node.element, element_ctx));
    child_ancestors.extend_from_slice(ancestors);

    let children = node
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            path.push(i);
            let cn = cascade_node(c, sheet, Some(&style), &child_ancestors, path, interaction);
            path.pop();
            cn
        })
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
fn inherit_into(child: &mut Style, parent: &Style, keywords: &HashMap<String, CssWideKeyword>) {
    macro_rules! inherit {
        ($(($field:ident, $name:literal)),* $(,)?) => {
            $(
                if child.$field.is_none()
                    && !keywords.contains_key($name)
                    && !child.reset_properties.contains($name)
                    && !child.keyword_reset_properties.contains($name)
                {
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
    for (prop, value) in &parent.deferred_longhands {
        if child.deferred_longhands.contains_key(prop) || keywords.contains_key(prop) {
            continue;
        }
        if child.reset_properties.contains(prop) {
            continue;
        }
        if child.keyword_reset_properties.contains(prop) {
            continue;
        }
        if wgpu_html_parser::is_inherited(prop) {
            child.deferred_longhands.insert(prop.clone(), value.clone());
        }
    }
}

/// Compute the cascaded style for one element against a stylesheet,
/// dropping any CSS-wide keyword overrides. Kept for callers that
/// don't have a parent style on hand and just want the typed values.
/// Use [`computed_decls`] in the cascade itself.
///
/// This convenience does NOT evaluate descendant-combinator rules
/// (it has no ancestor context); for that, use the cascade walk in
/// [`cascade`].
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
    computed_decls_in_tree(element, sheet, &[])
}

/// Same as [`computed_decls`] but evaluates descendant-combinator
/// selectors against the supplied ancestor chain (`ancestors[0]` is
/// the immediate parent, deeper indices going further up). Uses a
/// default `MatchContext` so dynamic pseudo-class rules don't match.
pub fn computed_decls_in_tree(
    element: &Element,
    sheet: &Stylesheet,
    ancestors: &[&Element],
) -> (Style, HashMap<String, CssWideKeyword>) {
    let with_default: Vec<(&Element, MatchContext)> = ancestors
        .iter()
        .map(|e| (*e, MatchContext::default()))
        .collect();
    computed_decls_in_tree_with_context(element, &MatchContext::default(), sheet, &with_default)
}

/// Stateful variant of [`computed_decls_in_tree`]. Each ancestor is
/// paired with its own `MatchContext` so pseudo-class compounds on
/// ancestors (e.g. `div:hover .child`) resolve correctly.
pub fn computed_decls_in_tree_with_context(
    element: &Element,
    element_ctx: &MatchContext,
    sheet: &Stylesheet,
    ancestors: &[(&Element, MatchContext)],
) -> (Style, HashMap<String, CssWideKeyword>) {
    let mut values = Style::default();
    let mut keywords: HashMap<String, CssWideKeyword> = HashMap::new();
    let inline = element_style_attr(element).map(parse_inline_style_decls);

    let select_layers = |target: fn(
        &wgpu_html_parser::Rule,
    ) -> (&Style, &HashMap<String, CssWideKeyword>)|
     -> Vec<(u32, &Style, &HashMap<String, CssWideKeyword>)> {
        sheet
            .rules
            .iter()
            .filter_map(|rule| {
                rule.selectors
                    .iter()
                    .filter(|s| {
                        matches_selector_in_tree_with_context(s, element, element_ctx, ancestors)
                    })
                    .map(|s| s.specificity())
                    .max()
                    .map(|spec| {
                        let (decls, kws) = target(rule);
                        (spec, decls, kws)
                    })
            })
            .collect()
    };

    // 1. Author normal.
    let mut author_normal = select_layers(|r| (&r.declarations, &r.keywords));
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
    let mut author_important = select_layers(|r| (&r.important, &r.important_keywords));
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
    for prop in &layer_values.keyword_reset_properties {
        wgpu_html_parser::clear_value_for(prop, values);
        values.keyword_reset_properties.insert(prop.clone());
    }
    for (prop, kw) in layer_keywords {
        wgpu_html_parser::clear_value_for(prop, values);
        keywords.insert(prop.clone(), *kw);
    }
    wgpu_html_parser::merge_values_clearing_keywords(values, keywords, layer_values);
}

// ---------------------------------------------------------------------------
// Selector matching
// ---------------------------------------------------------------------------

/// Match the selector's *subject* compound against `element`. Selectors
/// that carry ancestor requirements (e.g. parsed from `.row .item`)
/// always fail this check — they need [`matches_selector_in_tree`]
/// with an ancestor chain to evaluate the descendant combinator.
///
/// Uses a default `MatchContext`, so any selector carrying a dynamic
/// pseudo-class (`:hover`, `:active`, …) fails. Use
/// [`matches_selector_with_context`] to check against live state.
pub fn matches_selector(sel: &Selector, element: &Element) -> bool {
    matches_selector_with_context(sel, element, &MatchContext::default())
}

/// Stateful variant of [`matches_selector`] — checks dynamic
/// pseudo-classes against the supplied `MatchContext`.
pub fn matches_selector_with_context(
    sel: &Selector,
    element: &Element,
    element_ctx: &MatchContext,
) -> bool {
    if !sel.ancestors.is_empty() {
        return false;
    }
    matches_compound(sel, element) && pseudo_classes_satisfied(sel, element_ctx)
}

/// Match `sel` against `element` with the element's ancestor chain
/// available. `ancestors[0]` must be the immediate parent, deeper
/// indices going further up to the root. Used by the cascade so
/// descendant-combinator selectors (`.row .item`) actually fire.
///
/// Dynamic pseudo-classes (`:hover`, `:active`) on the subject or
/// any ancestor compound fail without a `MatchContext`; use
/// [`matches_selector_in_tree_with_context`] for stateful matching.
pub fn matches_selector_in_tree(sel: &Selector, element: &Element, ancestors: &[&Element]) -> bool {
    let with_default: Vec<(&Element, MatchContext)> = ancestors
        .iter()
        .map(|e| (*e, MatchContext::default()))
        .collect();
    matches_selector_in_tree_with_context(sel, element, &MatchContext::default(), &with_default)
}

/// Stateful variant of [`matches_selector_in_tree`]. Each ancestor
/// carries its own `MatchContext`, so pseudo-class compounds on
/// ancestor selectors (`div:hover .child`) resolve correctly.
pub fn matches_selector_in_tree_with_context(
    sel: &Selector,
    element: &Element,
    element_ctx: &MatchContext,
    ancestors: &[(&Element, MatchContext)],
) -> bool {
    if !matches_compound(sel, element) || !pseudo_classes_satisfied(sel, element_ctx) {
        return false;
    }
    if sel.ancestors.is_empty() {
        return true;
    }
    // Walk `sel.ancestors` (closest required ancestor first) against
    // the element's actual ancestor chain. Each required ancestor
    // must be found *somewhere* up the chain; later requirements
    // continue from where the previous match left off (so the chain
    // order is preserved).
    let mut idx = 0usize;
    for required in &sel.ancestors {
        let mut matched = false;
        while idx < ancestors.len() {
            let (cand, cand_ctx) = ancestors[idx];
            idx += 1;
            if matches_compound(required, cand) && pseudo_classes_satisfied(required, &cand_ctx) {
                matched = true;
                break;
            }
        }
        if !matched {
            return false;
        }
    }
    true
}

/// Pure compound match: tag/id/classes/universal. Ignores the
/// `ancestors` list and any pseudo-classes; pseudo-classes are
/// gated separately by [`pseudo_classes_satisfied`].
fn matches_compound(sel: &Selector, element: &Element) -> bool {
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
    for attr in &sel.attributes {
        let Some(actual) = element_attr(element, &attr.name) else {
            return false;
        };
        if let Some(expected) = &attr.value
            && !actual.eq_ignore_ascii_case(expected)
        {
            return false;
        }
    }
    true
}

/// Verify every pseudo-class on `sel` holds in `ctx`. AND-semantics:
/// `a:hover:active` requires both. Selectors without pseudo-classes
/// pass trivially.
fn pseudo_classes_satisfied(sel: &Selector, ctx: &MatchContext) -> bool {
    for pc in &sel.pseudo_classes {
        let ok = match pc {
            PseudoClass::Hover => ctx.is_hover,
            PseudoClass::Active => ctx.is_active,
            PseudoClass::Focus => ctx.is_focus,
            PseudoClass::Visited => false,
        };
        if !ok {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
