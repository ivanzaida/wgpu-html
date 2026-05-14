use crate::{ArcStr, color::CssColor, css_function::CssFunction, unit::CssUnit};

#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
  Number(f64),
  Percentage(f64),
  Dimension {
    value: f64,
    unit: CssUnit,
  },
  String(ArcStr),
  Color(CssColor),
  Function {
    function: CssFunction,
    args: Vec<CssValue>,
  },
  Var {
    name: ArcStr,
    fallback: Option<Box<CssValue>>,
  },
  Url(ArcStr),
  Unknown(ArcStr),
}

impl std::hash::Hash for CssValue {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);
    match self {
      CssValue::Number(n) => n.to_bits().hash(state),
      CssValue::Percentage(n) => n.to_bits().hash(state),
      CssValue::Dimension { value, unit } => {
        value.to_bits().hash(state);
        unit.hash(state);
      }
      CssValue::String(s) => s.hash(state),
      CssValue::Color(c) => c.hash(state),
      CssValue::Function { function, args } => {
        function.hash(state);
        args.hash(state);
      }
      CssValue::Var { name, fallback } => {
        name.hash(state);
        fallback.hash(state);
      }
      CssValue::Url(s) => s.hash(state),
      CssValue::Unknown(s) => s.hash(state),
    }
  }
}

impl CssValue {
  pub fn function(name: &str, args: Vec<CssValue>) -> Self {
    CssValue::Function {
      function: CssFunction::from_name(name),
      args,
    }
  }
}
