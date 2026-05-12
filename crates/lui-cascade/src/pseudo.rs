use bumpalo::Bump;
use lui_css_parser::{ArcStr, CssPseudo, CssValue, StyleRule};

use crate::PseudoElementStyle;
use crate::index::PreparedStylesheet;
use crate::matching::{AncestorEntry, MatchContext, matches_selector};
use crate::media::MediaContext;
use crate::style::ComputedStyle;

use crate::cascade::apply_declaration_ref;

/// Collect the ::before or ::after pseudo-element style for a node,
/// if any rule generates content for it.
pub fn collect_pseudo_element<'a>(
    pseudo: CssPseudo,
    node: &'a lui_html_parser::HtmlNode,
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
        for rule in &sheet.rules {
            if !rule_has_pseudo_element(&rule, &pseudo) {
                continue;
            }
            if !rule.selector.0.iter().any(|sel| {
                matches_selector(sel, node, ctx, ancestors, parent)
            }) {
                continue;
            }
            for decl in &rule.declarations {
                if !decl.important {
                    apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
                }
            }
        }

        // !important pass
        for rule in &sheet.rules {
            if !rule_has_pseudo_element(&rule, &pseudo) {
                continue;
            }
            if !rule.selector.0.iter().any(|sel| {
                matches_selector(sel, node, ctx, ancestors, parent)
            }) {
                continue;
            }
            for decl in &rule.declarations {
                if decl.important {
                    apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
                }
            }
        }
    }

    // Check for content property
    if let Some(content_val) = style.content {
        let is_none_or_normal = matches!(content_val,
            CssValue::String(s) if s.as_ref() == "none" || s.as_ref() == "normal"
        ) || matches!(content_val,
            CssValue::Unknown(s) if s.as_ref() == "none" || s.as_ref() == "normal"
        );
        if !is_none_or_normal {
            match content_val {
                CssValue::String(s) if !s.is_empty() => { has_content = true; }
                CssValue::Unknown(s) if !s.is_empty() => { has_content = true; }
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

fn rule_has_pseudo_element(rule: &StyleRule, target: &CssPseudo) -> bool {
    rule.selector.0.iter().any(|complex| {
        complex.compounds.last().map_or(false, |compound| {
            compound.pseudos.iter().any(|p| std::mem::discriminant(&p.pseudo) == std::mem::discriminant(target))
        })
    })
}

/// Collect rules targeting `pseudo` and merge their declarations into a
/// `ComputedStyle`. Does NOT check for `content` — use this for
/// `::first-line`, `::first-letter`, `::placeholder`, `::selection`, `::marker`.
pub fn collect_pseudo_style<'a>(
    pseudo: CssPseudo,
    node: &'a lui_html_parser::HtmlNode,
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
        for rule in &sheet.rules {
            if !rule_has_pseudo_element(&rule, &pseudo) {
                continue;
            }
            if !rule.selector.0.iter().any(|sel| {
                matches_selector(sel, node, ctx, ancestors, parent)
            }) {
                continue;
            }
            any_match = true;
            for decl in &rule.declarations {
                if !decl.important {
                    apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
                }
            }
        }

        for rule in &sheet.rules {
            if !rule_has_pseudo_element(&rule, &pseudo) {
                continue;
            }
            if !rule.selector.0.iter().any(|sel| {
                matches_selector(sel, node, ctx, ancestors, parent)
            }) {
                continue;
            }
            for decl in &rule.declarations {
                if decl.important {
                    apply_declaration_ref(&mut style, &decl.property, &decl.value, arena);
                }
            }
        }
    }

    if !any_match {
        return None;
    }

    style.inherit_from(parent_style);
    Some(Box::new(style))
}
