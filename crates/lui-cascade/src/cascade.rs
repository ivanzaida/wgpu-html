use std::{
  cell::{Cell, UnsafeCell},
  hash::{Hash, Hasher},
};

use bumpalo::Bump;
use lui_css_parser::{expand_shorthand, longhands_of, ArcStr, CssProperty, CssPseudo, CssValue, StyleRule, Stylesheet};
use lui_html_parser::HtmlNode;
use rustc_hash::{FxHashMap, FxHasher};
use smallvec::SmallVec;

use crate::{
  bloom::{bloom_might_match, AncestorBloom},
  index::{candidate_rules, PreparedStylesheet, RuleCondition},
  inline::node_inline_style,
  matching::{matches_selector, AncestorEntry, MatchContext},
  media::{evaluate_media, evaluate_supports, MediaContext},
  style::ComputedStyle,
  var_resolution::resolve_vars,
  StyledNode,
};

pub type ElementPath = Vec<usize>;

#[derive(Debug, Clone, Default)]
pub struct InteractionState {
  pub hover_path: Option<ElementPath>,
  pub active_path: Option<ElementPath>,
  pub focus_path: Option<ElementPath>,
  pub target_path: Option<ElementPath>,
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
  stats: Cell<CacheStats>,
}

impl CascadeContext {
  pub fn new() -> Self {
    Self {
      arena_a: UnsafeCell::new(Bump::new()),
      arena_b: UnsafeCell::new(Bump::new()),
      use_a: Cell::new(true),
      prepared: Vec::new(),
      stats: Cell::new(CacheStats::default()),
    }
  }

  /// Cache statistics from the last `cascade` or `cascade_dirty` call.
  pub fn cache_stats(&self) -> CacheStats {
    self.stats.get()
  }

  /// Prepare stylesheets for cascading. Call this when stylesheets change,
  /// not every frame. Pass your UA stylesheet as the first element if you want one.
  pub fn set_stylesheets(&mut self, sheets: &[Stylesheet]) {
    self.prepared = sheets.iter().map(|s| PreparedStylesheet::new(s.clone())).collect();
  }

  fn current_arena(&self) -> &Bump {
    // SAFETY: only the current arena is written to; the other is read-only.
    unsafe {
      if self.use_a.get() {
        &*self.arena_a.get()
      } else {
        &*self.arena_b.get()
      }
    }
  }

  fn swap_and_reset(&self) {
    let next = !self.use_a.get();
    self.use_a.set(next);
    // SAFETY: we reset only the arena we're about to write to.
    // No references into it exist — caller must drop previous results
    // from this arena before calling cascade/cascade_dirty again.
    unsafe {
      let arena = if next {
        &mut *self.arena_a.get()
      } else {
        &mut *self.arena_b.get()
      };
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
    let mut path: ElementPath = ElementPath::new();
    let mut cache = DeclCache::new();
    let mut bloom = AncestorBloom::new();

    let result = cascade_node(
      doc,
      doc,
      &sheets,
      None,
      &[],
      &mut path,
      media,
      interaction,
      arena,
      &mut cache,
      &mut bloom,
    );
    self.stats.set(cache.stats);
    result
  }

  /// Incremental cascade — only recomputes dirty subtrees.
  /// Clean subtrees are deep-copied from `prev` (no rule matching).
  ///
  /// `prev` borrows from the previous arena; this writes to the current one.
  pub fn cascade_dirty<'a>(
    &'a self,
    doc: &'a HtmlNode,
    prev: &StyledNode<'a>,
    dirty_paths: &[ElementPath],
    media: &MediaContext,
    interaction: &'a InteractionState,
  ) -> StyledNode<'a> {
    self.swap_and_reset();
    let arena = self.current_arena();

    let sheets: Vec<&PreparedStylesheet> = self.prepared.iter().collect();
    let mut path: ElementPath = ElementPath::new();

    let mut cache = DeclCache::new();
    let mut bloom = AncestorBloom::new();

    let result = cascade_node_incremental(
      doc,
      doc,
      &sheets,
      None,
      &[],
      &mut path,
      media,
      interaction,
      arena,
      prev,
      dirty_paths,
      &mut cache,
      &mut bloom,
    );
    self.stats.set(cache.stats);
    result
  }
}

impl Default for CascadeContext {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
  pub hits: usize,
  pub misses: usize,
}

struct DeclCache<'a> {
  map: FxHashMap<u64, ComputedStyle<'a>>,
  stats: CacheStats,
}

impl<'a> DeclCache<'a> {
  fn new() -> Self {
    Self {
      map: FxHashMap::default(),
      stats: CacheStats::default(),
    }
  }
}

fn element_cache_key(node: &HtmlNode, ctx: &MatchContext<'_>, ancestors: &[AncestorEntry<'_>]) -> u64 {
  let mut h = FxHasher::default();

  // Pre-computed node identity hash (tag + all attrs + styles, XOR for maps)
  node.node_hash.hash(&mut h);

  // Interaction + structural state (packed as bits)
  let flags: u16 = (ctx.is_hover as u16)
    | (ctx.is_active as u16) << 1
    | (ctx.is_focus as u16) << 2
    | (ctx.is_focus_within as u16) << 3
    | (ctx.is_root as u16) << 4
    | (ctx.is_first_child as u16) << 5
    | (ctx.is_last_child as u16) << 6
    | (ctx.is_only_child as u16) << 7
    | (ctx.is_target as u16) << 8;
  flags.hash(&mut h);

  // Ancestor context: pre-computed hashes, no re-hashing attrs
  let depth = ancestors.len().min(3);
  depth.hash(&mut h);
  for entry in ancestors.iter().take(3) {
    entry.node.node_hash.hash(&mut h);
  }

  h.finish()
}

// ---------------------------------------------------------------------------
fn is_path_dirty(path: &[usize], dirty_paths: &[ElementPath]) -> bool {
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
  sheets: &[&'a PreparedStylesheet],
  parent_style: Option<&ComputedStyle<'a>>,
  ancestors: &[AncestorEntry<'a>],
  path: &mut ElementPath,
  media: &MediaContext,
  interaction: &'a InteractionState,
  arena: &'a Bump,
  prev: &StyledNode<'a>,
  dirty_paths: &[ElementPath],
  cache: &mut DeclCache<'a>,
  bloom: &mut AncestorBloom,
) -> StyledNode<'a> {
  if !is_path_dirty(path, dirty_paths) {
    return clone_subtree(node, prev, parent_style);
  }

  cascade_node_with_prev(
    root,
    node,
    sheets,
    parent_style,
    ancestors,
    path,
    media,
    interaction,
    arena,
    Some(prev),
    dirty_paths,
    cache,
    bloom,
  )
}

fn cascade_node_with_prev<'a>(
  root: &'a HtmlNode,
  node: &'a HtmlNode,
  sheets: &[&'a PreparedStylesheet],
  parent_style: Option<&ComputedStyle<'a>>,
  ancestors: &[AncestorEntry<'a>],
  path: &mut ElementPath,
  media: &MediaContext,
  interaction: &'a InteractionState,
  arena: &'a Bump,
  prev: Option<&StyledNode<'a>>,
  dirty_paths: &[ElementPath],
  cache: &mut DeclCache<'a>,
  bloom: &mut AncestorBloom,
) -> StyledNode<'a> {
  let ctx = build_match_context(node, path, interaction, ancestors);

  let mut style = get_cached_or_compute(node, &ctx, sheets, ancestors, media, arena, cache, bloom);

  if let Some(parent) = parent_style {
    style.inherit_from(parent);
  }

  resolve_vars(&mut style, arena);

  let mut child_ancestors: SmallVec<[AncestorEntry<'a>; 16]> = SmallVec::with_capacity(ancestors.len() + 1);
  child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
  child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
    node: a.node,
    ctx: a.ctx.clone(),
  }));

  let tag = node.element.tag_name();
  let id = node.id.as_deref();
  
  bloom.push(tag, id, &node.class_list);

  let children: Vec<StyledNode<'a>> = node
    .children
    .iter()
    .enumerate()
    .map(|(i, child)| {
      path.push(i);
      let prev_child = prev.and_then(|p| p.children.get(i));
      let child_node = if let Some(pc) = prev_child {
        if !is_path_dirty(path, dirty_paths) {
          clone_subtree(child, pc, Some(&style))
        } else {
          cascade_node_with_prev(
            root, child, sheets, Some(&style), &child_ancestors, path,
            media, interaction, arena, Some(pc), dirty_paths, cache, bloom,
          )
        }
      } else {
        cascade_node(
          root, child, sheets, Some(&style), &child_ancestors, path,
          media, interaction, arena, cache, bloom,
        )
      };
      path.pop();
      child_node
    })
    .collect();

  bloom.pop(tag, id, &node.class_list);

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
  prev: &StyledNode<'a>,
  parent_style: Option<&ComputedStyle<'a>>,
) -> StyledNode<'a> {
  // prev's references point into the previous arena, which is still alive
  // for 'a (double-buffered). Just copy the Option<&'a CssValue> pointers.
  let mut style = prev.style.clone();

  if let Some(parent) = parent_style {
    style.inherit_from(parent);
  }

  let children: Vec<StyledNode<'a>> = node
    .children
    .iter()
    .enumerate()
    .map(|(i, child)| {
      if let Some(prev_child) = prev.children.get(i) {
        clone_subtree(child, prev_child, Some(&style))
      } else {
        StyledNode {
          node: child,
          style: ComputedStyle::default(),
          children: Vec::new(),
          before: None,
          after: None,
          first_line: None,
          first_letter: None,
          placeholder: None,
          selection: None,
          marker: None,
        }
      }
    })
    .collect();

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
  sheets: &[&'a PreparedStylesheet],
  parent_style: Option<&ComputedStyle<'a>>,
  ancestors: &[AncestorEntry<'a>],
  path: &mut Vec<usize>,
  media: &MediaContext,
  interaction: &'a InteractionState,
  arena: &'a Bump,
  cache: &mut DeclCache<'a>,
  bloom: &mut AncestorBloom,
) -> StyledNode<'a> {
  let ctx = build_match_context(node, path, interaction, ancestors);

  let mut style = get_cached_or_compute(node, &ctx, sheets, ancestors, media, arena, cache, bloom);

  if let Some(parent) = parent_style {
    style.inherit_from(parent);
  }

  resolve_vars(&mut style, arena);

  let mut child_ancestors: SmallVec<[AncestorEntry<'a>; 16]> = SmallVec::with_capacity(ancestors.len() + 1);
  child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
  child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
    node: a.node,
    ctx: a.ctx.clone(),
  }));

  // Push this node into the bloom filter before cascading children
  let tag = node.element.tag_name();
  let id = node.id.as_deref();

  bloom.push(tag, id, &node.class_list);

  let children: Vec<StyledNode<'a>> = node
    .children
    .iter()
    .enumerate()
    .map(|(i, child)| {
      path.push(i);
      let child_node = cascade_node(
        root,
        child,
        sheets,
        Some(&style),
        &child_ancestors,
        path,
        media,
        interaction,
        arena,
        cache,
        bloom,
      );
      path.pop();
      child_node
    })
    .collect();

  bloom.pop(tag, id, &node.class_list);

  let before =
    crate::pseudo::collect_pseudo_element(CssPseudo::Before, node, &style, sheets, ancestors, &ctx, media, arena);
  let after =
    crate::pseudo::collect_pseudo_element(CssPseudo::After, node, &style, sheets, ancestors, &ctx, media, arena);

  StyledNode {
    node,
    style,
    children,
    before,
    after,
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
  sheets: &[&'a PreparedStylesheet],
  ancestors: &[AncestorEntry<'_>],
  media: &MediaContext,
  arena: &'a Bump,
  bloom: &AncestorBloom,
) -> ComputedStyle<'a> {
  let tag = node.element.tag_name();
  let id = node.id.as_deref();
  let classes: SmallVec<[&str; 8]> = node.class_list.iter().map(|c| c.as_ref()).collect();
  let mut matched = collect_matching_rules(node, ctx, sheets, ancestors, media, tag, id, &classes, bloom);

  matched.sort_by_key(|m| (m.sheet_idx, m.specificity, m.rule_idx));

  let inline_decls = node_inline_style(node);

  let mut style = ComputedStyle::default();

  // Layer 1: author normal (ascending specificity)
  for m in &matched {
    for decl in &m.rule.declarations {
      if !decl.important {
        apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
      }
    }
  }

  // Layer 2: inline normal
  for decl in &inline_decls {
    if !decl.important {
      let val = arena.alloc(decl.value.clone());
      apply_declaration_ref(&mut style, &decl.property, val, arena);
    }
  }

  // Layer 3: author !important (ascending specificity)
  for m in &matched {
    for decl in &m.rule.declarations {
      if decl.important {
        apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
      }
    }
  }

  // Layer 4: inline !important
  for decl in &inline_decls {
    if decl.important {
      let val = arena.alloc(decl.value.clone());
      apply_declaration_ref(&mut style, &decl.property, val, arena);
    }
  }

  // Custom properties from declarations — stylesheet values are already 'a
  for m in &matched {
    for decl in &m.rule.declarations {
      if let CssProperty::Unknown(ref name) = decl.property {
        if name.starts_with("--") {
          style
            .custom_properties
            .get_or_insert_with(Default::default)
            .insert(ArcStr::from(name.as_str()), &decl.value);
        }
      }
    }
  }
  for decl in &inline_decls {
    if let CssProperty::Unknown(ref name) = decl.property {
      if name.starts_with("--") {
        let val = arena.alloc(decl.value.clone());
        style
          .custom_properties
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
  bloom: &AncestorBloom,
) -> SmallVec<[MatchedRule<'a>; 8]> {
  let mut matched = smallvec::smallvec![];

  for (sheet_idx, sheet) in sheets.iter().enumerate() {
    let candidates = candidate_rules(
      &sheet.index,
      tag,
      id,
      classes,
      &node.attrs,
      &node.data_attrs,
      &node.aria_attrs,
    );
    for rule_ref in candidates {
      let rule = &sheet.rules[rule_ref.rule_idx];
      if let Some(specificity) = matched_specificity_bloom(rule, node, ctx, ancestors, bloom) {
        matched.push(MatchedRule {
          rule,
          specificity,
          sheet_idx,
          rule_idx: rule_ref.rule_idx,
        });
      }
    }

    let cond_candidates = candidate_rules(
      &sheet.conditional_index,
      tag,
      id,
      classes,
      &node.attrs,
      &node.data_attrs,
      &node.aria_attrs,
    );
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
      if let Some(specificity) = matched_specificity_bloom(&cond_rule.rule, node, ctx, ancestors, bloom) {
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

fn matched_specificity_bloom(
  rule: &StyleRule,
  node: &HtmlNode,
  ctx: &MatchContext<'_>,
  ancestors: &[AncestorEntry<'_>],
  bloom: &AncestorBloom,
) -> Option<(u32, u32, u32)> {
  let parent = ancestors.first().map(|a| a.node);
  for sel in &rule.selector.0 {
    if sel.compounds.len() > 1 && !bloom_might_match(sel, bloom) {
      continue;
    }
    if matches_selector(sel, node, ctx, ancestors, parent) {
      return Some(sel.specificity());
    }
  }
  None
}

pub fn apply_declaration_ref<'a>(
  style: &mut ComputedStyle<'a>,
  prop: &CssProperty,
  value: &'a CssValue,
  arena: &'a Bump,
) {
  let longhands = longhands_of(prop.clone());
  if longhands.is_empty() {
    style.set(prop, value);
  } else {
    let expanded = expand_shorthand(prop.clone(), &[value.clone()]);
    for (lh_prop, lh_value) in expanded {
      let lh_ref = arena.alloc(lh_value);
      apply_declaration_ref(style, &lh_prop, lh_ref, arena);
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
  let sibling_count = ancestors.first().map(|a| a.node.children.len()).unwrap_or(1);

  let is_hover = interaction
    .hover_path
    .as_ref()
    .map(|hp| path.starts_with(hp) || hp.starts_with(path))
    .unwrap_or(false);
  let is_active = interaction
    .active_path
    .as_ref()
    .map(|ap| path == ap.as_slice())
    .unwrap_or(false);
  let is_focus = interaction
    .focus_path
    .as_ref()
    .map(|fp| path == fp.as_slice())
    .unwrap_or(false);
  let is_focus_within = interaction
    .focus_path
    .as_ref()
    .map(|fp| fp.starts_with(path))
    .unwrap_or(false);

  let lang = node
    .attrs
    .get("lang")
    .map(|s| s.as_ref())
    .or_else(|| ancestors.iter().find_map(|a| a.ctx.lang));

  let dir = node
    .attrs
    .get("dir")
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
    is_target: interaction.target_path.as_ref().is_some_and(|tp| path == tp.as_slice()),
    lang,
    dir,
    is_fullscreen: false,
    is_modal: false,
  }
}

fn get_cached_or_compute<'a>(
  node: &'a HtmlNode,
  ctx: &MatchContext<'_>,
  sheets: &[&'a PreparedStylesheet],
  ancestors: &[AncestorEntry<'a>],
  media: &MediaContext,
  arena: &'a Bump,
  cache: &mut DeclCache<'a>,
  bloom: &AncestorBloom,
) -> ComputedStyle<'a> {
  if node.element.is_text() || node.element.is_comment() {
    return ComputedStyle::default();
  }

  let key = element_cache_key(node, ctx, ancestors);
  if let Some(cached) = cache.map.get(&key) {
    cache.stats.hits += 1;
    return cached.clone();
  }

  cache.stats.misses += 1;
  let style = compute_style(node, ctx, sheets, ancestors, media, arena, bloom);
  cache.map.insert(key, style.clone());
  style
}
