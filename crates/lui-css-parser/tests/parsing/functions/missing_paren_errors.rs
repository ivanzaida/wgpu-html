use lui_css_parser::parse_value;
use lui_css_parser::ArcStr;

#[test]
fn returns_error_when_missing_closing_parenthesis() {
    assert!(parse_value("acos(1").is_err());
}

#[test]
fn returns_error_when_missing_opening_parenthesis() {
    assert!(parse_value("acos 1)").is_err());
}