//! CSS stylesheet parsing — selectors + rules.
//!
//! Structural parsing (tokenization, rule extraction, at-rule dispatch,
//! comment stripping, brace matching) is delegated to [`lui_css_old::CssParser`].
//! This module converts the generic `lui_css_old` AST into the engine's typed
//! [`Rule`] format that the cascade in `lui-style` expects.

use std::collections::HashMap;

pub use lui_css_old::stylesheet::{MediaFeature, MediaQuery, MediaQueryList, MediaType};
use lui_models::{ArcStr, Style};
pub use lui_tree::query::{
  AttrFilter, AttrOp, Combinator, ComplexSelector, CompoundSelector, MatchContext, PseudoClass, PseudoElement,
  SelectorList,
};

use crate::css_parser::{CssWideKeyword, parse_inline_style_decls};

/// One rule: any of the listed selectors triggers the declarations.
/// `declarations` holds the normal-importance properties; `important`
/// holds the ones marked `!important`. Cascade applies them in
/// separate passes per CSS-Cascade-3 §6.4.
///
/// `keywords` and `important_keywords` carry per-property CSS-wide
/// keywords (`inherit / initial / unset`) that override any matching
/// value the cascade has accumulated. Keys are CSS property names in
/// kebab-case (`color`, `font-size`, …).
#[derive(Debug, Clone, Default)]
pub struct Rule {
  pub selectors: SelectorList,
  pub declarations: Style,
  pub important: Style,
  pub keywords: HashMap<ArcStr, CssWideKeyword>,
  pub important_keywords: HashMap<ArcStr, CssWideKeyword>,
  /// Active media conditions enclosing this rule. Multiple entries
  /// come from nested `@media` blocks and are ANDed by the cascade.
  pub media: Vec<MediaQueryList>,
}

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
  pub rules: Vec<Rule>,
}

impl Stylesheet {
  pub fn append(&mut self, other: Stylesheet) {
    self.rules.extend(other.rules);
  }
}

// ---------------------------------------------------------------------------
// Parsing — delegates to lui_css_old, then converts
// ---------------------------------------------------------------------------

pub fn parse_stylesheet(css: &str) -> Stylesheet {
  let parser = lui_css_old::CssParser::new();
  let sheet = parser.parse_stylesheet(css);
  let mut rules = Vec::new();
  convert_rules(&sheet.rules, &mut Vec::new(), &mut rules);
  Stylesheet { rules }
}

pub fn parse_media_query_list(input: &str) -> Option<MediaQueryList> {
  lui_css_old::at_rules::MediaAtRuleParser::parse_media_query_list_from(input)
}

fn convert_rules(css_rules: &[lui_css_old::CssRule], media_stack: &mut Vec<MediaQueryList>, rules: &mut Vec<Rule>) {
  for css_rule in css_rules {
    match css_rule {
      lui_css_old::CssRule::Style(style_rule) => {
        let selectors = SelectorList::from(style_rule.selector_text.as_ref());
        if selectors.is_empty() {
          continue;
        }
        let decls = parse_inline_style_decls(
          &style_rule
            .declarations
            .declarations
            .iter()
            .map(|d| {
              let imp = if d.importance == lui_css_old::Importance::Important {
                " !important"
              } else {
                ""
              };
              format!("{}: {}{}", &*d.property, &*d.value, imp)
            })
            .collect::<Vec<_>>()
            .join("; "),
        );
        rules.push(Rule {
          selectors,
          declarations: decls.normal,
          important: decls.important,
          keywords: decls.keywords_normal,
          important_keywords: decls.keywords_important,
          media: media_stack.clone(),
        });
      }
      lui_css_old::CssRule::Media(media_rule) => {
        media_stack.push(media_rule.query.clone());
        convert_rules(&media_rule.rules, media_stack, rules);
        media_stack.pop();
      }
      _ => {}
    }
  }
}

/// Parse a single complex selector. Public for test compatibility.
pub fn parse_selector(s: &str) -> ComplexSelector {
  let list = SelectorList::from(s);
  list.selectors.first().cloned().unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a `@import` directive body (the text between `@import` and `;`).
/// Returns `(url, optional_media_query)`.
pub fn parse_import_directive(after_import: &str) -> Option<(&str, Option<&str>)> {
  let s = after_import.trim();
  let (url, rest) = if let Some(inner) = s.strip_prefix("url(") {
    let inner = inner.trim_start();
    if let Some(inner) = inner.strip_prefix('"') {
      let end = inner.find('"')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else if let Some(inner) = inner.strip_prefix('\'') {
      let end = inner.find('\'')?;
      let rest = inner[end + 1..].trim_start().strip_prefix(')')?.trim();
      (&inner[..end], rest)
    } else {
      let end = inner.find(')')?;
      (inner[..end].trim(), inner[end + 1..].trim())
    }
  } else if let Some(inner) = s.strip_prefix('"') {
    let end = inner.find('"')?;
    (&inner[..end], inner[end + 1..].trim())
  } else if let Some(inner) = s.strip_prefix('\'') {
    let end = inner.find('\'')?;
    (&inner[..end], inner[end + 1..].trim())
  } else {
    return None;
  };

  let media = if rest.is_empty() { None } else { Some(rest) };
  Some((url, media))
}
