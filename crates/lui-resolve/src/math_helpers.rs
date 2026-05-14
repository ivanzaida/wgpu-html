//! Shared types and helpers for math and unit resolution.

use lui_core::{CssUnit, CssValue};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MathValue {
  Number(f64),
  Dimension(f64, CssUnit),
  Percentage(f64),
}

impl MathValue {
  pub fn to_css_value(&self) -> CssValue {
    match self {
      MathValue::Number(n) => CssValue::Number(*n),
      MathValue::Percentage(n) => CssValue::Percentage(*n),
      MathValue::Dimension(v, u) => CssValue::Dimension {
        value: *v,
        unit: u.clone(),
      },
    }
  }
  pub fn negate(self) -> Self {
    match self {
      MathValue::Number(n) => MathValue::Number(-n),
      MathValue::Percentage(n) => MathValue::Percentage(-n),
      MathValue::Dimension(v, u) => MathValue::Dimension(-v, u),
    }
  }
  pub fn add(self, other: Self) -> Self {
    use MathValue::*;
    match (self, other) {
      (Number(a), Number(b)) => Number(a + b),
      (Dimension(av, au), Dimension(bv, bu)) if au == bu => Dimension(av + bv, au),
      (Percentage(a), Percentage(b)) => Percentage(a + b),
      (Dimension(v, u), Number(n)) | (Number(n), Dimension(v, u)) => Dimension(v + n, u),
      (Percentage(p), Number(n)) | (Number(n), Percentage(p)) => Percentage(p + n),
      _ => Number(0.0),
    }
  }
  pub fn as_f64(&self) -> f64 {
    match self {
      MathValue::Number(n) | MathValue::Percentage(n) | MathValue::Dimension(n, _) => *n,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ResolvedNumber {
  Number(f64),
  Px(f64),
  Percentage(f64),
}

impl ResolvedNumber {
  pub fn to_css_value(&self) -> CssValue {
    match self {
      ResolvedNumber::Number(n) => CssValue::Number(*n),
      ResolvedNumber::Px(n) => CssValue::Dimension {
        value: *n,
        unit: CssUnit::Px,
      },
      ResolvedNumber::Percentage(n) => CssValue::Percentage(*n),
    }
  }
  pub fn negate(self) -> Self {
    match self {
      ResolvedNumber::Number(n) => ResolvedNumber::Number(-n),
      ResolvedNumber::Px(n) => ResolvedNumber::Px(-n),
      ResolvedNumber::Percentage(n) => ResolvedNumber::Percentage(-n),
    }
  }
  pub fn add(self, other: Self) -> Self {
    use ResolvedNumber::*;
    match (self, other) {
      (Number(a), Number(b)) => Number(a + b),
      (Px(a), Px(b)) => Px(a + b),
      (Percentage(a), Percentage(b)) => Percentage(a + b),
      (Px(a), Number(b)) | (Number(b), Px(a)) => Px(a + b),
      (Percentage(a), Number(b)) | (Number(b), Percentage(a)) => Percentage(a + b),
      _ => Number(0.0),
    }
  }
  pub fn partial_cmp_rn(&self, other: &Self) -> std::cmp::Ordering {
    f64::total_cmp(&self.as_f64(), &other.as_f64())
  }
  pub(crate) fn as_f64(&self) -> f64 {
    match self {
      ResolvedNumber::Number(n) | ResolvedNumber::Px(n) | ResolvedNumber::Percentage(n) => *n,
    }
  }
}

impl PartialOrd for ResolvedNumber {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(f64::total_cmp(&self.as_f64(), &other.as_f64()))
  }
}

pub(crate) fn _to_math_value(v: &CssValue) -> MathValue {
  match v {
    CssValue::Number(n) => MathValue::Number(*n),
    CssValue::Percentage(n) => MathValue::Percentage(*n),
    CssValue::Dimension { value, unit } => MathValue::Dimension(*value, unit.clone()),
    _ => MathValue::Number(0.0),
  }
}

pub(crate) fn to_f64(v: &CssValue) -> f64 {
  match v {
    CssValue::Number(n) | CssValue::Percentage(n) => *n,
    CssValue::Dimension { value, .. } => *value,
    _ => 0.0,
  }
}
