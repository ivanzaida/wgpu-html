use crate::helpers::*;

#[test]
fn matches_single_class() {
  assert_matches(".foo", r#"<div class="foo"></div>"#);
}

#[test]
fn matches_among_multiple_classes() {
  assert_matches(".bar", r#"<div class="foo bar baz"></div>"#);
}

#[test]
fn rejects_absent_class() {
  assert_no_match(".missing", r#"<div class="foo bar"></div>"#);
}

#[test]
fn rejects_when_no_class_attr() {
  assert_no_match(".any", "<div></div>");
}

#[test]
fn requires_all_classes_in_compound() {
  assert_matches(".a.b", r#"<div class="a b c"></div>"#);
  assert_no_match(".a.b", r#"<div class="a c"></div>"#);
}

#[test]
fn class_is_case_sensitive() {
  assert_no_match(".Foo", r#"<div class="foo"></div>"#);
}
