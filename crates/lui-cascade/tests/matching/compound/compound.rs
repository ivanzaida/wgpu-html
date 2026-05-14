use crate::helpers::*;

#[test]
fn tag_plus_class() {
  assert_matches("div.card", r#"<div class="card"></div>"#);
  assert_no_match("span.card", r#"<div class="card"></div>"#);
}

#[test]
fn tag_plus_id() {
  assert_matches("div#hero", r#"<div id="hero"></div>"#);
  assert_no_match("span#hero", r#"<div id="hero"></div>"#);
}

#[test]
fn tag_plus_class_plus_id() {
  assert_matches("div.c#x", r#"<div id="x" class="c"></div>"#);
  assert_no_match("span.c#x", r#"<div id="x" class="c"></div>"#);
}

#[test]
fn multiple_classes_plus_tag() {
  assert_matches("div.a.b", r#"<div class="a b"></div>"#);
  assert_no_match("div.a.b", r#"<div class="a"></div>"#);
}
