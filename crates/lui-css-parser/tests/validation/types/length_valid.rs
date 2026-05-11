use lui_css_parser::{validate_value, CssProperty, CssUnit, CssValue, Validation};
use lui_css_parser::ArcStr;

#[test]
fn dimension_in_px_is_valid_for_font_size() {
    let result = validate_value(
        &CssProperty::FontSize,
        &CssValue::Dimension {
            value: 14.0,
            unit: CssUnit::Px,
        },
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dimension_in_px_is_valid_for_margin_top() {
    let result = validate_value(
        &CssProperty::MarginTop,
        &CssValue::Dimension {
            value: 14.0,
            unit: CssUnit::Px,
        },
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dimension_in_px_is_valid_for_width() {
    let result = validate_value(
        &CssProperty::Width,
        &CssValue::Dimension {
            value: 14.0,
            unit: CssUnit::Px,
        },
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}

#[test]
fn dimension_in_px_is_valid_for_height() {
    let result = validate_value(
        &CssProperty::Height,
        &CssValue::Dimension {
            value: 14.0,
            unit: CssUnit::Px,
        },
    );
    assert_eq!(result, Validation { valid: true, warning: None });
}