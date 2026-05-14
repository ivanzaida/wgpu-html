use lui_core::{
  CssCombinator, CssPseudo, SelectorList,
  selector::{AttrOp, AttributeSelector, ComplexSelector, CompoundSelector, PseudoSelector},
};
use lui_parse::HtmlNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
  Ltr,
  Rtl,
}

#[derive(Debug, Clone, Default)]
pub struct MatchContext<'a> {
  // ── Structural position ──
  pub is_root: bool,
  pub is_first_child: bool,
  pub is_last_child: bool,
  pub is_only_child: bool,
  pub sibling_index: usize,
  pub sibling_count: usize,

  // ── Interactive state ──
  pub is_hover: bool,
  pub is_active: bool,
  pub is_focus: bool,
  pub is_focus_visible: bool,
  pub is_focus_within: bool,

  // ── Document / navigation state ──
  pub is_target: bool,
  pub lang: Option<&'a str>,
  pub dir: Option<Dir>,

  // ── Viewport / API state ──
  pub is_fullscreen: bool,
  pub is_modal: bool,
}

#[derive(Debug, Clone)]
pub struct AncestorEntry<'a> {
  pub node: &'a HtmlNode,
  pub ctx: MatchContext<'a>,
}

pub fn any_selector_matches<'a>(
  list: &SelectorList,
  node: &'a HtmlNode,
  ctx: &MatchContext<'_>,
  ancestors: &[AncestorEntry<'a>],
  parent_node: Option<&'a HtmlNode>,
) -> Option<usize> {
  list.0.iter().enumerate().find_map(|(i, sel)| {
    if matches_selector(sel, node, ctx, ancestors, parent_node) {
      Some(i)
    } else {
      None
    }
  })
}

pub fn matches_selector<'a>(
  selector: &ComplexSelector,
  node: &'a HtmlNode,
  ctx: &MatchContext<'_>,
  ancestors: &[AncestorEntry<'a>],
  parent_node: Option<&'a HtmlNode>,
) -> bool {
  let n = selector.compounds.len();
  if n == 0 {
    return false;
  }

  let subject = &selector.compounds[n - 1];
  if !matches_compound(subject, node, ctx, parent_node, ancestors) {
    return false;
  }

  if n == 1 {
    return true;
  }

  match_ancestor_chain(
    &selector.compounds[..n - 1],
    &selector.combinators,
    ancestors,
    parent_node,
  )
}

fn match_ancestor_chain(
  compounds: &[CompoundSelector],
  combinators: &[CssCombinator],
  ancestors: &[AncestorEntry<'_>],
  parent_node: Option<&HtmlNode>,
) -> bool {
  let mut compound_idx = compounds.len();
  let mut ancestor_idx: usize = 0;

  while compound_idx > 0 {
    compound_idx -= 1;
    let compound = &compounds[compound_idx];
    let combinator = combinators[compound_idx];

    match combinator {
      CssCombinator::Descendant => {
        let mut found = false;
        while ancestor_idx < ancestors.len() {
          let entry = &ancestors[ancestor_idx];
          let anc_parent = ancestors.get(ancestor_idx + 1).map(|e| e.node);
          let further = &ancestors[ancestor_idx + 1..];
          ancestor_idx += 1;
          if matches_compound(compound, entry.node, &entry.ctx, anc_parent, further) {
            found = true;
            break;
          }
        }
        if !found {
          return false;
        }
      }
      CssCombinator::Child => {
        if ancestor_idx >= ancestors.len() {
          return false;
        }
        let entry = &ancestors[ancestor_idx];
        let anc_parent = ancestors.get(ancestor_idx + 1).map(|e| e.node);
        let further = &ancestors[ancestor_idx + 1..];
        ancestor_idx += 1;
        if !matches_compound(compound, entry.node, &entry.ctx, anc_parent, further) {
          return false;
        }
      }
      CssCombinator::NextSibling => {
        let sibling = find_preceding_sibling(parent_node, ancestors, ancestor_idx);
        match sibling {
          Some((sib_node, sib_ctx, sib_parent)) => {
            if !matches_compound(compound, sib_node, &sib_ctx, sib_parent, &[]) {
              return false;
            }
          }
          None => return false,
        }
      }
      CssCombinator::SubsequentSibling => {
        let siblings = collect_preceding_siblings(parent_node, ancestors, ancestor_idx);
        let found = siblings
          .iter()
          .any(|(sib_node, sib_ctx, sib_parent)| matches_compound(compound, sib_node, sib_ctx, *sib_parent, &[]));
        if !found {
          return false;
        }
      }
      CssCombinator::Column => {
        return false;
      }
    }
  }
  true
}

fn find_preceding_sibling<'a>(
  parent_node: Option<&'a HtmlNode>,
  ancestors: &[AncestorEntry<'a>],
  ancestor_idx: usize,
) -> Option<(&'a HtmlNode, MatchContext<'static>, Option<&'a HtmlNode>)> {
  let parent = if ancestor_idx == 0 {
    parent_node?
  } else {
    ancestors.get(ancestor_idx - 1)?.node
  };

  let current_entry = if ancestor_idx == 0 {
    return None;
  } else {
    &ancestors[ancestor_idx - 1]
  };

  let current_sib_idx = current_entry.ctx.sibling_index;
  if current_sib_idx == 0 {
    return None;
  }

  let count = parent.children.len();
  let prev = &parent.children[current_sib_idx - 1];
  let prev_ctx = MatchContext {
    is_first_child: current_sib_idx - 1 == 0,
    is_last_child: current_sib_idx - 1 == count - 1,
    is_only_child: count == 1,
    sibling_index: current_sib_idx - 1,
    sibling_count: count,
    ..Default::default()
  };
  Some((prev, prev_ctx, Some(parent)))
}

fn collect_preceding_siblings<'a>(
  parent_node: Option<&'a HtmlNode>,
  ancestors: &[AncestorEntry<'a>],
  ancestor_idx: usize,
) -> Vec<(&'a HtmlNode, MatchContext<'static>, Option<&'a HtmlNode>)> {
  let parent = if ancestor_idx == 0 {
    match parent_node {
      Some(p) => p,
      None => return vec![],
    }
  } else {
    match ancestors.get(ancestor_idx - 1) {
      Some(e) => e.node,
      None => return vec![],
    }
  };

  let current_sib_idx = if ancestor_idx == 0 {
    return vec![];
  } else {
    ancestors[ancestor_idx - 1].ctx.sibling_index
  };

  let count = parent.children.len();
  (0..current_sib_idx)
    .map(|i| {
      let node = &parent.children[i];
      let ctx = MatchContext {
        is_first_child: i == 0,
        is_last_child: i == count - 1,
        is_only_child: count == 1,
        sibling_index: i,
        sibling_count: count,
        ..Default::default()
      };
      (node, ctx, Some(parent))
    })
    .collect()
}

// ---------------------------------------------------------------------------
// Compound matching
// ---------------------------------------------------------------------------

pub fn matches_compound(
  compound: &CompoundSelector,
  node: &HtmlNode,
  ctx: &MatchContext<'_>,
  parent_node: Option<&HtmlNode>,
  ancestors: &[AncestorEntry<'_>],
) -> bool {
  if let Some(ref tag) = compound.tag {
    if tag != "*" && tag != node.element.tag_name() {
      return false;
    }
  }

  if let Some(ref id) = compound.id {
    match &node.id {
      Some(node_id) if node_id.as_ref() == id.as_str() => {}
      _ => return false,
    }
  }

  if !compound.classes.is_empty() {
    for class in &compound.classes {
      if !node.class_list.iter().any(|c| c.as_ref() == class) {
        return false;
      }
    }
  }

  for attr_sel in &compound.attrs {
    if !matches_attr(attr_sel, node) {
      return false;
    }
  }

  for pseudo in &compound.pseudos {
    if !matches_pseudo(pseudo, node, ctx, parent_node, ancestors) {
      return false;
    }
  }

  true
}

pub fn is_pseudo_element(pseudo: &CssPseudo) -> bool {
  matches!(
    pseudo,
    CssPseudo::Before
      | CssPseudo::After
      | CssPseudo::FirstLine
      | CssPseudo::FirstLetter
      | CssPseudo::Placeholder
      | CssPseudo::Selection
      | CssPseudo::Marker
      | CssPseudo::Backdrop
      | CssPseudo::BeforeLegacy
      | CssPseudo::AfterLegacy
      | CssPseudo::FirstLineLegacy
      | CssPseudo::FirstLetterLegacy
      | CssPseudo::Part
      | CssPseudo::Slotted
      | CssPseudo::Highlight
      | CssPseudo::GrammarError
      | CssPseudo::SpellingError
      | CssPseudo::TargetText
      | CssPseudo::SearchText
      | CssPseudo::ViewTransition
      | CssPseudo::ViewTransitionGroup
      | CssPseudo::ViewTransitionGroupChildren
      | CssPseudo::ViewTransitionImagePair
      | CssPseudo::ViewTransitionNew
      | CssPseudo::ViewTransitionOld
      | CssPseudo::CueFn
      | CssPseudo::CueRegionFn
      | CssPseudo::NthFragment
      | CssPseudo::Column
      | CssPseudo::FileSelectorButton
      | CssPseudo::Picker
      | CssPseudo::PickerIcon
      | CssPseudo::Checkmark
      | CssPseudo::ClearIcon
      | CssPseudo::ColorSwatch
      | CssPseudo::RevealIcon
      | CssPseudo::ScrollButton
      | CssPseudo::ScrollMarker
      | CssPseudo::ScrollMarkerGroup
      | CssPseudo::SliderFill
      | CssPseudo::SliderThumb
      | CssPseudo::SliderTrack
      | CssPseudo::StepControl
      | CssPseudo::StepDown
      | CssPseudo::StepUp
      | CssPseudo::FieldComponent
      | CssPseudo::FieldSeparator
      | CssPseudo::FieldText
      | CssPseudo::DetailsContent
  )
}

// ---------------------------------------------------------------------------
// Attribute matching
// ---------------------------------------------------------------------------

fn matches_attr(sel: &AttributeSelector, node: &HtmlNode) -> bool {
  let name = sel.name.as_str();
  let val = if name == "id" {
    node.id.as_ref()
  } else if name == "class" {
    None // class is in class_list, attribute presence check below handles it
  } else {
    node
      .attrs
      .get(name)
      .or_else(|| name.strip_prefix("data-").and_then(|rest| node.data_attrs.get(rest)))
      .or_else(|| name.strip_prefix("aria-").and_then(|rest| node.aria_attrs.get(rest)))
  };

  // Special case: [class] presence check
  if name == "class" && sel.op.is_none() {
    return !node.class_list.is_empty();
  }
  if name == "class" {
    let class_str: String = node.class_list.iter().map(|c| c.as_ref()).collect::<Vec<_>>().join(" ");
    let val_ref = &class_str;
    let expected = match &sel.value {
      Some(v) => v.as_str(),
      None => return false,
    };
    let case_insensitive = sel.modifier == Some('i');
    let (val_s, expected_s) = if case_insensitive {
      (val_ref.to_ascii_lowercase(), expected.to_ascii_lowercase())
    } else {
      (val_ref.to_string(), expected.to_string())
    };
    return match sel.op.as_ref().unwrap() {
      AttrOp::Eq => val_s == expected_s,
      AttrOp::Includes => val_s.split_ascii_whitespace().any(|w| w == expected_s),
      AttrOp::Hyphen => val_s == expected_s || val_s.starts_with(&format!("{}-", expected_s)),
      AttrOp::StartsWith => val_s.starts_with(expected_s.as_str()),
      AttrOp::EndsWith => val_s.ends_with(expected_s.as_str()),
      AttrOp::Contains => val_s.contains(expected_s.as_str()),
    };
  }

  let Some(op) = &sel.op else {
    return val.is_some();
  };

  let val = match val {
    Some(v) => v.as_ref(),
    None => return false,
  };

  let expected = match &sel.value {
    Some(v) => v.as_str(),
    None => return false,
  };

  let case_insensitive = sel.modifier == Some('i');

  let (val, expected) = if case_insensitive {
    (val.to_ascii_lowercase(), expected.to_ascii_lowercase())
  } else {
    (val.to_string(), expected.to_string())
  };

  match op {
    AttrOp::Eq => val == expected,
    AttrOp::Includes => val.split_ascii_whitespace().any(|w| w == expected),
    AttrOp::Hyphen => val == expected || val.starts_with(&format!("{}-", expected)),
    AttrOp::StartsWith => val.starts_with(expected.as_str()),
    AttrOp::EndsWith => val.ends_with(expected.as_str()),
    AttrOp::Contains => val.contains(expected.as_str()),
  }
}

// ---------------------------------------------------------------------------
// Pseudo-class matching
// ---------------------------------------------------------------------------

fn matches_pseudo(
  pseudo: &PseudoSelector,
  node: &HtmlNode,
  ctx: &MatchContext<'_>,
  parent_node: Option<&HtmlNode>,
  ancestors: &[AncestorEntry<'_>],
) -> bool {
  match &pseudo.pseudo {
    // ── Interactive state ──
    CssPseudo::Hover => ctx.is_hover,
    CssPseudo::Active => ctx.is_active,
    CssPseudo::Focus => ctx.is_focus,
    CssPseudo::FocusVisible => ctx.is_focus_visible,
    CssPseudo::FocusWithin => ctx.is_focus_within,

    // ── Structural position ──
    CssPseudo::Root => ctx.is_root,
    CssPseudo::Empty => node.children.is_empty(),
    CssPseudo::FirstChild => ctx.is_first_child,
    CssPseudo::LastChild => ctx.is_last_child,
    CssPseudo::OnlyChild => ctx.is_only_child,
    CssPseudo::NthChild => match_nth(pseudo.arg.as_deref(), ctx.sibling_index),
    CssPseudo::NthLastChild => {
      let from_end = ctx.sibling_count.saturating_sub(ctx.sibling_index + 1);
      match_nth(pseudo.arg.as_deref(), from_end)
    }

    // ── Type-based structural ──
    CssPseudo::FirstOfType => match parent_node {
      Some(parent) => {
        let tag = node.element.tag_name();
        !parent.children[..ctx.sibling_index]
          .iter()
          .any(|c| c.element.tag_name() == tag)
      }
      None => true,
    },
    CssPseudo::LastOfType => match parent_node {
      Some(parent) => {
        let tag = node.element.tag_name();
        !parent.children[ctx.sibling_index + 1..]
          .iter()
          .any(|c| c.element.tag_name() == tag)
      }
      None => true,
    },
    CssPseudo::OnlyOfType => match parent_node {
      Some(parent) => {
        let tag = node.element.tag_name();
        parent.children.iter().filter(|c| c.element.tag_name() == tag).count() == 1
      }
      None => true,
    },
    CssPseudo::NthOfType => match parent_node {
      Some(parent) => {
        let tag = node.element.tag_name();
        let type_index = parent.children[..ctx.sibling_index]
          .iter()
          .filter(|c| c.element.tag_name() == tag)
          .count();
        match_nth(pseudo.arg.as_deref(), type_index)
      }
      None => match_nth(pseudo.arg.as_deref(), 0),
    },
    CssPseudo::NthLastOfType => match parent_node {
      Some(parent) => {
        let tag = node.element.tag_name();
        let from_end = parent.children[ctx.sibling_index + 1..]
          .iter()
          .filter(|c| c.element.tag_name() == tag)
          .count();
        match_nth(pseudo.arg.as_deref(), from_end)
      }
      None => match_nth(pseudo.arg.as_deref(), 0),
    },

    // ── Link state ──
    CssPseudo::Link | CssPseudo::AnyLink => {
      matches!(node.element.tag_name(), "a" | "area" | "link") && node.attrs.contains_key("href")
    }
    CssPseudo::Visited => false,

    // ── Form / input state ──
    CssPseudo::Enabled => !node.attrs.contains_key("disabled"),
    CssPseudo::Disabled => node.attrs.contains_key("disabled"),
    CssPseudo::Checked => node.attrs.contains_key("checked"),
    CssPseudo::Indeterminate => node.attrs.contains_key("indeterminate"),
    CssPseudo::Required => node.attrs.contains_key("required"),
    CssPseudo::Optional => !node.attrs.contains_key("required"),
    CssPseudo::ReadOnly => node.attrs.contains_key("readonly"),
    CssPseudo::ReadWrite => !node.attrs.contains_key("readonly"),
    CssPseudo::PlaceholderShown => node.attrs.contains_key("placeholder") && node.children.is_empty(),
    CssPseudo::Default => node.attrs.contains_key("default"),
    CssPseudo::Valid
    | CssPseudo::Invalid
    | CssPseudo::UserValid
    | CssPseudo::UserInvalid
    | CssPseudo::InRange
    | CssPseudo::OutOfRange => false,
    CssPseudo::Autofill => false,

    // ── UI state ──
    CssPseudo::Open => node.attrs.contains_key("open"),
    CssPseudo::Fullscreen => ctx.is_fullscreen,
    CssPseudo::Modal => ctx.is_modal,
    CssPseudo::PictureInPicture => false,
    CssPseudo::PopoverOpen => {
      node.attrs.contains_key("popover") && node.attrs.get("popover").map(|v| !v.is_empty()).unwrap_or(false)
    }

    // ── Document / navigation ──
    CssPseudo::Target => ctx.is_target,
    CssPseudo::Scope => ctx.is_root,

    // ── Language / direction ──
    CssPseudo::Lang => match (&pseudo.arg, ctx.lang) {
      (Some(arg), Some(lang)) => lang == arg.as_str() || lang.starts_with(&format!("{}-", arg)),
      _ => false,
    },
    CssPseudo::Dir => match (&pseudo.arg, ctx.dir) {
      (Some(arg), Some(Dir::Ltr)) => arg == "ltr",
      (Some(arg), Some(Dir::Rtl)) => arg == "rtl",
      _ => false,
    },

    // ── Custom element ──
    CssPseudo::Defined => node.element.is_known(),

    // ── Media playback (require runtime state we don't have) ──
    CssPseudo::Playing
    | CssPseudo::Paused
    | CssPseudo::Seeking
    | CssPseudo::Buffering
    | CssPseudo::Muted
    | CssPseudo::Stalled
    | CssPseudo::VolumeLocked => false,

    // ── Functional pseudo-classes ──
    CssPseudo::Not => match &pseudo.arg {
      Some(arg) => match lui_parse::parse_selector_list(arg) {
        Ok(inner) => !inner
          .0
          .iter()
          .any(|sel| matches_selector(sel, node, ctx, ancestors, parent_node)),
        Err(_) => true,
      },
      None => true,
    },
    CssPseudo::Is | CssPseudo::Where | CssPseudo::Matches => match &pseudo.arg {
      Some(arg) => match lui_parse::parse_selector_list(arg) {
        Ok(inner) => inner
          .0
          .iter()
          .any(|sel| matches_selector(sel, node, ctx, ancestors, parent_node)),
        Err(_) => false,
      },
      None => false,
    },
    CssPseudo::Has => match &pseudo.arg {
      Some(arg) => match_has(node, arg.trim()),
      None => false,
    },

    // ── Pseudo-elements: pass through, filtered at cascade time ──
    p if is_pseudo_element(p) => true,

    // ── Everything else: reject ──
    _ => false,
  }
}

// ---------------------------------------------------------------------------
// :has() implementation
// ---------------------------------------------------------------------------

fn match_has(node: &HtmlNode, arg: &str) -> bool {
  if let Some(rest) = arg.strip_prefix('>') {
    match lui_parse::parse_selector_list(rest.trim()) {
      Ok(inner) => node.children.iter().enumerate().any(|(i, child)| {
        let ctx = child_ctx_for(i, node.children.len());
        inner
          .0
          .iter()
          .any(|sel| matches_compound_simple(&sel.compounds, child, &ctx, Some(node)))
      }),
      Err(_) => false,
    }
  } else if arg.starts_with('+') || arg.starts_with('~') {
    // :has(+ .sibling) / :has(~ .sibling) — requires parent context, not supported
    false
  } else {
    match lui_parse::parse_selector_list(arg) {
      Ok(inner) => has_matching_descendant(node, &inner),
      Err(_) => false,
    }
  }
}

fn has_matching_descendant(root: &HtmlNode, selectors: &SelectorList) -> bool {
  for (i, child) in root.children.iter().enumerate() {
    let ctx = child_ctx_for(i, root.children.len());
    if selectors
      .0
      .iter()
      .any(|sel| matches_selector_for_has(sel, child, &ctx, root))
    {
      return true;
    }
    if has_matching_descendant(child, selectors) {
      return true;
    }
  }
  false
}

fn matches_selector_for_has(
  selector: &ComplexSelector,
  node: &HtmlNode,
  ctx: &MatchContext<'_>,
  parent: &HtmlNode,
) -> bool {
  let n = selector.compounds.len();
  if n == 0 {
    return false;
  }
  let subject = &selector.compounds[n - 1];
  matches_compound(subject, node, ctx, Some(parent), &[])
}

fn matches_compound_simple(
  compounds: &[CompoundSelector],
  node: &HtmlNode,
  ctx: &MatchContext<'_>,
  parent: Option<&HtmlNode>,
) -> bool {
  if let Some(subject) = compounds.last() {
    matches_compound(subject, node, ctx, parent, &[])
  } else {
    false
  }
}

fn child_ctx_for(index: usize, count: usize) -> MatchContext<'static> {
  MatchContext {
    is_first_child: index == 0,
    is_last_child: index == count - 1,
    is_only_child: count == 1,
    sibling_index: index,
    sibling_count: count,
    ..Default::default()
  }
}

// ---------------------------------------------------------------------------
// An+B matching
// ---------------------------------------------------------------------------

fn match_nth(arg: Option<&str>, index: usize) -> bool {
  let arg = match arg {
    Some(a) => a.trim(),
    None => return false,
  };

  let pos = index + 1; // CSS :nth-child is 1-based

  if arg.eq_ignore_ascii_case("odd") {
    return pos % 2 == 1;
  }
  if arg.eq_ignore_ascii_case("even") {
    return pos % 2 == 0;
  }

  if let Ok(n) = arg.parse::<usize>() {
    return pos == n;
  }

  let arg = arg.to_ascii_lowercase().replace(" ", "");

  let n_pos = match arg.find('n') {
    Some(p) => p,
    None => return false,
  };

  let a_str = &arg[..n_pos];
  let a: i64 = match a_str {
    "" | "+" => 1,
    "-" => -1,
    _ => match a_str.parse() {
      Ok(v) => v,
      Err(_) => return false,
    },
  };

  let rest = &arg[n_pos + 1..];
  let b: i64 = if rest.is_empty() {
    0
  } else {
    match rest.parse() {
      Ok(v) => v,
      Err(_) => return false,
    }
  };

  if a == 0 {
    return pos as i64 == b;
  }

  let diff = pos as i64 - b;
  diff % a == 0 && diff / a >= 0
}
