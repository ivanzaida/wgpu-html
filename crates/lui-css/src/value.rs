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
    String(String),
    Function {
        function: CssFunction,
        args: Vec<CssValue>,
    },
    Unknown(String),
}

impl CssValue {
    pub fn function(name: &str, args: Vec<CssValue>) -> Option<Self> {
        CssFunction::from_name(name).map(|f| CssValue::Function { function: f, args })
    }
}
