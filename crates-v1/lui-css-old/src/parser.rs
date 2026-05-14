use crate::{
  at_rule::AtRuleRegistry,
  declaration::DeclarationBlock,
  stylesheet::{CssRule, StyleRule, Stylesheet, UnknownAtRule},
  syntax::{self, parse_raw_declarations, RawRule},
  values::ArcStr,
  warn_once,
};

pub struct CssParser {
  registry: AtRuleRegistry,
}

impl CssParser {
  pub fn new() -> Self {
    Self {
      registry: AtRuleRegistry::with_builtins(),
    }
  }

  pub fn with_registry(registry: AtRuleRegistry) -> Self {
    Self { registry }
  }

  pub fn parse_stylesheet(&self, input: &str) -> Stylesheet {
    let rules = self.parse_rule_list(input);
    Stylesheet { rules }
  }

  pub fn parse_rule_list(&self, input: &str) -> Vec<CssRule> {
    let raw_rules = syntax::parse_raw_rules(input);
    let mut rules = Vec::new();

    for raw in raw_rules {
      match raw {
        RawRule::QualifiedRule(qr) => {
          let declarations = parse_raw_declarations(&qr.block);
          if !qr.prelude.is_empty() {
            rules.push(CssRule::Style(StyleRule {
              selector_text: ArcStr::from(qr.prelude),
              declarations,
            }));
          }
        }
        RawRule::AtRule(ar) => {
          let rule = if let Some(ref block) = ar.block {
            self
              .registry
              .try_parse_block(&ar.name, &ar.prelude, block, &|nested| self.parse_rule_list(nested))
          } else {
            self.registry.try_parse_statement(&ar.name, &ar.prelude)
          };

          let rule = rule.unwrap_or_else(|| {
            warn_once!("unsupported at-rule: `@{}`", ar.name);
            CssRule::Unknown(UnknownAtRule {
              name: ArcStr::from(ar.name),
              prelude: ArcStr::from(ar.prelude),
              block: ar.block.map(|b| ArcStr::from(b)),
            })
          });

          rules.push(rule);
        }
      }
    }

    rules
  }

  pub fn parse_declarations(&self, input: &str) -> DeclarationBlock {
    parse_raw_declarations(input)
  }
}

impl Default for CssParser {
  fn default() -> Self {
    Self::new()
  }
}
