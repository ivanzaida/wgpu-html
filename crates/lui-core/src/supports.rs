/// A parsed `@supports` condition.
#[derive(Debug, Clone, PartialEq)]
pub enum SupportsCondition {
  Feature(SupportsFeature),
  Not(Box<SupportsCondition>),
  And(Vec<SupportsCondition>),
  Or(Vec<SupportsCondition>),
}

/// A single feature test like `(display: grid)` or `selector(.foo)`.
#[derive(Debug, Clone, PartialEq)]
pub struct SupportsFeature {
  pub name: String,
  pub value: Option<String>,
  pub is_selector: bool,
}
