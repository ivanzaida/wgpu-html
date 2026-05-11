use crate::css_function::CssFunction;

#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    Number(f64),
    Percentage(f64),
    Dimension {
        value: f64,
        unit: String,
    },
    String(String),
    Function {
        function: CssFunction,
        args: Vec<CssValue>,
    },
}

impl CssValue {
    pub fn function(name: &str, args: Vec<CssValue>) -> Option<Self> {
        CssFunction::from_name(name).map(|f| CssValue::Function { function: f, args })
    }
}
