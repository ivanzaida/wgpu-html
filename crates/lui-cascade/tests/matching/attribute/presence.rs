use crate::helpers::*;

#[test]
fn matches_when_attr_present() {
    assert_matches("[required]", r#"<input required>"#);
}

#[test]
fn rejects_when_attr_absent() {
    assert_no_match("[disabled]", r#"<input required>"#);
}

#[test]
fn matches_data_attr() {
    assert_matches("[data-id]", r#"<div data-id="42"></div>"#);
}

#[test]
fn matches_boolean_attr_without_value() {
    assert_matches("[hidden]", r#"<div hidden></div>"#);
}
