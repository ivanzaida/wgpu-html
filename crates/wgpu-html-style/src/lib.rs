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

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use wgpu_html_models::Style;
use wgpu_html_parser::{
    CssWideKeyword, PseudoClass, Rule, Selector, Stylesheet, parse_inline_style_decls,
    parse_stylesheet,
};
use wgpu_html_tree::{Element, InteractionState, Node, Tree};

/// Per-element state consulted by the matcher when resolving dynamic
/// pseudo-classes (`:hover`, `:active`, …). Default is "nothing on";
/// rules with pseudo-classes never match a default context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MatchContext {
    pub is_hover: bool,
    pub is_active: bool,
    pub is_focus: bool,
    pub is_root: bool,
    pub is_first_child: bool,
    pub is_last_child: bool,
}

impl MatchContext {
    /// Compute the context for the element at `path` given the
    /// document's `InteractionState`. An element is "in the hover
    /// chain" iff its path is a prefix of `state.hover_path` (i.e. it
    /// is, or is an ancestor of, the deepest hovered element).
    pub fn for_path(path: &[usize], state: &InteractionState) -> Self {
        Self::for_path_with_siblings(path, state, None)
    }

    pub fn for_path_with_siblings(
        path: &[usize],
        state: &InteractionState,
        sibling_count: Option<usize>,
    ) -> Self {
        Self {
            is_hover: path_is_prefix(path, state.hover_path.as_deref()),
            is_active: path_is_prefix(path, state.active_path.as_deref()),
            is_focus: state.focus_path.as_deref() == Some(path),
            is_root: path.is_empty(),
            is_first_child: path.last().copied() == Some(0),
            is_last_child: match (path.last(), sibling_count) {
                (Some(&idx), Some(count)) => idx + 1 == count,
                _ => false,
            },
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

#[derive(Debug, Clone)]
struct PreparedStylesheet {
    sheet: Arc<Stylesheet>,
    index: RuleIndex,
    normal_nonempty: Vec<bool>,
    important_nonempty: Vec<bool>,
    relevant: RelevantSelectors,
}

#[derive(Debug, Clone, Default)]
struct RelevantSelectors {
    ids: HashSet<String>,
    classes: HashSet<String>,
    tags: HashSet<String>,
    attrs: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
struct RuleIndex {
    by_id: HashMap<String, Vec<SelectorRuleRef>>,
    by_class: HashMap<String, Vec<SelectorRuleRef>>,
    by_tag: HashMap<String, Vec<SelectorRuleRef>>,
    universal: Vec<SelectorRuleRef>,
}

#[derive(Debug, Clone, Copy)]
struct SelectorRuleRef {
    rule_idx: usize,
    selector_idx: usize,
}

impl PreparedStylesheet {
    fn from_sheet(sheet: Arc<Stylesheet>) -> Self {
        let mut index = RuleIndex::default();
        let mut normal_nonempty = Vec::with_capacity(sheet.rules.len());
        let mut important_nonempty = Vec::with_capacity(sheet.rules.len());
        let mut relevant = RelevantSelectors::default();
        for (rule_idx, rule) in sheet.rules.iter().enumerate() {
            normal_nonempty.push(!rule.keywords.is_empty() || style_has_values(&rule.declarations));
            important_nonempty
                .push(!rule.important_keywords.is_empty() || style_has_values(&rule.important));
            for (selector_idx, selector) in rule.selectors.iter().enumerate() {
                collect_relevant_selector_bits(selector, &mut relevant);
                let entry = SelectorRuleRef {
                    rule_idx,
                    selector_idx,
                };
                if let Some(id) = &selector.id {
                    index.by_id.entry(id.clone()).or_default().push(entry);
                } else if let Some(class) = selector.classes.first() {
                    index.by_class.entry(class.clone()).or_default().push(entry);
                } else if let Some(tag) = &selector.tag {
                    index.by_tag.entry(tag.clone()).or_default().push(entry);
                } else {
                    index.universal.push(entry);
                }
            }
        }
        Self {
            sheet,
            index,
            normal_nonempty,
            important_nonempty,
            relevant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DeclCacheKey {
    element: SelectorSignature,
    ancestors: Vec<SelectorSignature>,
    inline_style: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SelectorSignature {
    tag: Option<String>,
    id: Option<String>,
    classes: Vec<String>,
    attrs: Vec<(String, Option<String>)>,
    ctx: MatchContext,
}

fn collect_relevant_selector_bits(sel: &Selector, relevant: &mut RelevantSelectors) {
    if let Some(id) = &sel.id {
        relevant.ids.insert(id.clone());
    }
    if let Some(tag) = &sel.tag {
        relevant.tags.insert(tag.clone());
    }
    relevant.classes.extend(sel.classes.iter().cloned());
    relevant
        .attrs
        .extend(sel.attributes.iter().map(|attr| attr.name.clone()));
    for ancestor in &sel.ancestors {
        collect_relevant_selector_bits(ancestor, relevant);
    }
}

fn decl_cache_key(
    element: &Element,
    element_ctx: MatchContext,
    sheets: &[&PreparedStylesheet],
    ancestors: &[(&Element, MatchContext)],
) -> DeclCacheKey {
    DeclCacheKey {
        element: selector_signature(element, element_ctx, sheets),
        ancestors: ancestors
            .iter()
            .map(|(ancestor, ctx)| selector_signature(ancestor, *ctx, sheets))
            .collect(),
        inline_style: element_style_attr(element).map(str::to_owned),
    }
}

fn selector_signature(
    element: &Element,
    ctx: MatchContext,
    sheets: &[&PreparedStylesheet],
) -> SelectorSignature {
    let tag = element_tag(element)
        .filter(|tag| relevant_tag(sheets, tag))
        .map(str::to_owned);
    let id = element_id(element)
        .filter(|id| relevant_id(sheets, id))
        .map(str::to_owned);
    let mut classes = element_class(element)
        .into_iter()
        .flat_map(|class_attr| class_attr.split_ascii_whitespace())
        .filter(|class| relevant_class(sheets, class))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    classes.sort_unstable();
    classes.dedup();

    let mut attr_names = relevant_attr_names(sheets);
    let attrs = attr_names
        .drain(..)
        .map(|name| {
            let value = element_attr(element, &name);
            (name, value)
        })
        .collect();

    SelectorSignature {
        tag,
        id,
        classes,
        attrs,
        ctx,
    }
}

fn relevant_id(sheets: &[&PreparedStylesheet], id: &str) -> bool {
    sheets.iter().any(|sheet| sheet.relevant.ids.contains(id))
}

fn relevant_class(sheets: &[&PreparedStylesheet], class: &str) -> bool {
    sheets
        .iter()
        .any(|sheet| sheet.relevant.classes.contains(class))
}

fn relevant_tag(sheets: &[&PreparedStylesheet], tag: &str) -> bool {
    sheets.iter().any(|sheet| sheet.relevant.tags.contains(tag))
}

fn relevant_attr_names(sheets: &[&PreparedStylesheet]) -> Vec<String> {
    let mut names = sheets
        .iter()
        .flat_map(|sheet| sheet.relevant.attrs.iter().cloned())
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
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
    // UA defaults sit before author rules, so on a specificity tie
    // the author rule wins on source order.
    let author = collect_prepared_stylesheet_cached(tree);
    let stylesheets = [ua_prepared_stylesheet(), author.as_ref()];
    let interaction = &tree.interaction;
    let mut path: Vec<usize> = Vec::new();
    let mut decl_cache: HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)> =
        HashMap::new();
    CascadedTree {
        root: tree.root.as_ref().map(|n| {
            cascade_node(
                n,
                &stylesheets,
                None,
                &[],
                &mut path,
                interaction,
                &mut decl_cache,
                None, // root has no siblings
            )
        }),
    }
}

/// Walk the tree, gather text content of all `<style>` blocks, and parse it.
pub fn collect_stylesheet(tree: &Tree) -> Stylesheet {
    collect_prepared_stylesheet_cached(tree)
        .sheet
        .as_ref()
        .clone()
}

fn collect_prepared_stylesheet_cached(tree: &Tree) -> Arc<PreparedStylesheet> {
    let css = collect_stylesheet_source(tree);
    if css.is_empty() {
        return Arc::new(PreparedStylesheet::from_sheet(Arc::new(
            Stylesheet::default(),
        )));
    }
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<PreparedStylesheet>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if let Some(sheet) = cache.get(&css) {
            return sheet.clone();
        }
        let prepared = Arc::new(PreparedStylesheet::from_sheet(Arc::new(parse_stylesheet(
            &css,
        ))));
        cache.insert(css, prepared.clone());
        return prepared;
    }
    Arc::new(PreparedStylesheet::from_sheet(Arc::new(parse_stylesheet(
        &css,
    ))))
}

fn ua_prepared_stylesheet() -> &'static PreparedStylesheet {
    static UA: OnceLock<PreparedStylesheet> = OnceLock::new();
    UA.get_or_init(|| PreparedStylesheet::from_sheet(Arc::new(ua::ua_stylesheet().clone())))
}

fn collect_stylesheet_source(tree: &Tree) -> String {
    let mut css = String::new();
    if let Some(root) = &tree.root {
        gather(root, &mut css);
    }
    css
}

fn gather(node: &Node, out: &mut String) {
    if matches!(&node.element, Element::StyleElement(_)) {
        for child in &node.children {
            if let Element::Text(t) = &child.element {
                out.push_str(t);
            }
        }
        out.push('\n');
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
    sheets: &[&PreparedStylesheet],
    parent_style: Option<&Style>,
    ancestors: &[(&Element, MatchContext)],
    path: &mut Vec<usize>,
    interaction: &InteractionState,
    decl_cache: &mut HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)>,
    sibling_count: Option<usize>,
) -> CascadedNode {
    let element_ctx = MatchContext::for_path_with_siblings(path, interaction, sibling_count);
    let (mut style, keywords) = if matches!(node.element, Element::Text(_)) {
        (Style::default(), HashMap::new())
    } else {
        let key = decl_cache_key(&node.element, element_ctx, sheets, ancestors);
        if let Some(cached) = decl_cache.get(&key) {
            cached.clone()
        } else {
            let computed = computed_decls_in_prepared_stylesheets_with_context(
                &node.element,
                &element_ctx,
                sheets,
                ancestors,
            );
            decl_cache.insert(key, computed.clone());
            computed
        }
    };

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

    // Inject programmatic custom properties from the Node. These act
    // like inline-style declarations and override any inherited values.
    for (prop, value) in &node.custom_properties {
        style.custom_properties.insert(prop.clone(), value.clone());
    }

    // Resolve var() references now that custom properties are final.
    if !style.var_properties.is_empty() || style.custom_properties.values().any(|v| v.contains("var(")) {
        wgpu_html_parser::resolve_var_references(&mut style);
    }

    // Build the child ancestor chain by prepending this element.
    let mut child_ancestors: Vec<(&Element, MatchContext)> =
        Vec::with_capacity(ancestors.len() + 1);
    child_ancestors.push((&node.element, element_ctx));
    child_ancestors.extend_from_slice(ancestors);

    let child_count = node.children.len();
    let children = node
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| {
            path.push(i);
            let cn = cascade_node(
                c,
                sheets,
                Some(&style),
                &child_ancestors,
                path,
                interaction,
                decl_cache,
                Some(child_count),
            );
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
    // Custom properties always inherit.
    for (prop, value) in &parent.custom_properties {
        if !child.custom_properties.contains_key(prop) && !keywords.contains_key(prop) {
            child.custom_properties.insert(prop.clone(), value.clone());
        }
    }
    // Inherit var_properties for inherited CSS properties.
    for (prop, value) in &parent.var_properties {
        if child.var_properties.contains_key(prop) || keywords.contains_key(prop) {
            continue;
        }
        if child.reset_properties.contains(prop) || child.keyword_reset_properties.contains(prop) {
            continue;
        }
        if wgpu_html_parser::is_inherited(prop) {
            child.var_properties.insert(prop.clone(), value.clone());
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
    let prepared = PreparedStylesheet::from_sheet(Arc::new(sheet.clone()));
    computed_decls_in_prepared_stylesheets_with_context(
        element,
        element_ctx,
        &[&prepared],
        ancestors,
    )
}

fn computed_decls_in_prepared_stylesheets_with_context(
    element: &Element,
    element_ctx: &MatchContext,
    sheets: &[&PreparedStylesheet],
    ancestors: &[(&Element, MatchContext)],
) -> (Style, HashMap<String, CssWideKeyword>) {
    let mut values = Style::default();
    let mut keywords: HashMap<String, CssWideKeyword> = HashMap::new();
    let inline = element_style_attr(element).map(parse_inline_style_decls);
    let tag = element_tag(element);
    let id = element_id(element);
    let class_attr = element_class(element);

    let mut matched_rules: Vec<(u32, usize, usize, &Rule, bool, bool)> = sheets
        .iter()
        .enumerate()
        .flat_map(|(sheet_idx, sheet)| {
            matching_rules_for_element(sheet, element, element_ctx, ancestors, tag, id, class_attr)
                .into_iter()
                .map(
                    move |(spec, rule_idx, rule, normal_nonempty, important_nonempty)| {
                        (
                            spec,
                            sheet_idx,
                            rule_idx,
                            rule,
                            normal_nonempty,
                            important_nonempty,
                        )
                    },
                )
        })
        .collect();
    matched_rules
        .sort_by_key(|(spec, sheet_idx, rule_idx, _, _, _)| (*spec, *sheet_idx, *rule_idx));

    // 1. Author normal.
    for (_, _, _, rule, normal_nonempty, _) in &matched_rules {
        if *normal_nonempty {
            apply_layer(
                &mut values,
                &mut keywords,
                &rule.declarations,
                &rule.keywords,
            );
        }
    }

    // 2. Inline normal.
    if let Some(decls) = &inline {
        apply_layer_if_nonempty(
            &mut values,
            &mut keywords,
            &decls.normal,
            &decls.keywords_normal,
        );
    }

    // 3. Author !important.
    for (_, _, _, rule, _, important_nonempty) in &matched_rules {
        if *important_nonempty {
            apply_layer(
                &mut values,
                &mut keywords,
                &rule.important,
                &rule.important_keywords,
            );
        }
    }

    // 4. Inline !important.
    if let Some(decls) = &inline {
        apply_layer_if_nonempty(
            &mut values,
            &mut keywords,
            &decls.important,
            &decls.keywords_important,
        );
    }

    (values, keywords)
}

fn selector_prefilter_is_complete(sel: &Selector) -> bool {
    sel.ancestors.is_empty() && sel.attributes.is_empty() && sel.pseudo_classes.is_empty()
}

fn matching_rules_for_element<'a>(
    sheet: &'a PreparedStylesheet,
    element: &Element,
    element_ctx: &MatchContext,
    ancestors: &[(&Element, MatchContext)],
    tag: Option<&str>,
    id: Option<&str>,
    class_attr: Option<&str>,
) -> Vec<(u32, usize, &'a Rule, bool, bool)> {
    let mut selector_entries = Vec::new();
    let mut push_entries = |entries: &[SelectorRuleRef]| {
        for entry in entries {
            if !selector_entries.iter().any(|seen: &SelectorRuleRef| {
                seen.rule_idx == entry.rule_idx && seen.selector_idx == entry.selector_idx
            }) {
                selector_entries.push(*entry);
            }
        }
    };

    if let Some(id) = id
        && let Some(entries) = sheet.index.by_id.get(id)
    {
        push_entries(entries);
    }
    if let Some(class_attr) = class_attr {
        for class in class_attr.split_ascii_whitespace() {
            if let Some(entries) = sheet.index.by_class.get(class) {
                push_entries(entries);
            }
        }
    }
    if let Some(tag) = tag
        && let Some(entries) = sheet.index.by_tag.get(tag)
    {
        push_entries(entries);
    }
    push_entries(&sheet.index.universal);

    let mut rule_specs: Vec<(usize, u32)> = Vec::new();
    for entry in selector_entries {
        let Some(rule) = sheet.sheet.rules.get(entry.rule_idx) else {
            continue;
        };
        let Some(selector) = rule.selectors.get(entry.selector_idx) else {
            continue;
        };
        if !selector_subject_might_match(selector, tag, id, class_attr) {
            continue;
        }
        if !selector_prefilter_is_complete(selector)
            && !matches_selector_in_tree_with_context(selector, element, element_ctx, ancestors)
        {
            continue;
        }
        let spec = selector.specificity();
        if let Some((_, prev)) = rule_specs
            .iter_mut()
            .find(|(rule_idx, _)| *rule_idx == entry.rule_idx)
        {
            *prev = (*prev).max(spec);
        } else {
            rule_specs.push((entry.rule_idx, spec));
        }
    }

    rule_specs
        .into_iter()
        .filter_map(|(rule_idx, spec)| {
            let rule = sheet.sheet.rules.get(rule_idx)?;
            let normal_nonempty = sheet.normal_nonempty.get(rule_idx).copied().unwrap_or(true);
            let important_nonempty = sheet
                .important_nonempty
                .get(rule_idx)
                .copied()
                .unwrap_or(true);
            Some((spec, rule_idx, rule, normal_nonempty, important_nonempty))
        })
        .collect()
}

fn selector_subject_might_match(
    sel: &Selector,
    tag: Option<&str>,
    id: Option<&str>,
    class_attr: Option<&str>,
) -> bool {
    if let Some(needed_tag) = &sel.tag
        && tag != Some(needed_tag.as_str())
    {
        return false;
    }
    if let Some(needed_id) = &sel.id
        && id != Some(needed_id.as_str())
    {
        return false;
    }
    if !sel.classes.is_empty() {
        let Some(class_attr) = class_attr else {
            return false;
        };
        for needed in &sel.classes {
            if !class_attr
                .split_ascii_whitespace()
                .any(|class| class == needed)
            {
                return false;
            }
        }
    }
    true
}

/// Apply one cascade layer's `(values, keyword overrides)` to the
/// running `(values, keywords)` accumulator. Keyword overrides go
/// first — they clear the matching value field — then the value
/// merge runs and drops any keyword left behind for fields the layer
/// also wrote a value for.
fn apply_layer_if_nonempty(
    values: &mut Style,
    keywords: &mut HashMap<String, CssWideKeyword>,
    layer_values: &Style,
    layer_keywords: &HashMap<String, CssWideKeyword>,
) {
    if layer_keywords.is_empty() && !style_has_values(layer_values) {
        return;
    }
    apply_layer(values, keywords, layer_values, layer_keywords);
}

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
    // merge_values_clearing_keywords handles custom_properties,
    // var_properties, and their interplay with typed fields internally.
    wgpu_html_parser::merge_values_clearing_keywords(values, keywords, layer_values);
}

fn style_has_values(style: &Style) -> bool {
    macro_rules! any_option {
        ($($field:ident),* $(,)?) => {
            false $(|| style.$field.is_some())*
        };
    }
    any_option!(
        display,
        position,
        top,
        right,
        bottom,
        left,
        width,
        height,
        min_width,
        min_height,
        max_width,
        max_height,
        margin,
        margin_top,
        margin_right,
        margin_bottom,
        margin_left,
        padding,
        padding_top,
        padding_right,
        padding_bottom,
        padding_left,
        color,
        background,
        background_color,
        background_image,
        background_size,
        background_position,
        background_repeat,
        background_clip,
        border,
        border_top_width,
        border_right_width,
        border_bottom_width,
        border_left_width,
        border_top_style,
        border_right_style,
        border_bottom_style,
        border_left_style,
        border_top_color,
        border_right_color,
        border_bottom_color,
        border_left_color,
        border_top_left_radius,
        border_top_right_radius,
        border_bottom_right_radius,
        border_bottom_left_radius,
        border_top_left_radius_v,
        border_top_right_radius_v,
        border_bottom_right_radius_v,
        border_bottom_left_radius_v,
        font_family,
        font_size,
        font_weight,
        font_style,
        line_height,
        letter_spacing,
        text_align,
        text_decoration,
        text_transform,
        white_space,
        overflow,
        overflow_x,
        overflow_y,
        opacity,
        visibility,
        z_index,
        flex_direction,
        flex_wrap,
        justify_content,
        align_items,
        align_content,
        align_self,
        order,
        gap,
        row_gap,
        column_gap,
        flex,
        flex_grow,
        flex_shrink,
        flex_basis,
        grid_template_columns,
        grid_template_rows,
        grid_auto_columns,
        grid_auto_rows,
        grid_auto_flow,
        grid_column_start,
        grid_column_end,
        grid_row_start,
        grid_row_end,
        grid_column,
        grid_row,
        justify_items,
        justify_self,
        transform,
        transform_origin,
        transition,
        animation,
        cursor,
        pointer_events,
        user_select,
        box_shadow,
        box_sizing,
    ) || !style.deferred_longhands.is_empty()
        || !style.reset_properties.is_empty()
        || !style.keyword_reset_properties.is_empty()
        || !style.custom_properties.is_empty()
        || !style.var_properties.is_empty()
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
            PseudoClass::Root => ctx.is_root,
            PseudoClass::FirstChild => ctx.is_first_child,
            PseudoClass::LastChild => ctx.is_last_child,
        };
        if !ok {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests;
