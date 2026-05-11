use lui_css_parser::parse_declaration;
use lui_css_parser::ArcStr;

#[test]
fn returns_error_for_malformed_value_on_valid_property() {
    let result = parse_declaration("color", ")))");
    assert!(result.is_err(), "expected parse_declaration to fail for malformed value");
}