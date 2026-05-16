use std::{
  cell::{Cell, UnsafeCell},
  hash::{Hash, Hasher},
};

use bumpalo::Bump;
use lui_core::{ArcStr, CssProperty, CssPseudo, CssValue, StyleRule, Stylesheet};
use lui_parse::{expand_shorthand, longhands_of, HtmlNode};
use lui_resolve::ResolutionContext;
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHasher};
use smallvec::SmallVec;

use crate::{
  bloom::{bloom_might_match, AncestorBloom},
  index::{candidate_rules, PreparedStylesheet, RuleCondition},
  inline::node_inline_style,
  matching::{is_pseudo_element, matches_selector, AncestorEntry, MatchContext},
  media::{evaluate_media, evaluate_supports, MediaContext},
  style::ComputedStyle,
  var_resolution::resolve_vars,
  StyledNode,
};

pub type ElementPath = lui_core::node::ElementPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarPart {
  Scrollbar,
  Thumb,
  Track,
  Corner,
}

#[derive(Debug, Clone, Default)]
pub struct InteractionState {
  pub hover_path: Option<ElementPath>,
  pub active_path: Option<ElementPath>,
  pub focus_path: Option<ElementPath>,
  pub target_path: Option<ElementPath>,
  pub scrollbar_hover: Option<(ElementPath, ScrollbarPart)>,
  pub scrollbar_active: Option<(ElementPath, ScrollbarPart)>,
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
  _res: ResolutionContext,
}

impl CascadeContext {
  pub fn new() -> Self {
    Self {
      arena_a: UnsafeCell::new(Bump::new()),
      arena_b: UnsafeCell::new(Bump::new()),
      use_a: Cell::new(true),
      prepared: Vec::new(),
      stats: Cell::new(CacheStats::default()),
      _res: ResolutionContext::new(lui_resolve::ResolverContext::default()),
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

    let root_res = ResolutionContext::new(lui_resolve::ResolverContext::from_cascade(
      media.viewport_width,
      media.viewport_height,
      16.0,
      16.0,
    ));

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
      &root_res,
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

    let root_res = ResolutionContext::new(lui_resolve::ResolverContext::from_cascade(
      media.viewport_width,
      media.viewport_height,
      16.0,
      16.0,
    ));

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
      &root_res,
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
  node.node_hash().hash(&mut h);

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
    entry.node.node_hash().hash(&mut h);
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
  res: &ResolutionContext,
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
    res,
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
  res: &ResolutionContext,
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
  resolve_math_style(&mut style, res, arena);

  let self_font_size = extract_font_size_px(&style, res.env.parent_font_size);
  let child_res = ResolutionContext::new(lui_resolve::ResolverContext::from_cascade(
    res.env.viewport_width,
    res.env.viewport_height,
    res.env.root_font_size,
    self_font_size,
  ));

  let mut child_ancestors: SmallVec<[AncestorEntry<'a>; 16]> = SmallVec::with_capacity(ancestors.len() + 1);
  child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
  child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
    node: a.node,
    ctx: a.ctx.clone(),
  }));

  let tag = node.element().tag_name();
  let id = node.id();

  bloom.push(tag, id, node.class_list().as_slice());

  let children: Vec<StyledNode<'a>> = node
    .children()
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
            root,
            child,
            sheets,
            Some(&style),
            &child_ancestors,
            path,
            media,
            interaction,
            arena,
            &child_res,
            Some(pc),
            dirty_paths,
            cache,
            bloom,
          )
        }
      } else {
        cascade_node(
          root,
          child,
          sheets,
          Some(&style),
          &child_ancestors,
          path,
          media,
          interaction,
          arena,
          &child_res,
          cache,
          bloom,
        )
      };
      path.pop();
      child_node
    })
    .collect();

  bloom.pop(tag, id, node.class_list().as_slice());

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
    scrollbar_pseudo: None,
    _arenas: Vec::new(),
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
    .children()
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
          scrollbar_pseudo: None,
          _arenas: Vec::new(),
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
    scrollbar_pseudo: None,
    _arenas: Vec::new(),
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
  res: &ResolutionContext,
  cache: &mut DeclCache<'a>,
  bloom: &mut AncestorBloom,
) -> StyledNode<'a> {
  let ctx = build_match_context(node, path, interaction, ancestors);

  let mut style = get_cached_or_compute(node, &ctx, sheets, ancestors, media, arena, cache, bloom);

  if let Some(parent) = parent_style {
    style.inherit_from(parent);
  }

  resolve_vars(&mut style, arena);
  resolve_math_style(&mut style, res, arena);

  let self_font_size = extract_font_size_px(&style, res.env.parent_font_size);
  let child_res = ResolutionContext::new(lui_resolve::ResolverContext::from_cascade(
    res.env.viewport_width,
    res.env.viewport_height,
    res.env.root_font_size,
    self_font_size,
  ));

  let mut child_ancestors: SmallVec<[AncestorEntry<'a>; 16]> = SmallVec::with_capacity(ancestors.len() + 1);
  child_ancestors.push(AncestorEntry { node, ctx: ctx.clone() });
  child_ancestors.extend(ancestors.iter().map(|a| AncestorEntry {
    node: a.node,
    ctx: a.ctx.clone(),
  }));

  // Push this node into the bloom filter before cascading children
  let tag = node.element().tag_name();
  let id = node.id();

  bloom.push(tag, id, node.class_list().as_slice());

  const PAR_CHILDREN: usize = 16;

  let (children, par_arenas) = if node.children().len() >= PAR_CHILDREN {
    cascade_children_parallel(
      root,
      node,
      sheets,
      Some(&style),
      &child_ancestors,
      path,
      media,
      interaction,
      bloom,
    )
  } else {
    (
      node
        .children()
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
            &child_res,
            cache,
            bloom,
          );
          path.pop();
          child_node
        })
        .collect(),
      Vec::new(),
    )
  };

  bloom.pop(tag, id, node.class_list().as_slice());

  let before =
    crate::pseudo::collect_pseudo_element(CssPseudo::Before, node, &style, sheets, ancestors, &ctx, media, arena);
  let after =
    crate::pseudo::collect_pseudo_element(CssPseudo::After, node, &style, sheets, ancestors, &ctx, media, arena);
  let first_line = crate::pseudo::collect_pseudo_style(
    CssPseudo::FirstLine,
    node,
    &style,
    sheets,
    ancestors,
    &ctx,
    media,
    arena,
  );
  let first_letter = crate::pseudo::collect_pseudo_style(
    CssPseudo::FirstLetter,
    node,
    &style,
    sheets,
    ancestors,
    &ctx,
    media,
    arena,
  );
  let placeholder = crate::pseudo::collect_pseudo_style(
    CssPseudo::Placeholder,
    node,
    &style,
    sheets,
    ancestors,
    &ctx,
    media,
    arena,
  );
  let selection = crate::pseudo::collect_pseudo_style(
    CssPseudo::Selection,
    node,
    &style,
    sheets,
    ancestors,
    &ctx,
    media,
    arena,
  );
  let marker =
    crate::pseudo::collect_pseudo_element(CssPseudo::Marker, node, &style, sheets, ancestors, &ctx, media, arena);

  let scrollbar_hover_part = interaction
    .scrollbar_hover
    .as_ref()
    .and_then(|(hp, part)| if hp == path { Some(*part) } else { None });
  let scrollbar_active_part = interaction
    .scrollbar_active
    .as_ref()
    .and_then(|(ap, part)| if ap == path { Some(*part) } else { None });
  let scrollbar_pseudo = crate::pseudo::collect_scrollbar_pseudo_styles(
    node,
    &style,
    sheets,
    ancestors,
    &ctx,
    media,
    arena,
    scrollbar_hover_part,
    scrollbar_active_part,
  );

  StyledNode {
    node,
    style,
    children,
    before,
    after,
    first_line,
    first_letter,
    placeholder,
    selection,
    marker,
    scrollbar_pseudo,
    _arenas: par_arenas,
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
  let tag = node.element().tag_name();
  let id = node.id();
  let classes: SmallVec<[&str; 8]> = node.class_list().iter().map(|c| c.as_ref()).collect();
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
      node.attrs(),
      node.data_attrs(),
      node.aria_attrs(),
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
      node.attrs(),
      node.data_attrs(),
      node.aria_attrs(),
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
    if sel
      .compounds
      .last()
      .is_some_and(|compound| compound.pseudos.iter().any(|pseudo| is_pseudo_element(&pseudo.pseudo)))
    {
      continue;
    }
    if sel.compounds.len() > 1 && !bloom_might_match(sel, bloom) {
      continue;
    }
    if matches_selector(sel, node, ctx, ancestors, parent) {
      return Some(lui_parse::complex_specificity(sel));
    }
  }
  None
}

/// Parallel cascade of children when sibling count exceeds threshold.
/// Each thread gets its own Bump arena; results are clone_into'd the
/// main arena after collection.
fn cascade_children_parallel<'a>(
  root: &'a HtmlNode,
  node: &'a HtmlNode,
  sheets: &[&'a PreparedStylesheet],
  style: Option<&ComputedStyle<'a>>,
  ancestors: &[AncestorEntry<'a>],
  path: &mut Vec<usize>,
  media: &MediaContext,
  interaction: &'a InteractionState,
  bloom: &AncestorBloom,
) -> (Vec<StyledNode<'a>>, Vec<Bump>) {
  struct ParResult {
    _arena: Box<Bump>,
    node_ptr: *const StyledNode<'static>,
  }
  // SAFETY: ParResult is only accessed from the current thread
  // after the parallel map completes.
  unsafe impl Send for ParResult {}

  let mut results: Vec<(usize, ParResult)> = node
    .children()
    .par_iter()
    .enumerate()
    .map(|(i, child)| {
      let mut child_path = path.clone();
      child_path.push(i);
      let arena = Box::new(Bump::new());
      let mut cache = DeclCache::new();
      let mut child_bloom = *bloom;
      let parent_fs = style.map(|s| extract_font_size_px(s, 16.0)).unwrap_or(16.0);
      let res = ResolutionContext::new(lui_resolve::ResolverContext::from_cascade(
        media.viewport_width,
        media.viewport_height,
        16.0,
        parent_fs,
      ));

      let child_node = cascade_node(
        root,
        child,
        sheets,
        style,
        ancestors,
        &mut child_path,
        media,
        interaction,
        &arena,
        &res,
        &mut cache,
        &mut child_bloom,
      );

      // Allocate the result inside the arena so it lives as long as arena.
      let node_ref = arena.alloc(child_node);
      let ptr = node_ref as *const StyledNode;
      // node_ref overwritten below, drop is implicit
      let node_ptr = unsafe { std::mem::transmute::<*const StyledNode<'_>, *const StyledNode<'static>>(ptr) };
      (
        i,
        ParResult {
          _arena: arena,
          node_ptr,
        },
      )
    })
    .collect();

  results.sort_by_key(|(i, _)| *i);

  let mut child_arenas = Vec::with_capacity(results.len());
  let nodes: Vec<StyledNode<'a>> = results
    .into_iter()
    .map(|(_, r)| {
      // SAFETY: The Bump arena (in Box) stays at a fixed heap address.
      // We copy the StyledNode out with transmute_copy — the original
      // in the arena is never dropped (Bump::reset skips Drop).
      // CssValue references point into the arena, kept alive via child_arenas.
      let node: StyledNode = unsafe { std::mem::transmute_copy(&*r.node_ptr) };
      child_arenas.push(*r._arena);
      unsafe { std::mem::transmute::<StyledNode<'_>, StyledNode<'a>>(node) }
    })
    .collect();

  (nodes, child_arenas)
}

/// Resolve math functions and var() in all style properties.
fn resolve_math_style<'a>(style: &mut ComputedStyle<'a>, res: &ResolutionContext, arena: &'a Bump) {
  use lui_core::CssValue;
  fn needs_resolve(v: &CssValue) -> bool {
    matches!(v, CssValue::Function { .. } | CssValue::Var { .. })
      || matches!(v, CssValue::Dimension { unit, .. } if !matches!(unit, lui_core::CssUnit::Px))
  }

  // Resolve font-size first (em in font-size resolves against parent's font-size)
  if let Some(val) = &style.font_size {
    if needs_resolve(val) {
      let new_val = res.resolve_value(val, arena);
      if !std::ptr::eq(*val, new_val) {
        style.font_size = Some(new_val);
      }
    }
  }

  // Extract the element's own resolved font-size for em resolution of other properties
  let self_font_size = match style.font_size {
    Some(CssValue::Dimension {
      value,
      unit: lui_core::CssUnit::Px,
    }) => *value as f32,
    Some(CssValue::Number(n)) => *n as f32,
    _ => res.env.parent_font_size,
  };

  let self_res = if (self_font_size - res.env.parent_font_size).abs() > 0.001 {
    let env = lui_resolve::ResolverContext::from_cascade(
      res.env.viewport_width,
      res.env.viewport_height,
      res.env.root_font_size,
      self_font_size,
    );
    Some(ResolutionContext::new(env))
  } else {
    None
  };
  let res = self_res.as_ref().unwrap_or(res);

  macro_rules! r {
        ($($field:ident),* $(,)?) => {
            $( if let Some(val) = &style.$field {
                if needs_resolve(val) {
                    let new_val = res.resolve_value(val, arena);
                    if !std::ptr::eq(*val, new_val) {
                        style.$field = Some(new_val);
                    }
                }
            } )*
        };
    }
  r!(
    display,
    position,
    top,
    right,
    bottom,
    left,
    float,
    clear,
    width,
    height,
    min_width,
    min_height,
    max_width,
    max_height,
    box_sizing,
    aspect_ratio,
    margin_top,
    margin_right,
    margin_bottom,
    margin_left,
    padding_top,
    padding_right,
    padding_bottom,
    padding_left,
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
    background_color,
    background_image,
    background_size,
    background_position,
    background_repeat,
    background_clip,
    color,
    opacity,
    visibility,
    font_family,
    font_size,
    font_weight,
    font_style,
    line_height,
    letter_spacing,
    word_spacing,
    text_align,
    text_decoration_line,
    text_decoration_color,
    text_decoration_style,
    text_transform,
    white_space,
    word_break,
    text_overflow,
    vertical_align,
    flex_direction,
    flex_wrap,
    justify_content,
    align_items,
    align_content,
    align_self,
    flex_grow,
    flex_shrink,
    flex_basis,
    order,
    row_gap,
    column_gap,
    grid_template_columns,
    grid_template_rows,
    grid_auto_columns,
    grid_auto_rows,
    grid_auto_flow,
    grid_column_start,
    grid_column_end,
    grid_row_start,
    grid_row_end,
    justify_items,
    justify_self,
    overflow_x,
    overflow_y,
    scrollbar_color,
    scrollbar_gutter,
    scrollbar_width,
    transform,
    transform_origin,
    box_shadow,
    z_index,
    cursor,
    pointer_events,
    user_select,
    resize,
    accent_color,
    list_style_type,
    list_style_position,
    list_style_image,
    content,
    fill,
    fill_opacity,
    fill_rule,
    stroke,
    stroke_width,
    stroke_opacity,
    stroke_linecap,
    stroke_linejoin,
    stroke_dasharray,
    stroke_dashoffset,
  );
  if let Some(ref mut extra) = style.extra {
    let mut to_update = Vec::new();
    for (prop, val) in extra.iter() {
      if needs_resolve(val) {
        let new_val = res.resolve_value(val, arena);
        if !std::ptr::eq(*val, new_val) {
          to_update.push((prop.clone(), new_val));
        }
      }
    }
    for (prop, new_val) in to_update {
      extra.insert(prop, new_val);
    }
  }
}

fn extract_font_size_px(style: &ComputedStyle, fallback: f32) -> f32 {
  match style.font_size {
    Some(lui_core::CssValue::Dimension {
      value,
      unit: lui_core::CssUnit::Px,
    }) => *value as f32,
    Some(lui_core::CssValue::Number(n)) => *n as f32,
    _ => fallback,
  }
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
    let skip_value_parsing = matches!(
      prop,
      CssProperty::GridArea
        | CssProperty::GridColumn
        | CssProperty::GridRow
        | CssProperty::GridTemplate
    );
    let values = match value {
      CssValue::String(s) | CssValue::Unknown(s) if !skip_value_parsing => {
        lui_parse::parse_values(s.as_ref()).unwrap_or_else(|_| vec![value.clone()])
      }
      _ => vec![value.clone()],
    };
    let expanded = expand_shorthand(prop.clone(), &values);
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
  let sibling_count = ancestors.first().map(|a| a.node.children().len()).unwrap_or(1);

  let is_hover = interaction
    .hover_path
    .as_ref()
    .map(|hp| hp.starts_with(path))
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
    .attrs()
    .get("lang")
    .map(|s| s.as_ref())
    .or_else(|| ancestors.iter().find_map(|a| a.ctx.lang));

  let dir = node
    .attrs()
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
  if node.element().is_text() || node.element().is_comment() {
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
