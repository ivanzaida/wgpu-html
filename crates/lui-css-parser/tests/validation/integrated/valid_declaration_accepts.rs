use lui_css_parser::{parse_declaration, validate_value, Validation};
use lui_css_parser::ArcStr;

#[test]
fn parse_and_validate_valid_color_declaration() {
    let (property, value) = parse_declaration("color", "red").unwrap();
    let result = validate_value(&property, &value);
    assert_eq!(result, Validation { valid: true, warning: None });
}