use lui_css_parser::{parse_declaration, validate_value};
use lui_css_parser::ArcStr;

#[test]
fn parse_and_validate_invalid_display_keyword_warns() {
    let (property, value) = parse_declaration("display", "nope").unwrap();
    let result = validate_value(&property, &value);
    assert_eq!(result.valid, false);
    assert!(result.warning.is_some());
    let warning = result.warning.unwrap();
    assert!(warning.contains("nope"));
    assert!(warning.contains("display"));
}