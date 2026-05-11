use lui_css_parser::{validate_value, CssFunction, CssProperty, CssValue, Validation};
use lui_css_parser::ArcStr;

/// Width syntax contains `calc-size()` which has "calc" as a substring,
/// so the `calc` function name is matched by `syntax.contains(&name)`.
#[test]
fn calc_function_is_valid_for_width() {
    let result = validate_value(
        &CssProperty::Width,
        &CssValue::Function {
            function: CssFunction::Calc,
            args: vec![],
        },
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}