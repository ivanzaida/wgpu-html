use crate::{
  at_rule::AtRuleParser,
  stylesheet::{CssRule, SupportsRule},
  values::ArcStr,
};

pub struct SupportsAtRuleParser;

impl AtRuleParser for SupportsAtRuleParser {
  fn name(&self) -> &'static str {
    "supports"
  }

  fn parse_block(&self, prelude: &str, block: &str, parse_nested: &dyn Fn(&str) -> Vec<CssRule>) -> Option<CssRule> {
    let condition = prelude.trim();
    if condition.is_empty() {
      return None;
    }
    let rules = parse_nested(block);
    Some(CssRule::Supports(SupportsRule {
      condition: ArcStr::from(condition),
      rules,
    }))
  }
}
