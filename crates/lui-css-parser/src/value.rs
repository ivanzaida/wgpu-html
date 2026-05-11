use crate::ArcStr;
use crate::color::CssColor;
use crate::css_function::CssFunction;
use crate::unit::CssUnit;

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

impl CssValue {
    pub fn function(name: &str, args: Vec<CssValue>) -> Self {
        CssValue::Function { function: CssFunction::from_name(name), args }
    }
}
