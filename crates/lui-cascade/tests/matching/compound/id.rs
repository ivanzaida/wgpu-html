use crate::helpers::*;

#[test]
fn matches_id() {
    assert_matches("#main", r#"<div id="main"></div>"#);
}

#[test]
fn rejects_wrong_id() {
    assert_matches("#main", r#"<div id="main"></div>"#);
    assert_no_match("#other", r#"<div id="main"></div>"#);
}

#[test]
fn rejects_when_no_id() {
    assert_no_match("#x", "<div></div>");
}

#[test]
fn id_is_case_sensitive() {
    assert_no_match("#Main", r#"<div id="main"></div>"#);
}
