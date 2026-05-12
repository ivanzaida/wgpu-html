use crate::values::ArcStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Importance {
  Normal,
  Important,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CssWideKeyword {
  Inherit,
  Initial,
  Unset,
}

impl CssWideKeyword {
  pub fn from_value(v: &str) -> Option<Self> {
    let trimmed = v.trim();
    if trimmed.eq_ignore_ascii_case("inherit") {
      Some(Self::Inherit)
    } else if trimmed.eq_ignore_ascii_case("initial") {
      Some(Self::Initial)
    } else if trimmed.eq_ignore_ascii_case("unset") {
      Some(Self::Unset)
    } else {
      None
    }
  }
}

#[derive(Debug, Clone)]
pub struct Declaration {
  pub property: ArcStr,
  pub value: ArcStr,
  pub importance: Importance,
}

#[derive(Debug, Clone, Default)]
pub struct DeclarationBlock {
  pub declarations: Vec<Declaration>,
}

impl DeclarationBlock {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn push(&mut self, property: impl Into<ArcStr>, value: impl Into<ArcStr>, importance: Importance) {
    self.declarations.push(Declaration {
      property: property.into(),
      value: value.into(),
      importance,
    });
  }

  pub fn is_empty(&self) -> bool {
    self.declarations.is_empty()
  }

  pub fn len(&self) -> usize {
    self.declarations.len()
  }
}
