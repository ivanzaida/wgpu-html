use std::cell::{Cell, UnsafeCell};

use bumpalo::Bump;
use lui_css_parser::{
    ArcStr, CssProperty, CssValue, Stylesheet, StyleRule,
    longhands_of, expand_shorthand,
};
use lui_html_parser::HtmlNode;

use crate::StyledNode;
use crate::index::{PreparedStylesheet, candidate_rules, RuleCondition};
use crate::inline::node_inline_style;
use crate::matching::{AncestorEntry, MatchContext, matches_selector};
use crate::media::{MediaContext, evaluate_media, evaluate_supports};
use crate::style::ComputedStyle;
use crate::var_resolution::resolve_vars;

#[derive(Debug, Clone, Default)]
pub struct InteractionState {
    pub hover_path: Option<Vec<usize>>,
    pub active_path: Option<Vec<usize>>,
    pub focus_path: Option<Vec<usize>>,
    /// URL fragment (the part after `#`). Matched by `:target`.
    pub target_id: Option<String>,
}

/// Persistent context for the cascade engine. Caches prepared stylesheets
/// and uses double-buffered arenas so incremental cascades can reference
/// the previous frame's results while building new ones.
///
/// Call `set_stylesheets` when CSS changes, then `cascade` or
/// `cascade_dirty` each frame.
pub struct CascadeContext {
    arena_a: UnsafeCell<Bump>,
    arena_b: UnsafeCell<Bump>,
    use_a: Cell<bool>,
    prepared: Vec<PreparedStylesheet>,
}

impl CascadeContext {
    pub fn new() -> Self {
        Self {
            arena_a: UnsafeCell::new(Bump::new()),
            arena_b: UnsafeCell::new(Bump::new()),
            use_a: Cell::new(true),
            prepared: Vec::new(),
        }
    }

    /// Prepare stylesheets for cascading. Call this when stylesheets change,
    /// not every frame.
    pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
        self.prepared = sheets.iter()
            .map(|s| PreparedStylesheet::new(s.clone()))
            .collect();
    }

    fn current_arena(&self) -> &Bump {
        // SAFETY: only the current arena is written to; the other is read-only.
        unsafe {
            if self.use_a.get() { &*self.arena_a.get() } else { &*self.arena_b.get() }
        }
    }

    fn swap_and_reset(&self) {
        let next = !self.use_a.get();
        self.use_a.set(next);
        // SAFETY: we reset only the arena we're about to write to.
        // No references into it exist — caller must drop previous results
        // from this arena before calling cascade/cascade_dirty again.
        unsafe {
            let arena = if next { &mut *self.arena_a.get() } else { &mut *self.arena_b.get() };
            arena.reset();
        }
    }

    /// Full cascade — recomputes every node.
    /// Drop the returned `StyledNode` before calling `cascade` again.
    pub fn cascade<'a>(
        &'a self,
        doc: &'a HtmlNode,
        media: &MediaContext,
        interaction: &'a InteractionState,
    ) -> StyledNode<'a> {
        self.swap_and_reset();
        let arena = self.current_arena();

        let sheets: Vec<&PreparedStylesheet> = self.prepared.iter().collect();
        let mut path: Vec<usize> = Vec::new();

        cascade_node(
            doc, doc, &sheets, None, &[], &mut path,
            media, interaction, arena,
        )
    }

    /// Incremental cascade — only recomputes dirty subtrees.
    /// Clean subtrees are deep-copied from `prev` (no rule matching).
    ///
    /// `prev` borrows from the previous arena; this writes to the current one.
    pub fn cascade_dirty<'a>(
        &'a self,
        doc: &'a HtmlNode,
        prev: &StyledNode<'a>,
        dirty_paths: &[Vec<usize>],
        media: &MediaContext,
        interaction: &'a InteractionState,
    ) -> StyledNode<'a> {
        self.swap_and_reset();
        let arena = self.current_arena();

        let sheets: Vec<&PreparedStylesheet> = self.prepared.iter().collect();
        let mut path: Vec<usize> = Vec::new();

        cascade_node_incremental(
            doc, doc, &sheets, None, &[], &mut path,
            media, interaction, arena,
            prev, dirty_paths,
        )
    }
}

impl Default for CascadeContext {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
fn is_path_dirty(path: &[usize], dirty_paths: &[Vec<usize>]) -> bool {
    dirty_paths.iter().any(|dp| {
        // Dirty if: path is a prefix of dp (ancestor of dirty node),
        // dp is a prefix of path (dirty node or descendant), or exact match.
        path.starts_with(dp) || dp.starts_with(path)
    })
}

// ---------------------------------------------------------------------------
// Internal cascade
// ---------------------------------------------------------------------------

fn cascade_node_incremental<'a>(
    root: &'a HtmlNode,
    node: &'a HtmlNode,
    sheets: &[&PreparedStylesheet],
    parent_style: Option<&ComputedStyle<'a>>,
    ancestors: &[AncestorEntry<'a>],
    path: &mut Vec<usize>,
    media: &MediaContext,
    interaction: &'a InteractionState,
    arena: &'a Bump,
    prev: &StyledNode<'_>,
    dirty_paths: &[Vec<usize>],
) -> StyledNode<'a> {
    if !is_path_dirty(path, dirty_paths) {
        return clone_subtree(node, prev, parent_style, arena);
    }

    cascade_node_with_prev(
        root, node, sheets, parent_style, ancestors, path,
        media, interaction, arena, Some(prev), dirty_paths,
    )
}

fn cascade_node_with_prev<'a>(
    root: &'a HtmlNode,
    node: &'a HtmlNode,
    sheets: &[&PreparedStylesheet],
    parent_style: Option<&ComputedStyle<'a>>,
    ancestors: &[AncestorEntry<'a>],
    path: &mut Vec<usize>,
    media: &MediaContext,
    interaction: &'a InteractionState,
    arena: &'a Bump,
    prev: Option<&StyledNode<'_>>,
    dirty_paths: &[Vec<usize>],
) -> StyledNode<'a> {
    let ctx = build_match_context(node, path, interaction, ancestors);

    let mut style = if node.element.is_text() || node.element.is_comment() {
        ComputedStyle::default()
    } else {
        compute_style(node, &ctx, sheets, ancestors, media, arena)
    };

    if let Some(parent) = parent_style {
        style.inherit_from(parent);
    }

    resolve_vars(&mut style, arena);

    let mut child_ancestors = Vec::with_capacity(ancestors.len() + 1);
    child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
    child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
        node: a.node,
        ctx: a.ctx.clone(),
    }));

    let children: Vec<StyledNode<'a>> = node.children.iter().enumerate().map(|(i, child)| {
        path.push(i);
        let prev_child = prev.and_then(|p| p.children.get(i));
        let child_node = if let Some(pc) = prev_child {
            if !is_path_dirty(path, dirty_paths) {
                clone_subtree(child, pc, Some(&style), arena)
            } else {
                cascade_node_with_prev(
                    root, child, sheets, Some(&style), &child_ancestors,
                    path, media, interaction, arena,
                    Some(pc), dirty_paths,
                )
            }
        } else {
            cascade_node(
                root, child, sheets, Some(&style), &child_ancestors,
                path, media, interaction, arena,
            )
        };
        path.pop();
        child_node
    }).collect();

    StyledNode {
        node,
        style,
        children,
        before: None,
        after: None,
        first_line: None,
        first_letter: None,
        placeholder: None,
        selection: None,
        marker: None,
    }
}

fn clone_subtree<'a>(
    node: &'a HtmlNode,
    prev: &StyledNode<'_>,
    parent_style: Option<&ComputedStyle<'a>>,
    arena: &'a Bump,
) -> StyledNode<'a> {
    let mut style = prev.style.clone_into(arena);

    if let Some(parent) = parent_style {
        style.inherit_from(parent);
    }

    let children: Vec<StyledNode<'a>> = node.children.iter().enumerate().map(|(i, child)| {
        if let Some(prev_child) = prev.children.get(i) {
            clone_subtree(child, prev_child, Some(&style), arena)
        } else {
            StyledNode {
                node: child,
                style: ComputedStyle::default(),
                children: Vec::new(),
                before: None, after: None, first_line: None,
                first_letter: None, placeholder: None,
                selection: None, marker: None,
            }
        }
    }).collect();

    StyledNode {
        node,
        style,
        children,
        before: None,
        after: None,
        first_line: None,
        first_letter: None,
        placeholder: None,
        selection: None,
        marker: None,
    }
}

fn cascade_node<'a>(
    root: &'a HtmlNode,
    node: &'a HtmlNode,
    sheets: &[&PreparedStylesheet],
    parent_style: Option<&ComputedStyle<'a>>,
    ancestors: &[AncestorEntry<'a>],
    path: &mut Vec<usize>,
    media: &MediaContext,
    interaction: &'a InteractionState,
    arena: &'a Bump,
) -> StyledNode<'a> {
    let ctx = build_match_context(node, path, interaction, ancestors);

    let mut style = if node.element.is_text() || node.element.is_comment() {
        ComputedStyle::default()
    } else {
        compute_style(node, &ctx, sheets, ancestors, media, arena)
    };

    if let Some(parent) = parent_style {
        style.inherit_from(parent);
    }

    resolve_vars(&mut style, arena);

    let mut child_ancestors = Vec::with_capacity(ancestors.len() + 1);
    child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
    child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
        node: a.node,
        ctx: a.ctx.clone(),
    }));

    let children: Vec<StyledNode<'a>> = node.children.iter().enumerate().map(|(i, child)| {
        path.push(i);
        let child_node = cascade_node(
            root, child, sheets, Some(&style), &child_ancestors,
            path, media, interaction, arena,
        );
        path.pop();
        child_node
    }).collect();

    StyledNode {
        node,
        style,
        children,
        before: None,
        after: None,
        first_line: None,
        first_letter: None,
        placeholder: None,
        selection: None,
        marker: None,
    }
}

fn compute_style<'a>(
    node: &'a HtmlNode,
    ctx: &MatchContext<'_>,
    sheets: &[&PreparedStylesheet],
    ancestors: &[AncestorEntry<'_>],
    media: &MediaContext,
    arena: &'a Bump,
) -> ComputedStyle<'a> {
    let tag = node.element.tag_name();
    let id = node.attrs.get("id").map(|s| s.as_ref());
    let class_attr = node.attrs.get("class").map(|s| s.as_ref()).unwrap_or("");
    let classes: Vec<&str> = if class_attr.is_empty() {
        vec![]
    } else {
        class_attr.split_ascii_whitespace().collect()
    };

    let mut matched = collect_matching_rules(
        node, ctx, sheets, ancestors, media, tag, id, &classes,
    );

    matched.sort_by_key(|m| (m.sheet_idx, m.specificity, m.rule_idx));

    let inline_decls = node_inline_style(node);

    let mut style = ComputedStyle::default();

    // Layer 1: author normal (ascending specificity)
    for m in &matched {
        for decl in &m.rule.declarations {
            if !decl.important {
                apply_declaration(&mut style, &decl.property, &decl.value, arena);
            }
        }
    }

    // Layer 2: inline normal
    for decl in &inline_decls {
        if !decl.important {
            apply_declaration(&mut style, &decl.property, &decl.value, arena);
        }
    }

    // Layer 3: author !important (ascending specificity)
    for m in &matched {
        for decl in &m.rule.declarations {
            if decl.important {
                apply_declaration(&mut style, &decl.property, &decl.value, arena);
            }
        }
    }

    // Layer 4: inline !important
    for decl in &inline_decls {
        if decl.important {
            apply_declaration(&mut style, &decl.property, &decl.value, arena);
        }
    }

    // Custom properties from declarations
    for m in &matched {
        for decl in &m.rule.declarations {
            if let CssProperty::Unknown(ref name) = decl.property {
                if name.starts_with("--") {
                    let val = arena.alloc(decl.value.clone());
                    style.custom_properties
                        .get_or_insert_with(Default::default)
                        .insert(ArcStr::from(name.as_str()), val);
                }
            }
        }
    }
    for decl in &inline_decls {
        if let CssProperty::Unknown(ref name) = decl.property {
            if name.starts_with("--") {
                let val = arena.alloc(decl.value.clone());
                style.custom_properties
                    .get_or_insert_with(Default::default)
                    .insert(ArcStr::from(name.as_str()), val);
            }
        }
    }

    style
}

struct MatchedRule<'a> {
    rule: &'a StyleRule,
    specificity: (u32, u32, u32),
    sheet_idx: usize,
    rule_idx: usize,
}

fn collect_matching_rules<'a>(
    node: &HtmlNode,
    ctx: &MatchContext<'_>,
    sheets: &[&'a PreparedStylesheet],
    ancestors: &[AncestorEntry<'_>],
    media: &MediaContext,
    tag: &str,
    id: Option<&str>,
    classes: &[&str],
) -> Vec<MatchedRule<'a>> {
    let mut matched = Vec::new();

    for (sheet_idx, sheet) in sheets.iter().enumerate() {
        let candidates = candidate_rules(&sheet.index, tag, id, classes);
        for rule_ref in candidates {
            let rule = &sheet.rules[rule_ref.rule_idx];
            if let Some(specificity) = matched_specificity(rule, node, ctx, ancestors) {
                matched.push(MatchedRule {
                    rule,
                    specificity,
                    sheet_idx,
                    rule_idx: rule_ref.rule_idx,
                });
            }
        }

        let cond_candidates = candidate_rules(&sheet.conditional_index, tag, id, classes);
        for rule_ref in cond_candidates {
            let cond_rule = &sheet.conditional_rules[rule_ref.rule_idx];
            let condition = &sheet.conditions[cond_rule.condition_idx];
            let condition_met = match condition {
                RuleCondition::Media(mql) => evaluate_media(mql, media),
                RuleCondition::Supports(sc) => evaluate_supports(sc),
            };
            if !condition_met {
                continue;
            }
            if let Some(specificity) = matched_specificity(&cond_rule.rule, node, ctx, ancestors) {
                matched.push(MatchedRule {
                    rule: &cond_rule.rule,
                    specificity,
                    sheet_idx,
                    rule_idx: rule_ref.rule_idx + sheet.rules.len(),
                });
            }
        }
    }

    matched
}

fn matched_specificity(
    rule: &StyleRule,
    node: &HtmlNode,
    ctx: &MatchContext<'_>,
    ancestors: &[AncestorEntry<'_>],
) -> Option<(u32, u32, u32)> {
    let parent = ancestors.first().map(|a| a.node);
    for sel in &rule.selector.0 {
        if matches_selector(sel, node, ctx, ancestors, parent) {
            return Some(sel.specificity());
        }
    }
    None
}

fn apply_declaration<'a>(
    style: &mut ComputedStyle<'a>,
    prop: &CssProperty,
    value: &CssValue,
    arena: &'a Bump,
) {
    let longhands = longhands_of(prop.clone());
    if longhands.is_empty() {
        let val_ref = arena.alloc(value.clone());
        style.set(prop, val_ref);
    } else {
        let expanded = expand_shorthand(prop.clone(), &[value.clone()]);
        for (lh_prop, lh_value) in expanded {
            apply_declaration(style, &lh_prop, &lh_value, arena);
        }
    }
}

fn build_match_context<'a>(
    node: &'a HtmlNode,
    path: &[usize],
    interaction: &'a InteractionState,
    ancestors: &[AncestorEntry<'a>],
) -> MatchContext<'a> {
    let sibling_index = path.last().copied().unwrap_or(0);
    let sibling_count = ancestors.first()
        .map(|a| a.node.children.len())
        .unwrap_or(1);

    let is_hover = interaction.hover_path.as_ref()
        .map(|hp| path.starts_with(hp) || hp.starts_with(path))
        .unwrap_or(false);
    let is_active = interaction.active_path.as_ref()
        .map(|ap| path == ap.as_slice())
        .unwrap_or(false);
    let is_focus = interaction.focus_path.as_ref()
        .map(|fp| path == fp.as_slice())
        .unwrap_or(false);
    let is_focus_within = interaction.focus_path.as_ref()
        .map(|fp| fp.starts_with(path))
        .unwrap_or(false);

    let lang = node.attrs.get("lang").map(|s| s.as_ref())
        .or_else(|| ancestors.iter().find_map(|a| a.ctx.lang));

    let dir = node.attrs.get("dir")
        .and_then(|s| match s.as_ref() {
            "ltr" => Some(crate::matching::Dir::Ltr),
            "rtl" => Some(crate::matching::Dir::Rtl),
            _ => None,
        })
        .or_else(|| ancestors.iter().find_map(|a| a.ctx.dir));

    MatchContext {
        is_root: path.is_empty(),
        is_first_child: sibling_index == 0,
        is_last_child: sibling_index == sibling_count.saturating_sub(1),
        is_only_child: sibling_count == 1,
        sibling_index,
        sibling_count,
        is_hover,
        is_active,
        is_focus,
        is_focus_visible: is_focus,
        is_focus_within,
        target_id: interaction.target_id.as_deref(),
        lang,
        dir,
        is_fullscreen: false,
        is_modal: false,
    }
}
