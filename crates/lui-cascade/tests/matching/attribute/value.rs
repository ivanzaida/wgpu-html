use crate::helpers::*;

#[test]
fn eq_matches_exact_value() {
  assert_matches(r#"[type="text"]"#, r#"<input type="text">"#);
}

#[test]
fn eq_rejects_wrong_value() {
  assert_no_match(r#"[type="email"]"#, r#"<input type="text">"#);
}

#[test]
fn contains_matches_substring() {
  assert_matches(r#"[class*="ar"]"#, r#"<div class="card"></div>"#);
}

#[test]
fn contains_rejects_absent_substring() {
  assert_no_match(r#"[class*="xyz"]"#, r#"<div class="card"></div>"#);
}

#[test]
fn starts_with_matches_prefix() {
  assert_matches(r#"[href^="https"]"#, r#"<a href="https://example.com">x</a>"#);
}

#[test]
fn starts_with_rejects_wrong_prefix() {
  assert_no_match(r#"[href^="ftp"]"#, r#"<a href="https://example.com">x</a>"#);
}

#[test]
fn ends_with_matches_suffix() {
  assert_matches(r#"[href$=".pdf"]"#, r#"<a href="doc.pdf">x</a>"#);
}

#[test]
fn ends_with_rejects_wrong_suffix() {
  assert_no_match(r#"[href$=".png"]"#, r#"<a href="doc.pdf">x</a>"#);
}

#[test]
fn includes_matches_space_separated_word() {
  assert_matches(r#"[class~="bar"]"#, r#"<div class="foo bar baz"></div>"#);
}

#[test]
fn includes_rejects_partial_word() {
  assert_no_match(r#"[class~="ba"]"#, r#"<div class="foo bar baz"></div>"#);
}

#[test]
fn hyphen_matches_exact_or_prefix_hyphen() {
  assert_matches(r#"[lang|="en"]"#, r#"<div lang="en-US"></div>"#);
  assert_matches(r#"[lang|="en"]"#, r#"<div lang="en"></div>"#);
  assert_no_match(r#"[lang|="en"]"#, r#"<div lang="fr"></div>"#);
}

#[test]
fn case_insensitive_modifier() {
  assert_matches(r#"[type="TEXT" i]"#, r#"<input type="text">"#);
}
