//! Selector matching and CSS cascade.
//!
//! Inputs:
//! - a parsed `Tree` of typed elements (with their inline `style` attributes and `id` / `class` attributes already
//!   populated by the HTML parser)
//! - the contents of any `<style>` blocks found in the tree
//!
//! Output: a parallel `CascadedTree` where every element has a fully
//! computed `Style`. Layout consumes this and never re-parses CSS.
//!
//! Cascade order, lowest specificity first, last writer wins on ties:
//! 1. Stylesheet rules (sorted by specificity ascending, then source order)
//! 2. The element's inline `style="…"` attribute (treated as specificity higher than any selector, per CSS)

use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, Mutex, OnceLock},
};

use wgpu_html_models as m;
use wgpu_html_models::Style;
use wgpu_html_parser::{
  AttrOp, ComplexSelector, CompoundSelector, CssWideKeyword, MatchContext as QueryMatchContext, MediaFeature,
  MediaQuery, MediaQueryList, MediaType, PseudoClass, Rule, Stylesheet, parse_inline_style_decls, parse_stylesheet,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MediaContext {
  /// Viewport width in CSS pixels.
  pub viewport_width: f32,
  /// Viewport height in CSS pixels.
  pub viewport_height: f32,
  /// CSS-px to physical-px scale. Used for resolution queries.
  pub scale: f32,
  pub media_type: MediaType,
}

impl MediaContext {
  pub fn screen(viewport_width: f32, viewport_height: f32, scale: f32) -> Self {
    Self {
      viewport_width,
      viewport_height,
      scale,
      media_type: MediaType::Screen,
    }
  }
}

impl Default for MediaContext {
  fn default() -> Self {
    Self::screen(f32::INFINITY, f32::INFINITY, 1.0)
  }
}

impl MatchContext {
  /// Compute the context for the element at `path` given the
  /// document's `InteractionState`. An element is "in the hover
  /// chain" iff its path is a prefix of `state.hover_path` (i.e. it
  /// is, or is an ancestor of, the deepest hovered element).
  pub fn for_path(path: &[usize], state: &InteractionState) -> Self {
    Self::for_path_with_siblings(path, state, None)
  }

  pub fn for_path_with_siblings(path: &[usize], state: &InteractionState, sibling_count: Option<usize>) -> Self {
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

impl CascadedNode {
  /// Walk a child-index path from this node to a descendant.
  /// Returns `None` if any index is out of bounds.
  pub fn at_path(&self, path: &[usize]) -> Option<&CascadedNode> {
    let mut cursor: &CascadedNode = self;
    for &i in path {
      cursor = cursor.children.get(i)?;
    }
    Some(cursor)
  }
}

/// Tracks which pseudo-classes appear in which position across all
/// selectors. Used by incremental cascade to decide whether a
/// pseudo-class state change requires re-cascade at all.
#[derive(Debug, Clone, Default)]
struct PseudoClassUsage {
  /// Any selector uses `:hover` on the subject compound.
  has_hover_subject: bool,
  /// Any selector uses `:hover` on an ancestor compound (descendant
  /// combinator, e.g. `.nav:hover .link { … }`).
  has_hover_ancestor: bool,
  /// Same for `:active`.
  has_active_subject: bool,
  has_active_ancestor: bool,
  /// Same for `:focus`.
  has_focus_subject: bool,
  has_focus_ancestor: bool,
  /// True when ALL pseudo-class rules in this stylesheet only declare
  /// paint-affecting properties (background-color, color, opacity, etc.)
  /// and never layout-affecting ones (width, padding, display, etc.).
  /// When true, the pipeline can skip re-layout on pseudo-class changes.
  all_pseudo_rules_paint_only: bool,
}

impl PseudoClassUsage {
  fn has_hover(&self) -> bool {
    self.has_hover_subject || self.has_hover_ancestor
  }
  fn has_active(&self) -> bool {
    self.has_active_subject || self.has_active_ancestor
  }
  fn has_focus(&self) -> bool {
    self.has_focus_subject || self.has_focus_ancestor
  }
  fn has_any_ancestor(&self) -> bool {
    self.has_hover_ancestor || self.has_active_ancestor || self.has_focus_ancestor
  }
  fn has_any(&self) -> bool {
    self.has_hover() || self.has_active() || self.has_focus()
  }
}

#[derive(Debug, Clone)]
struct PreparedStylesheet {
  sheet: Arc<Stylesheet>,
  index: RuleIndex,
  normal_nonempty: Vec<bool>,
  important_nonempty: Vec<bool>,
  relevant: RelevantSelectors,
  pseudo_usage: PseudoClassUsage,
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
    let mut pseudo_usage = PseudoClassUsage::default();
    let mut has_any_pseudo_rule = false;
    let mut all_pseudo_paint_only = true;
    for (rule_idx, rule) in sheet.rules.iter().enumerate() {
      normal_nonempty.push(!rule.keywords.is_empty() || style_has_values(&rule.declarations));
      important_nonempty.push(!rule.important_keywords.is_empty() || style_has_values(&rule.important));
      for (selector_idx, selector) in rule.selectors.iter().enumerate() {
        collect_relevant_selector_bits(selector, &mut relevant);
        let subj = selector.subject();
        let has_pseudo = !subj.pseudo_classes.is_empty()
          || selector
            .ancestor_compounds()
            .iter()
            .any(|a| !a.pseudo_classes.is_empty());
        if has_pseudo {
          has_any_pseudo_rule = true;
          if style_has_layout_properties(&rule.declarations) || style_has_layout_properties(&rule.important) {
            all_pseudo_paint_only = false;
          }
        }
        // Scan subject compound for pseudo-class usage.
        for pc in &subj.pseudo_classes {
          match pc {
            PseudoClass::Hover => pseudo_usage.has_hover_subject = true,
            PseudoClass::Active => pseudo_usage.has_active_subject = true,
            PseudoClass::Focus => pseudo_usage.has_focus_subject = true,
            _ => {}
          }
        }
        // Scan ancestor compounds for pseudo-class usage.
        for anc in selector.ancestor_compounds() {
          for pc in &anc.pseudo_classes {
            match pc {
              PseudoClass::Hover => pseudo_usage.has_hover_ancestor = true,
              PseudoClass::Active => pseudo_usage.has_active_ancestor = true,
              PseudoClass::Focus => pseudo_usage.has_focus_ancestor = true,
              _ => {}
            }
          }
        }
        let entry = SelectorRuleRef { rule_idx, selector_idx };
        if let Some(id) = &subj.id {
          index.by_id.entry(id.clone()).or_default().push(entry);
        } else if let Some(class) = subj.classes.first() {
          index.by_class.entry(class.clone()).or_default().push(entry);
        } else if let Some(tag) = &subj.tag {
          index.by_tag.entry(tag.clone()).or_default().push(entry);
        } else {
          index.universal.push(entry);
        }
      }
    }
    pseudo_usage.all_pseudo_rules_paint_only = has_any_pseudo_rule && all_pseudo_paint_only;
    Self {
      sheet,
      index,
      normal_nonempty,
      important_nonempty,
      relevant,
      pseudo_usage,
    }
  }
}

/// Fingerprint-based cache key for computed declarations. Uses
/// hashing instead of owned strings so building the key requires
/// zero heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct DeclCacheKey {
  fingerprint: u64,
}

fn collect_relevant_selector_bits(sel: &ComplexSelector, relevant: &mut RelevantSelectors) {
  let subj = sel.subject();
  if let Some(id) = &subj.id {
    relevant.ids.insert(id.clone());
  }
  if let Some(tag) = &subj.tag {
    relevant.tags.insert(tag.clone());
  }
  relevant.classes.extend(subj.classes.iter().cloned());
  relevant.attrs.extend(subj.attrs.iter().map(|attr| attr.name.clone()));
  for ancestor in sel.ancestor_compounds() {
    if let Some(id) = &ancestor.id {
      relevant.ids.insert(id.clone());
    }
    if let Some(tag) = &ancestor.tag {
      relevant.tags.insert(tag.clone());
    }
    relevant.classes.extend(ancestor.classes.iter().cloned());
    relevant.attrs.extend(ancestor.attrs.iter().map(|a| a.name.clone()));
  }
}

fn decl_cache_key(
  element: &Element,
  element_ctx: MatchContext,
  sheets: &[&PreparedStylesheet],
  ancestors: &[(&Element, MatchContext)],
  cascade_ctx: &CascadeContext,
) -> DeclCacheKey {
  use std::hash::{Hash, Hasher};
  let mut h = std::collections::hash_map::DefaultHasher::new();

  // Hash the element's selector-relevant bits.
  hash_element_signature(element, element_ctx, sheets, cascade_ctx, &mut h);

  // Hash the inline style attribute (highest specificity layer).
  element_style_attr(element).hash(&mut h);

  // Hash ancestor signatures so descendant-combinator rules
  // that differ by ancestor path produce distinct keys.
  ancestors.len().hash(&mut h);
  for (ancestor, ctx) in ancestors {
    hash_element_signature(ancestor, *ctx, sheets, cascade_ctx, &mut h);
  }

  DeclCacheKey {
    fingerprint: h.finish(),
  }
}

/// Hash the selector-relevant bits of an element into `h` without
/// allocating any owned Strings. Uses `&str` references from the
/// Element's existing fields, hashing them directly.
fn hash_element_signature(
  element: &Element,
  ctx: MatchContext,
  sheets: &[&PreparedStylesheet],
  cascade_ctx: &CascadeContext,
  h: &mut impl std::hash::Hasher,
) {
  use std::hash::Hash;

  // Tag
  if let Some(tag) = element_tag(element) {
    if relevant_tag(sheets, tag) {
      tag.hash(h);
    }
  }

  // ID
  if let Some(id) = element_id(element) {
    if relevant_id(sheets, id) {
      id.hash(h);
    }
  }

  // Classes — hash in sorted order for determinism. Collects
  // `&str` references (no owned Strings).
  if let Some(class_attr) = element_class(element) {
    let mut classes: Vec<&str> = class_attr
      .split_ascii_whitespace()
      .filter(|c| relevant_class(sheets, c))
      .collect();
    classes.sort_unstable();
    classes.dedup();
    classes.len().hash(h);
    for c in &classes {
      c.hash(h);
    }
  } else {
    0usize.hash(h);
  }

  // Attributes — uses the pre-computed list from CascadeContext.
  cascade_ctx.attr_names.len().hash(h);
  for name in &cascade_ctx.attr_names {
    name.hash(h);
    element_attr(element, name).hash(h);
  }

  // Pseudo-class state
  ctx.hash(h);
}

fn relevant_id(sheets: &[&PreparedStylesheet], id: &str) -> bool {
  sheets.iter().any(|sheet| sheet.relevant.ids.contains(id))
}

fn relevant_class(sheets: &[&PreparedStylesheet], class: &str) -> bool {
  sheets.iter().any(|sheet| sheet.relevant.classes.contains(class))
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

/// Pre-computed per-cascade-pass data that stays constant across
/// all elements. Avoids recomputing `relevant_attr_names()` and
/// the merged relevant sets on every `hash_element_signature` call.
struct CascadeContext {
  attr_names: Vec<String>,
}

impl CascadeContext {
  fn new(sheets: &[&PreparedStylesheet]) -> Self {
    Self {
      attr_names: relevant_attr_names(sheets),
    }
  }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Cascade a tree end-to-end:
///
/// 1. collect every `<style>` block's text content into one stylesheet,
/// 2. for each element compute its style from the matching rules (consulting `tree.interaction` so dynamic
///    pseudo-classes like `:hover` / `:active` resolve correctly),
/// 3. layer the inline `style="…"` attribute on top, and
/// 4. inherit the standard inheriting properties from the parent's resolved style (CSS-Cascade-3 §3.3 — `color`,
///    font-related properties, line-height, text-align, etc.).
pub fn cascade(tree: &Tree) -> CascadedTree {
  cascade_with_media(tree, &MediaContext::default())
}

pub fn cascade_with_media(tree: &Tree, media: &MediaContext) -> CascadedTree {
  // UA defaults sit before author rules, so on a specificity tie
  // the author rule wins on source order.
  let author = collect_prepared_stylesheet_cached(tree);
  let stylesheets = [ua_prepared_stylesheet(), author.as_ref()];
  let interaction = &tree.interaction;
  let mut path: Vec<usize> = Vec::new();
  let mut decl_cache: HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)> = HashMap::new();
  let cascade_ctx = CascadeContext::new(&stylesheets);
  let Some(root) = tree.root.as_ref() else {
    return CascadedTree { root: None };
  };
  CascadedTree {
    root: Some(cascade_node(
      root,
      root, // node = root for the root element
      &stylesheets,
      None,
      &[],
      &mut path,
      interaction,
      &mut decl_cache,
      None,
      &cascade_ctx,
      media,
    )),
  }
}

/// Incrementally re-cascade only the nodes whose pseudo-class state
/// changed between `old_snapshot` and the current `tree.interaction`.
/// Mutates `cached` in-place. Returns `true` if any node was
/// re-cascaded (meaning layout must re-run).
///
/// When no CSS rule uses the changed pseudo-class (e.g. hover changed
/// Returns `true` when every pseudo-class rule (`:hover`, `:active`,
/// `:focus`) in the tree's stylesheets only declares paint-affecting
/// properties (background-color, color, opacity, etc.) and never
/// layout-affecting ones. When true the pipeline can safely skip
/// re-layout on interaction state changes.
pub fn pseudo_rules_are_paint_only(tree: &Tree) -> bool {
  let author = collect_prepared_stylesheet_cached(tree);
  let sheets: [&PreparedStylesheet; 2] = [ua_prepared_stylesheet(), author.as_ref()];
  sheets
    .iter()
    .all(|s| s.pseudo_usage.all_pseudo_rules_paint_only || !s.pseudo_usage.has_any())
}

/// but no `:hover` rules exist), this short-circuits and returns
/// `false` — the most common case for pages without hover styles.
pub fn cascade_incremental(
  tree: &Tree,
  cached: &mut CascadedTree,
  old_snapshot: &wgpu_html_tree::InteractionSnapshot,
) -> bool {
  cascade_incremental_with_media(tree, cached, old_snapshot, &MediaContext::default())
}

pub fn cascade_incremental_with_media(
  tree: &Tree,
  cached: &mut CascadedTree,
  old_snapshot: &wgpu_html_tree::InteractionSnapshot,
  media: &MediaContext,
) -> bool {
  let new_snapshot = tree.interaction.cascade_snapshot();
  if *old_snapshot == new_snapshot {
    return false;
  }

  let author = collect_prepared_stylesheet_cached(tree);
  let sheets: [&PreparedStylesheet; 2] = [ua_prepared_stylesheet(), author.as_ref()];

  // Check which pseudo-classes changed AND have rules.
  let hover_changed = old_snapshot.hover_path != new_snapshot.hover_path;
  let active_changed = old_snapshot.active_path != new_snapshot.active_path;
  let focus_changed = old_snapshot.focus_path != new_snapshot.focus_path;

  let any_hover = sheets.iter().any(|s| s.pseudo_usage.has_hover());
  let any_active = sheets.iter().any(|s| s.pseudo_usage.has_active());
  let any_focus = sheets.iter().any(|s| s.pseudo_usage.has_focus());

  let needs_cascade = (hover_changed && any_hover) || (active_changed && any_active) || (focus_changed && any_focus);

  if !needs_cascade {
    return false;
  }

  // Are there any ancestor-compound pseudo-class rules? If so, we
  // must dirty subtrees, not just the exact path nodes.
  let any_ancestor_rules = sheets.iter().any(|s| s.pseudo_usage.has_any_ancestor());

  // Collect all paths that need re-cascade.
  let mut dirty: HashSet<Vec<usize>> = HashSet::new();
  let mut dirty_subtrees: HashSet<Vec<usize>> = HashSet::new();

  if hover_changed && any_hover {
    collect_dirty_from_diff(
      &old_snapshot.hover_path,
      &new_snapshot.hover_path,
      &mut dirty,
      if any_ancestor_rules {
        Some(&mut dirty_subtrees)
      } else {
        None
      },
    );
  }
  if active_changed && any_active {
    collect_dirty_from_diff(
      &old_snapshot.active_path,
      &new_snapshot.active_path,
      &mut dirty,
      if any_ancestor_rules {
        Some(&mut dirty_subtrees)
      } else {
        None
      },
    );
  }
  if focus_changed && any_focus {
    collect_dirty_from_diff(
      &old_snapshot.focus_path,
      &new_snapshot.focus_path,
      &mut dirty,
      if any_ancestor_rules {
        Some(&mut dirty_subtrees)
      } else {
        None
      },
    );
  }

  if dirty.is_empty() && dirty_subtrees.is_empty() {
    return false;
  }

  let interaction = &tree.interaction;
  let Some(dom_root) = &tree.root else {
    return false;
  };
  let Some(cascaded_root) = &mut cached.root else {
    return false;
  };

  let mut path: Vec<usize> = Vec::new();
  let mut decl_cache: HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)> = HashMap::new();
  let cascade_ctx = CascadeContext::new(&sheets);
  re_cascade_dirty(
    dom_root,
    dom_root,
    cascaded_root,
    &sheets,
    None,
    &[],
    &mut path,
    interaction,
    &mut decl_cache,
    &dirty,
    &dirty_subtrees,
    None,
    &cascade_ctx,
    media,
  );
  true
}

/// Diff two pseudo-class paths and collect the node paths that
/// changed. Every node from the divergence point to the leaf of
/// both old and new paths is marked dirty.
fn collect_dirty_from_diff(
  old: &Option<Vec<usize>>,
  new: &Option<Vec<usize>>,
  dirty: &mut HashSet<Vec<usize>>,
  mut subtrees: Option<&mut HashSet<Vec<usize>>>,
) {
  // Find the common prefix length.
  let common_len = match (old, new) {
    (Some(o), Some(n)) => o.iter().zip(n.iter()).take_while(|(a, b)| a == b).count(),
    _ => 0,
  };

  // Old path: nodes from common_len onward lose their pseudo-class.
  if let Some(old_path) = old {
    for depth in common_len..=old_path.len() {
      let prefix = old_path[..depth.min(old_path.len())].to_vec();
      dirty.insert(prefix.clone());
      if let Some(ref mut subs) = subtrees.as_deref_mut() {
        subs.insert(prefix);
      }
    }
  }

  // New path: nodes from common_len onward gain their pseudo-class.
  if let Some(new_path) = new {
    for depth in common_len..=new_path.len() {
      let prefix = new_path[..depth.min(new_path.len())].to_vec();
      dirty.insert(prefix.clone());
      if let Some(ref mut subs) = subtrees.as_deref_mut() {
        subs.insert(prefix);
      }
    }
  }
}

/// Walk the cached tree and re-cascade only dirty nodes in-place.
fn re_cascade_dirty(
  root: &Node,
  node: &Node,
  cached: &mut CascadedNode,
  sheets: &[&PreparedStylesheet],
  parent_style: Option<&Style>,
  ancestors: &[(&Element, MatchContext)],
  path: &mut Vec<usize>,
  interaction: &InteractionState,
  decl_cache: &mut HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)>,
  dirty: &HashSet<Vec<usize>>,
  dirty_subtrees: &HashSet<Vec<usize>>,
  sibling_count: Option<usize>,
  cascade_ctx: &CascadeContext,
  media: &MediaContext,
) {
  let is_dirty = dirty.contains(path.as_slice());
  let subtree_dirty = dirty_subtrees
    .iter()
    .any(|s| path.len() >= s.len() && &path[..s.len()] == s.as_slice());

  if is_dirty || subtree_dirty {
    // Re-cascade this node (same logic as cascade_node).
    let element_ctx = MatchContext::for_path_with_siblings(path, interaction, sibling_count);
    let (mut style, keywords) = if matches!(node.element, Element::Text(_)) {
      (Style::default(), HashMap::new())
    } else {
      let key = decl_cache_key(&node.element, element_ctx, sheets, ancestors, cascade_ctx);
      if let Some(hit) = decl_cache.get(&key) {
        hit.clone()
      } else {
        let computed = computed_decls_in_prepared_stylesheets_with_context(
          &node.element,
          &element_ctx,
          sheets,
          ancestors,
          media,
          root,
          path,
          interaction,
        );
        decl_cache.insert(key, computed.clone());
        computed
      }
    };
    for (prop, kw) in &keywords {
      wgpu_html_parser::apply_keyword(&mut style, parent_style, prop, *kw);
    }
    if let Some(parent) = parent_style {
      inherit_into(&mut style, parent, &keywords);
    }
    for (prop, value) in &node.custom_properties {
      style.custom_properties.insert(prop.clone(), value.clone());
    }
    if !style.var_properties.is_empty() || style.custom_properties.values().any(|v| v.contains("var(")) {
      wgpu_html_parser::resolve_var_references(&mut style);
    }
    cached.style = style;
  }

  // Build ancestor chain for children.
  let element_ctx = MatchContext::for_path_with_siblings(path, interaction, sibling_count);
  let mut child_ancestors: Vec<(&Element, MatchContext)> = Vec::with_capacity(ancestors.len() + 1);
  child_ancestors.push((&node.element, element_ctx));
  child_ancestors.extend_from_slice(ancestors);

  // Check if any child needs work before recursing.
  let any_child_dirty = dirty
    .iter()
    .any(|d| d.len() > path.len() && d[..path.len()] == *path.as_slice())
    || dirty_subtrees.iter().any(|s| {
      // A subtree root at or above us means children are dirty.
      path.len() >= s.len() && &path[..s.len()] == s.as_slice()
    });

  if any_child_dirty {
    let child_count = node.children.len();
    for (i, (dom_child, cascaded_child)) in node.children.iter().zip(cached.children.iter_mut()).enumerate() {
      path.push(i);
      re_cascade_dirty(
        root,
        dom_child,
        cascaded_child,
        sheets,
        Some(&cached.style),
        &child_ancestors,
        path,
        interaction,
        decl_cache,
        dirty,
        dirty_subtrees,
        Some(child_count),
        cascade_ctx,
        media,
      );
      path.pop();
    }
  }
}

/// Walk the tree, gather text content of all `<style>` blocks, and parse it.
pub fn collect_stylesheet(tree: &Tree) -> Stylesheet {
  collect_prepared_stylesheet_cached(tree).sheet.as_ref().clone()
}

fn collect_prepared_stylesheet_cached(tree: &Tree) -> Arc<PreparedStylesheet> {
  let css = collect_stylesheet_source(tree);
  if css.is_empty() {
    return Arc::new(PreparedStylesheet::from_sheet(Arc::new(Stylesheet::default())));
  }
  static CACHE: OnceLock<Mutex<HashMap<String, Arc<PreparedStylesheet>>>> = OnceLock::new();
  let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
  if let Ok(mut cache) = cache.lock() {
    if let Some(sheet) = cache.get(&css) {
      return sheet.clone();
    }
    let prepared = Arc::new(PreparedStylesheet::from_sheet(Arc::new(parse_stylesheet(&css))));
    cache.insert(css, prepared.clone());
    return prepared;
  }
  Arc::new(PreparedStylesheet::from_sheet(Arc::new(parse_stylesheet(&css))))
}

fn ua_prepared_stylesheet() -> &'static PreparedStylesheet {
  static UA: OnceLock<PreparedStylesheet> = OnceLock::new();
  UA.get_or_init(|| PreparedStylesheet::from_sheet(Arc::new(ua::ua_stylesheet().clone())))
}

fn collect_stylesheet_source(tree: &Tree) -> String {
  let mut css = String::new();
  // Gather from DOM-referenced stylesheets (<link> and <style> elements).
  let mut referenced: HashSet<&str> = HashSet::new();
  if let Some(root) = &tree.root {
    gather(root, &tree.linked_stylesheets, &mut css, false, &mut referenced);
  }
  // Also include any registered stylesheets that have NO matching
  // <link> element in the DOM (e.g. component styles registered
  // programmatically).
  for (href, sheet_css) in &tree.linked_stylesheets {
    if !referenced.contains(href.as_str()) {
      append_stylesheet_source(&mut css, sheet_css, None);
    }
  }
  css
}

fn gather<'a>(
  node: &'a Node,
  linked_stylesheets: &'a HashMap<String, String>,
  out: &mut String,
  inside_template: bool,
  referenced: &mut HashSet<&'a str>,
) {
  let inside_template = inside_template || matches!(&node.element, Element::Template(_));
  if inside_template {
    return;
  }
  if let Element::StyleElement(style_el) = &node.element {
    let media = style_el.media.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(media) = media {
      out.push_str("@media ");
      out.push_str(media);
      out.push_str(" {\n");
    }
    for child in &node.children {
      if let Element::Text(t) = &child.element {
        out.push_str(t);
      }
    }
    if media.is_some() {
      out.push_str("\n}\n");
    }
    out.push('\n');
  }
  if let Element::Link(link) = &node.element {
    if link_is_stylesheet(link) {
      if let Some(href) = link.href.as_deref() {
        if let Some(css) = linked_stylesheets.get(href) {
          referenced.insert(href);
          append_stylesheet_source(out, css, link.media.as_deref());
        }
      }
    }
  }
  for child in &node.children {
    gather(child, linked_stylesheets, out, inside_template, referenced);
  }
}

fn link_is_stylesheet(link: &wgpu_html_models::Link) -> bool {
  link
    .rel
    .as_deref()
    .map(|rel| {
      rel
        .split_ascii_whitespace()
        .any(|token| token.eq_ignore_ascii_case("stylesheet"))
    })
    .unwrap_or(false)
}

fn append_stylesheet_source(out: &mut String, css: &str, media: Option<&str>) {
  let media = media.map(str::trim).filter(|s| !s.is_empty());
  if let Some(media) = media {
    out.push_str("@media ");
    out.push_str(media);
    out.push_str(" {\n");
  }
  out.push_str(css);
  if media.is_some() {
    out.push_str("\n}\n");
  }
  out.push('\n');
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
  root: &Node,
  node: &Node,
  sheets: &[&PreparedStylesheet],
  parent_style: Option<&Style>,
  ancestors: &[(&Element, MatchContext)],
  path: &mut Vec<usize>,
  interaction: &InteractionState,
  decl_cache: &mut HashMap<DeclCacheKey, (Style, HashMap<String, CssWideKeyword>)>,
  sibling_count: Option<usize>,
  cascade_ctx: &CascadeContext,
  media: &MediaContext,
) -> CascadedNode {
  let element_ctx = MatchContext::for_path_with_siblings(path, interaction, sibling_count);
  let (mut style, keywords) = if matches!(node.element, Element::Text(_)) {
    (Style::default(), HashMap::new())
  } else {
    let key = decl_cache_key(&node.element, element_ctx, sheets, ancestors, cascade_ctx);
    if let Some(cached) = decl_cache.get(&key) {
      cached.clone()
    } else {
      let computed = computed_decls_in_prepared_stylesheets_with_context(
        &node.element,
        &element_ctx,
        sheets,
        ancestors,
        media,
        root,
        path,
        interaction,
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
  let mut child_ancestors: Vec<(&Element, MatchContext)> = Vec::with_capacity(ancestors.len() + 1);
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
        root,
        c,
        sheets,
        Some(&style),
        &child_ancestors,
        path,
        interaction,
        decl_cache,
        Some(child_count),
        cascade_ctx,
        media,
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
    (svg_fill, "fill"),
    (svg_fill_opacity, "fill-opacity"),
    (svg_fill_rule, "fill-rule"),
    (svg_stroke, "stroke"),
    (svg_stroke_width, "stroke-width"),
    (svg_stroke_opacity, "stroke-opacity"),
    (svg_stroke_linecap, "stroke-linecap"),
    (svg_stroke_linejoin, "stroke-linejoin"),
    (svg_stroke_dasharray, "stroke-dasharray"),
    (svg_stroke_dashoffset, "stroke-dashoffset"),
    (pointer_events, "pointer-events"),
    (user_select, "user-select"),
  );
  // Deferred longhands: bulk-clone when child has no overrides and
  // no keyword/reset blocks apply. One HashMap::clone instead of N
  // individual insert calls.
  if !parent.deferred_longhands.is_empty() {
    if child.deferred_longhands.is_empty()
      && keywords.is_empty()
      && child.reset_properties.is_empty()
      && child.keyword_reset_properties.is_empty()
    {
      // Fast path: clone the entire map at once. We still need
      // to filter to inherited properties, but if all of them
      // are inherited (common case), a bulk clone is faster.
      child.deferred_longhands = parent
        .deferred_longhands
        .iter()
        .filter(|(prop, _)| wgpu_html_parser::is_inherited(prop))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    } else {
      for (prop, value) in &parent.deferred_longhands {
        if child.deferred_longhands.contains_key(prop) || keywords.contains_key(prop) {
          continue;
        }
        if child.reset_properties.contains(prop) || child.keyword_reset_properties.contains(prop) {
          continue;
        }
        if wgpu_html_parser::is_inherited(prop) {
          child.deferred_longhands.insert(prop.clone(), value.clone());
        }
      }
    }
  }
  // Custom properties always inherit. Bulk-clone when child is clean.
  if !parent.custom_properties.is_empty() {
    if child.custom_properties.is_empty() && keywords.is_empty() {
      child.custom_properties = parent.custom_properties.clone();
    } else {
      for (prop, value) in &parent.custom_properties {
        if !child.custom_properties.contains_key(prop) && !keywords.contains_key(prop) {
          child.custom_properties.insert(prop.clone(), value.clone());
        }
      }
    }
  }
  // Inherit var_properties for inherited CSS properties.
  if !parent.var_properties.is_empty() {
    if child.var_properties.is_empty()
      && keywords.is_empty()
      && child.reset_properties.is_empty()
      && child.keyword_reset_properties.is_empty()
    {
      child.var_properties = parent
        .var_properties
        .iter()
        .filter(|(prop, _)| wgpu_html_parser::is_inherited(prop))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    } else {
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
pub fn computed_decls(element: &Element, sheet: &Stylesheet) -> (Style, HashMap<String, CssWideKeyword>) {
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
  let with_default: Vec<(&Element, MatchContext)> = ancestors.iter().map(|e| (*e, MatchContext::default())).collect();
  // Dummy root/path for public API without tree access.
  let dummy_root = Node::new(Element::Div(m::Div::default()));
  computed_decls_in_tree_with_context(
    element,
    &MatchContext::default(),
    sheet,
    &with_default,
    &dummy_root,
    &[],
  )
}

/// Stateful variant of [`computed_decls_in_tree`]. Each ancestor is
/// paired with its own `MatchContext` so pseudo-class compounds on
/// ancestors (e.g. `div:hover .child`) resolve correctly.
pub fn computed_decls_in_tree_with_context(
  element: &Element,
  element_ctx: &MatchContext,
  sheet: &Stylesheet,
  ancestors: &[(&Element, MatchContext)],
  root: &Node,
  path: &[usize],
) -> (Style, HashMap<String, CssWideKeyword>) {
  let prepared = PreparedStylesheet::from_sheet(Arc::new(sheet.clone()));
  computed_decls_in_prepared_stylesheets_with_context(
    element,
    element_ctx,
    &[&prepared],
    ancestors,
    &MediaContext::default(),
    root,
    path,
    &InteractionState::default(),
  )
}

fn computed_decls_in_prepared_stylesheets_with_context(
  element: &Element,
  element_ctx: &MatchContext,
  sheets: &[&PreparedStylesheet],
  ancestors: &[(&Element, MatchContext)],
  media: &MediaContext,
  root: &Node,
  path: &[usize],
  interaction: &InteractionState,
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
      matching_rules_for_element(
        sheet,
        element,
        element_ctx,
        ancestors,
        tag,
        id,
        class_attr,
        media,
        root,
        path,
        interaction,
      )
      .into_iter()
      .map(move |(spec, rule_idx, rule, normal_nonempty, important_nonempty)| {
        (spec, sheet_idx, rule_idx, rule, normal_nonempty, important_nonempty)
      })
    })
    .collect();
  matched_rules.sort_by_key(|(spec, sheet_idx, rule_idx, ..)| (*spec, *sheet_idx, *rule_idx));

  // 1. Author normal.
  for (_, _, _, rule, normal_nonempty, _) in &matched_rules {
    if *normal_nonempty {
      apply_layer(&mut values, &mut keywords, &rule.declarations, &rule.keywords);
    }
  }

  // 2. Inline normal.
  if let Some(decls) = &inline {
    apply_layer_if_nonempty(&mut values, &mut keywords, &decls.normal, &decls.keywords_normal);
  }

  // 3. Author !important.
  for (_, _, _, rule, _, important_nonempty) in &matched_rules {
    if *important_nonempty {
      apply_layer(&mut values, &mut keywords, &rule.important, &rule.important_keywords);
    }
  }

  // 4. Inline !important.
  if let Some(decls) = &inline {
    apply_layer_if_nonempty(&mut values, &mut keywords, &decls.important, &decls.keywords_important);
  }

  (values, keywords)
}

fn selector_prefilter_is_complete(sel: &ComplexSelector) -> bool {
  sel.ancestor_compounds().is_empty() && sel.subject().attrs.is_empty() && sel.subject().pseudo_classes.is_empty()
}

fn matching_rules_for_element<'a>(
  sheet: &'a PreparedStylesheet,
  _element: &Element,
  _element_ctx: &MatchContext,
  _ancestors: &[(&Element, MatchContext)],
  tag: Option<&str>,
  id: Option<&str>,
  class_attr: Option<&str>,
  media: &MediaContext,
  root: &Node,
  path: &[usize],
  interaction: &InteractionState,
) -> Vec<(u32, usize, &'a Rule, bool, bool)> {
  let mut selector_entries = Vec::new();
  let mut push_entries = |entries: &[SelectorRuleRef]| {
    for entry in entries {
      if !selector_entries
        .iter()
        .any(|seen: &SelectorRuleRef| seen.rule_idx == entry.rule_idx && seen.selector_idx == entry.selector_idx)
      {
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

  let qctx = QueryMatchContext {
    interaction: Some(interaction),
  };

  let mut rule_specs: Vec<(usize, u32)> = Vec::new();
  for entry in selector_entries {
    let Some(rule) = sheet.sheet.rules.get(entry.rule_idx) else {
      continue;
    };
    if !rule_media_matches(rule, media) {
      continue;
    }
    let Some(selector) = rule.selectors.selectors.get(entry.selector_idx) else {
      continue;
    };
    if !selector_subject_might_match(selector, tag, id, class_attr) {
      continue;
    }
    if !selector_prefilter_is_complete(selector) && !selector.matches_in_tree(root, path, &qctx) {
      continue;
    }
    let spec = selector.specificity();
    if let Some((_, prev)) = rule_specs.iter_mut().find(|(rule_idx, _)| *rule_idx == entry.rule_idx) {
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
      let important_nonempty = sheet.important_nonempty.get(rule_idx).copied().unwrap_or(true);
      Some((spec, rule_idx, rule, normal_nonempty, important_nonempty))
    })
    .collect()
}

fn rule_media_matches(rule: &Rule, media: &MediaContext) -> bool {
  rule
    .media
    .iter()
    .all(|query_list| media_query_list_matches(query_list, media))
}

fn media_query_list_matches(list: &MediaQueryList, ctx: &MediaContext) -> bool {
  list.queries.iter().any(|query| media_query_matches(query, ctx))
}

fn media_query_matches(query: &MediaQuery, ctx: &MediaContext) -> bool {
  let type_matches = matches!(query.media_type, MediaType::All) || query.media_type == ctx.media_type;
  let features_match = query
    .features
    .iter()
    .all(|feature| media_feature_matches(*feature, ctx));
  let matches = type_matches && features_match;
  if query.not { !matches } else { matches }
}

fn media_feature_matches(feature: MediaFeature, ctx: &MediaContext) -> bool {
  match feature {
    MediaFeature::Width(v) => approx_eq(ctx.viewport_width, v),
    MediaFeature::MinWidth(v) => ctx.viewport_width >= v,
    MediaFeature::MaxWidth(v) => ctx.viewport_width <= v,
    MediaFeature::Height(v) => approx_eq(ctx.viewport_height, v),
    MediaFeature::MinHeight(v) => ctx.viewport_height >= v,
    MediaFeature::MaxHeight(v) => ctx.viewport_height <= v,
    MediaFeature::OrientationPortrait => ctx.viewport_height >= ctx.viewport_width,
    MediaFeature::OrientationLandscape => ctx.viewport_width > ctx.viewport_height,
  }
}

fn approx_eq(a: f32, b: f32) -> bool {
  (a - b).abs() <= 0.01
}

fn selector_subject_might_match(
  sel: &ComplexSelector,
  tag: Option<&str>,
  id: Option<&str>,
  class_attr: Option<&str>,
) -> bool {
  let subj = sel.subject();
  if let Some(needed_tag) = &subj.tag
    && tag != Some(needed_tag.as_str())
  {
    return false;
  }
  if let Some(needed_id) = &subj.id
    && id != Some(needed_id.as_str())
  {
    return false;
  }
  if !subj.classes.is_empty() {
    let Some(class_attr) = class_attr else {
      return false;
    };
    for needed in &subj.classes {
      if !class_attr.split_ascii_whitespace().any(|class| class == needed) {
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
    scrollbar_color,
    scrollbar_width,
    opacity,
    visibility,
    z_index,
    svg_fill,
    svg_fill_opacity,
    svg_fill_rule,
    svg_stroke,
    svg_stroke_width,
    svg_stroke_opacity,
    svg_stroke_linecap,
    svg_stroke_linejoin,
    svg_stroke_dasharray,
    svg_stroke_dashoffset,
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

/// Returns true if the style declares any property that can affect layout
/// geometry (sizes, spacing, display mode, font metrics, etc.). Used to
/// detect whether pseudo-class rules are "paint-only" — if all pseudo
/// rules only set paint properties, the pipeline can skip re-layout on
/// hover/active/focus changes.
fn style_has_layout_properties(style: &Style) -> bool {
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
    border_top_width,
    border_right_width,
    border_bottom_width,
    border_left_width,
    border_top_style,
    border_right_style,
    border_bottom_style,
    border_left_style,
    font_family,
    font_size,
    font_weight,
    font_style,
    line_height,
    letter_spacing,
    text_align,
    text_transform,
    white_space,
    overflow,
    overflow_x,
    overflow_y,
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
    box_sizing,
  )
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
pub fn matches_selector(sel: &ComplexSelector, element: &Element) -> bool {
  matches_selector_with_context(sel, element, &MatchContext::default())
}

/// Stateful variant of [`matches_selector`] — checks dynamic
/// pseudo-classes against the supplied `MatchContext`.
pub fn matches_selector_with_context(sel: &ComplexSelector, element: &Element, element_ctx: &MatchContext) -> bool {
  if !sel.ancestor_compounds().is_empty() {
    return false;
  }
  matches_compound(sel.subject(), element) && pseudo_classes_satisfied(sel.subject(), element_ctx)
}

/// Match `sel` against `element` with the element's ancestor chain
/// available. `ancestors[0]` must be the immediate parent, deeper
/// indices going further up to the root. Used by the cascade so
/// descendant-combinator selectors (`.row .item`) actually fire.
///
/// Dynamic pseudo-classes (`:hover`, `:active`) on the subject or
/// any ancestor compound fail without a `MatchContext`; use
/// [`matches_selector_in_tree_with_context`] for stateful matching.
pub fn matches_selector_in_tree(sel: &ComplexSelector, element: &Element, ancestors: &[&Element]) -> bool {
  let with_default: Vec<(&Element, MatchContext)> = ancestors.iter().map(|e| (*e, MatchContext::default())).collect();
  matches_selector_in_tree_with_context(sel, element, &MatchContext::default(), &with_default)
}

/// Stateful variant of [`matches_selector_in_tree`]. Each ancestor
/// carries its own `MatchContext`, so pseudo-class compounds on
/// ancestor selectors (`div:hover .child`) resolve correctly.
pub fn matches_selector_in_tree_with_context(
  sel: &ComplexSelector,
  element: &Element,
  element_ctx: &MatchContext,
  ancestors: &[(&Element, MatchContext)],
) -> bool {
  let subj = sel.subject();
  if !matches_compound(subj, element) || !pseudo_classes_satisfied(subj, element_ctx) {
    return false;
  }
  let anc_comps = sel.ancestor_compounds();
  if anc_comps.is_empty() {
    return true;
  }
  let mut idx = 0usize;
  for required in anc_comps {
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
fn matches_compound(sel: &CompoundSelector, element: &Element) -> bool {
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
  for attr in &sel.attrs {
    let Some(actual) = element_attr(element, &attr.name) else {
      return false;
    };
    if attr.op != AttrOp::Exists && attr.op != AttrOp::Equals {
      continue; // skip non-simple operators in legacy matcher
    }
    if !attr.value.is_empty() && !actual.eq_ignore_ascii_case(&attr.value) {
      return false;
    }
  }
  true
}

/// Verify every pseudo-class on `sel` holds in `ctx`. AND-semantics:
/// `a:hover:active` requires both. Selectors without pseudo-classes
/// pass trivially.
fn pseudo_classes_satisfied(sel: &CompoundSelector, ctx: &MatchContext) -> bool {
  for pc in &sel.pseudo_classes {
    let ok = match pc {
      PseudoClass::Hover => ctx.is_hover,
      PseudoClass::Active => ctx.is_active,
      PseudoClass::Focus => ctx.is_focus,
      PseudoClass::Root => ctx.is_root,
      PseudoClass::FirstChild => ctx.is_first_child,
      PseudoClass::LastChild => ctx.is_last_child,
      _ => return false,
    };
    if !ok {
      return false;
    }
  }
  true
}

#[cfg(test)]
mod tests;
