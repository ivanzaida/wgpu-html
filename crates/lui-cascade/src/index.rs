use lui_core::{AtRule, CssAtRule, CssPseudo, StyleRule, Stylesheet};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

#[derive(Debug, Clone, Copy)]
pub struct RuleRef {
  pub rule_idx: usize,
  pub selector_idx: usize,
}

#[derive(Debug, Default)]
pub struct RuleIndex {
  pub by_id: FxHashMap<String, Vec<RuleRef>>,
  pub by_class: FxHashMap<String, Vec<RuleRef>>,
  pub by_tag: FxHashMap<String, Vec<RuleRef>>,
  pub by_attr: FxHashMap<String, Vec<RuleRef>>,
  pub by_pseudo_link: Vec<RuleRef>,
  pub by_pseudo_form: Vec<RuleRef>,
  pub universal: Vec<RuleRef>,
}

#[derive(Debug)]
pub struct ConditionalRule {
  pub rule: StyleRule,
  pub condition_idx: usize,
}

#[derive(Debug)]
pub enum RuleCondition {
  Media(lui_core::MediaQueryList),
  Supports(lui_core::SupportsCondition),
}

#[derive(Debug)]
pub struct PreparedStylesheet {
  pub rules: Vec<StyleRule>,
  pub conditional_rules: Vec<ConditionalRule>,
  pub conditions: Vec<RuleCondition>,
  pub index: RuleIndex,
  pub conditional_index: RuleIndex,
  /// Rule indices for each pseudo-element for fast lookup.
  pub pseudo_index: FxHashMap<CssPseudo, Vec<usize>>,
}

impl PreparedStylesheet {
  pub fn new(sheet: Stylesheet) -> Self {
    let mut conditions: Vec<RuleCondition> = Vec::new();
    let mut conditional_rules: Vec<ConditionalRule> = Vec::new();

    flatten_at_rules(&sheet.at_rules, &mut conditions, &mut conditional_rules);

    let index = build_index(&sheet.rules);
    let conditional_index = build_conditional_index(&conditional_rules);

    let pseudo_index = build_pseudo_index(&sheet.rules);

    Self {
      rules: sheet.rules,
      conditional_rules,
      conditions,
      index,
      conditional_index,
      pseudo_index,
    }
  }
}

fn flatten_at_rules(at_rules: &[AtRule], conditions: &mut Vec<RuleCondition>, out: &mut Vec<ConditionalRule>) {
  for at_rule in at_rules {
    match at_rule.at_rule {
      CssAtRule::Media => {
        if let Some(ref mql) = at_rule.media {
          let cond_idx = conditions.len();
          conditions.push(RuleCondition::Media(mql.clone()));
          for rule in &at_rule.rules {
            out.push(ConditionalRule {
              rule: rule.clone(),
              condition_idx: cond_idx,
            });
          }
          flatten_at_rules(&at_rule.at_rules, conditions, out);
        }
      }
      CssAtRule::Supports => {
        if let Some(ref sc) = at_rule.supports {
          let cond_idx = conditions.len();
          conditions.push(RuleCondition::Supports(sc.clone()));
          for rule in &at_rule.rules {
            out.push(ConditionalRule {
              rule: rule.clone(),
              condition_idx: cond_idx,
            });
          }
          flatten_at_rules(&at_rule.at_rules, conditions, out);
        }
      }
      _ => {}
    }
  }
}

fn build_index(rules: &[StyleRule]) -> RuleIndex {
  let mut index = RuleIndex::default();
  for (rule_idx, rule) in rules.iter().enumerate() {
    for (selector_idx, complex) in rule.selector.0.iter().enumerate() {
      let entry = RuleRef { rule_idx, selector_idx };
      index_selector(&mut index, &entry, complex);
    }
  }
  index
}

fn build_conditional_index(rules: &[ConditionalRule]) -> RuleIndex {
  let mut index = RuleIndex::default();
  for (rule_idx, cond_rule) in rules.iter().enumerate() {
    for (selector_idx, complex) in cond_rule.rule.selector.0.iter().enumerate() {
      let entry = RuleRef { rule_idx, selector_idx };
      index_selector(&mut index, &entry, complex);
    }
  }
  index
}

fn index_selector(index: &mut RuleIndex, entry: &RuleRef, complex: &lui_core::selector::ComplexSelector) {
  let subject = match complex.compounds.last() {
    Some(s) => s,
    None => return,
  };

  // Priority: id > class > tag > attr > pseudo-bucket > universal
  if let Some(ref id) = subject.id {
    index.by_id.entry(id.clone()).or_default().push(*entry);
    return;
  }
  if let Some(class) = subject.classes.first() {
    index.by_class.entry(class.clone()).or_default().push(*entry);
    return;
  }
  if let Some(ref tag) = subject.tag {
    if tag != "*" {
      index.by_tag.entry(tag.clone()).or_default().push(*entry);
      return;
    }
  }

  // Attribute selector → index by attribute name
  if let Some(attr) = subject.attrs.first() {
    index.by_attr.entry(attr.name.clone()).or_default().push(*entry);
    return;
  }

  // Pseudo-class only: route to type-specific buckets
  if !subject.pseudos.is_empty() {
    let pseudo = &subject.pseudos[0].pseudo;
    if is_link_pseudo(pseudo) {
      index.by_pseudo_link.push(*entry);
      return;
    }
    if is_form_pseudo(pseudo) {
      index.by_pseudo_form.push(*entry);
      return;
    }
  }

  index.universal.push(*entry);
}

fn is_link_pseudo(pseudo: &CssPseudo) -> bool {
  matches!(
    pseudo,
    CssPseudo::Link | CssPseudo::Visited | CssPseudo::AnyLink | CssPseudo::Active | CssPseudo::Hover | CssPseudo::Focus
  )
}

fn is_form_pseudo(pseudo: &CssPseudo) -> bool {
  matches!(
    pseudo,
    CssPseudo::Checked
      | CssPseudo::Disabled
      | CssPseudo::Enabled
      | CssPseudo::Required
      | CssPseudo::Optional
      | CssPseudo::ReadOnly
      | CssPseudo::ReadWrite
      | CssPseudo::PlaceholderShown
      | CssPseudo::Indeterminate
      | CssPseudo::Valid
      | CssPseudo::Invalid
      | CssPseudo::InRange
      | CssPseudo::OutOfRange
      | CssPseudo::Default
      | CssPseudo::Autofill
  )
}

const LINK_TAGS: &[&str] = &["a", "area", "link"];
const FORM_TAGS: &[&str] = &[
  "input", "textarea", "select", "button", "fieldset", "output", "option", "optgroup",
];

/// Collect candidate `RuleRef`s that might match an element.
pub fn candidate_rules<'a>(
  index: &'a RuleIndex,
  tag: &str,
  id: Option<&str>,
  classes: &[&str],
  attrs: &std::collections::HashMap<lui_core::ArcStr, lui_core::ArcStr>,
  data_attrs: &std::collections::HashMap<lui_core::ArcStr, lui_core::ArcStr>,
  aria_attrs: &std::collections::HashMap<lui_core::ArcStr, lui_core::ArcStr>,
) -> SmallVec<[&'a RuleRef; 32]> {
  let mut seen = FxHashSet::default();
  let mut candidates: SmallVec<[&'a RuleRef; 32]> = smallvec::smallvec![];

  let mut add = |refs: &'a [RuleRef]| {
    for r in refs {
      let key = (r.rule_idx, r.selector_idx);
      if seen.insert(key) {
        candidates.push(r);
      }
    }
  };

  if let Some(id) = id {
    if let Some(refs) = index.by_id.get(id) {
      add(refs);
    }
  }

  for class in classes {
    if let Some(refs) = index.by_class.get(*class) {
      add(refs);
    }
  }

  if let Some(refs) = index.by_tag.get(tag) {
    add(refs);
  }

  // Attribute-indexed rules: check all attribute types
  if !index.by_attr.is_empty() {
    for attr_name in attrs.keys() {
      if let Some(refs) = index.by_attr.get(attr_name.as_ref()) {
        add(refs);
      }
    }
    for key in data_attrs.keys() {
      let prefixed = format!("data-{}", key);
      if let Some(refs) = index.by_attr.get(&prefixed) {
        add(refs);
      }
    }
    for key in aria_attrs.keys() {
      let prefixed = format!("aria-{}", key);
      if let Some(refs) = index.by_attr.get(&prefixed) {
        add(refs);
      }
    }
  }

  // Link pseudo-class rules: only for link-like elements
  if LINK_TAGS.contains(&tag) {
    add(&index.by_pseudo_link);
  }

  // Form pseudo-class rules: only for form elements
  if FORM_TAGS.contains(&tag) {
    add(&index.by_pseudo_form);
  }

  add(&index.universal);

  candidates
}

fn build_pseudo_index(rules: &[StyleRule]) -> FxHashMap<CssPseudo, Vec<usize>> {
  let mut map: FxHashMap<CssPseudo, Vec<usize>> = FxHashMap::default();
  for (rule_idx, rule) in rules.iter().enumerate() {
    for complex in &rule.selector.0 {
      if let Some(compound) = complex.compounds.last() {
        for pseudo_sel in &compound.pseudos {
          let pe = &pseudo_sel.pseudo;
          // Only index pseudo-elements (::before, ::after, etc.), not pseudo-classes (:hover)
          if is_pseudo_element(pe) {
            map.entry(pe.clone()).or_default().push(rule_idx);
          }
        }
      }
    }
  }
  // Deduplicate
  for indices in map.values_mut() {
    indices.sort_unstable();
    indices.dedup();
  }
  map
}

fn is_pseudo_element(pseudo: &CssPseudo) -> bool {
  matches!(
    pseudo,
    CssPseudo::After
      | CssPseudo::Before
      | CssPseudo::FirstLine
      | CssPseudo::FirstLetter
      | CssPseudo::Placeholder
      | CssPseudo::Selection
      | CssPseudo::Marker
      | CssPseudo::Backdrop
      | CssPseudo::FileSelectorButton
      | CssPseudo::LuiScrollbar
      | CssPseudo::LuiScrollbarThumb
      | CssPseudo::LuiScrollbarTrack
      | CssPseudo::LuiScrollbarCorner
  )
}
