use bumpalo::Bump;
use lui_core::{ArcStr, CssPseudo, CssValue};

use crate::{
  PseudoElementStyle,
  cascade::apply_declaration_ref,
  index::PreparedStylesheet,
  matching::{AncestorEntry, MatchContext, matches_selector},
  media::MediaContext,
  style::ComputedStyle,
};

/// Collect the ::before or ::after pseudo-element style for a node,
/// if any rule generates content for it.
pub fn collect_pseudo_element<'a>(
  pseudo: CssPseudo,
  node: &'a lui_parse::HtmlNode,
  parent_style: &ComputedStyle<'a>,
  sheets: &[&'a PreparedStylesheet],
  ancestors: &[AncestorEntry<'_>],
  ctx: &MatchContext<'_>,
  _media: &MediaContext,
  arena: &'a Bump,
) -> Option<Box<PseudoElementStyle<'a>>> {
  let mut style = ComputedStyle::default();
  let mut has_content = false;

  let parent = ancestors.first().map(|a| a.node);

  for sheet in sheets {
    let matched_rules = matching_rules_for_pseudo(sheet, &pseudo, node, ctx, ancestors, parent, arena, false);
    for decl in matched_rules {
      if !decl.important {
        apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
      }
    }
    let matched_rules = matching_rules_for_pseudo(sheet, &pseudo, node, ctx, ancestors, parent, arena, true);
    for decl in matched_rules {
      apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
    }
  }

  if let Some(content_val) = style.content {
    let is_none_or_normal = matches!(content_val,
        CssValue::String(s) if s.as_ref() == "none" || s.as_ref() == "normal"
    ) || matches!(content_val,
        CssValue::Unknown(s) if s.as_ref() == "none" || s.as_ref() == "normal"
    );
    if !is_none_or_normal {
      match content_val {
        CssValue::String(s) if !s.is_empty() => {
          has_content = true;
        }
        CssValue::Unknown(s) if !s.is_empty() => {
          has_content = true;
        }
        _ => {}
      }
    }
  }

  if !has_content {
    return None;
  }

  style.inherit_from(parent_style);

  let content_text = match style.content {
    Some(CssValue::String(s)) => ArcStr::from(s.as_ref()),
    Some(CssValue::Unknown(s)) => ArcStr::from(s.as_ref()),
    _ => ArcStr::from(""),
  };

  Some(Box::new(PseudoElementStyle { style, content_text }))
}

/// Collect rules targeting `pseudo` and merge their declarations into a
/// `ComputedStyle`. For `::first-line`, `::first-letter`, `::placeholder`,
/// `::selection`, `::marker`.
pub fn collect_pseudo_style<'a>(
  pseudo: CssPseudo,
  node: &'a lui_parse::HtmlNode,
  parent_style: &ComputedStyle<'a>,
  sheets: &[&'a PreparedStylesheet],
  ancestors: &[AncestorEntry<'_>],
  ctx: &MatchContext<'_>,
  _media: &MediaContext,
  arena: &'a Bump,
) -> Option<Box<ComputedStyle<'a>>> {
  let mut style = ComputedStyle::default();
  let mut any_match = false;

  let parent = ancestors.first().map(|a| a.node);

  for sheet in sheets {
    let matched_rules = matching_rules_for_pseudo(sheet, &pseudo, node, ctx, ancestors, parent, arena, false);
    for decl in matched_rules {
      any_match = true;
      apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
    }
    let matched_rules = matching_rules_for_pseudo(sheet, &pseudo, node, ctx, ancestors, parent, arena, true);
    for decl in matched_rules {
      apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
    }
  }

  if !any_match {
    return None;
  }

  style.inherit_from(parent_style);
  Some(Box::new(style))
}

/// Collect matching declarations for a pseudo-element using the index.
fn matching_rules_for_pseudo<'a>(
  sheet: &'a PreparedStylesheet,
  pseudo: &CssPseudo,
  node: &lui_parse::HtmlNode,
  ctx: &MatchContext<'_>,
  ancestors: &[AncestorEntry<'_>],
  parent: Option<&lui_parse::HtmlNode>,
  _arena: &'a Bump,
  important: bool,
) -> Vec<&'a lui_core::Declaration> {
  let mut decls = Vec::new();
  let Some(rule_indices) = sheet.pseudo_index.get(pseudo) else {
    return decls;
  };
  for &rule_idx in rule_indices {
    let rule = &sheet.rules[rule_idx];
    if !rule
      .selector
      .0
      .iter()
      .any(|sel| matches_selector(sel, node, ctx, ancestors, parent))
    {
      continue;
    }
    for decl in &rule.declarations {
      if decl.important == important {
        decls.push(decl);
      }
    }
  }
  decls
}
