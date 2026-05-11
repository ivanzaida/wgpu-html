use crate::CssProperty;
use crate::CssValue;

/// Result of validating a property value.
#[derive(Debug, Clone, PartialEq)]
pub struct Validation {
    pub valid: bool,
    pub warning: Option<String>,
}

impl Validation {
    pub fn ok() -> Self { Validation { valid: true, warning: None } }
    pub fn warn(msg: impl Into<String>) -> Self {
        Validation { valid: false, warning: Some(msg.into()) }
    }
}

/// Check if a value is valid for the given property.
/// Returns `Validation::ok()` if the value appears valid,
/// `Validation::warn()` with a message if it's likely invalid.
pub fn validate_value(property: &CssProperty, value: &CssValue) -> Validation {
    let syntax = property.clone().syntax();
    if syntax.is_empty() { return Validation::ok(); }

    match value {
        CssValue::String(keyword) | CssValue::Unknown(keyword) => {
            if keyword_exists_in_syntax(keyword, syntax) {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` is not a valid keyword for `{}`",
                    keyword, property.name()
                ))
            }
        }

        CssValue::Number(_) => {
            if syntax.contains("<number") || syntax.contains("<integer") {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` does not accept number values",
                    property.name()
                ))
            }
        }

        CssValue::Dimension { unit, .. } => {
            let unit_type = dimension_type(unit);
            if syntax.contains("<length") || syntax.contains("<dimension")
                || syntax.contains(unit_type)
            {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` does not accept dimension values",
                    property.name()
                ))
            }
        }

        CssValue::Percentage(_) => {
            if syntax.contains("<percentage") || syntax.contains("<length-percentage") {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` does not accept percentage values",
                    property.name()
                ))
            }
        }

        CssValue::Color(_) => {
            if syntax.contains("<color") {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` does not accept color values",
                    property.name()
                ))
            }
        }

        CssValue::Function { function, .. } => {
            let name = function.name();
            if syntax.contains(&format!("<{}>", name)) || syntax.contains(&name) {
                Validation::ok()
            } else {
                Validation::warn(format!(
                    "`{}` function is not valid for `{}`",
                    name, property.name()
                ))
            }
        }
    }
}

/// Check if a keyword appears as a complete word in the syntax string.
fn keyword_exists_in_syntax(keyword: &str, syntax: &str) -> bool {
    let lower = syntax.to_lowercase();
    let kw = keyword.to_lowercase();

    // Split syntax on common delimiters
    for word in lower.split(|c: char| c.is_ascii_whitespace() || c == '|' || c == '[' || c == ']' || c == '?' || c == '{' || c == '}' || c == ';') {
        if word.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-') == kw {
            return true;
        }
    }
    false
}

fn dimension_type(unit: &crate::CssUnit) -> &'static str {
    match unit {
        crate::CssUnit::Px | crate::CssUnit::Cm | crate::CssUnit::Mm | crate::CssUnit::Q
        | crate::CssUnit::In | crate::CssUnit::Pt | crate::CssUnit::Pc
        | crate::CssUnit::Em | crate::CssUnit::Rem | crate::CssUnit::Ex | crate::CssUnit::Ch
        | crate::CssUnit::Vw | crate::CssUnit::Vh | crate::CssUnit::Vmin | crate::CssUnit::Vmax
        | crate::CssUnit::Vi | crate::CssUnit::Vb => "<length",
        crate::CssUnit::Deg | crate::CssUnit::Rad | crate::CssUnit::Grad | crate::CssUnit::Turn => "<angle",
        crate::CssUnit::S | crate::CssUnit::Ms => "<time",
        crate::CssUnit::Hz | crate::CssUnit::Khz => "<frequency",
        crate::CssUnit::Dpi | crate::CssUnit::Dpcm | crate::CssUnit::Dppx => "<resolution",
        crate::CssUnit::Fr => "<flex",
        crate::CssUnit::Other(_) => "<dimension",
    }
}
