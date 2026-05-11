use lui_css_parser::parse_value;
use lui_css_parser::ArcStr;

#[test]
fn returns_error_for_text_after_a_complete_value() {
    assert!(parse_value("acos(1) blah").is_err());
}