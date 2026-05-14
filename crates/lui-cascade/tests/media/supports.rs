use lui_cascade::media::evaluate_supports;
use lui_parse::parse_supports_condition;

#[test]
fn supports_known_property() {
  let cond = parse_supports_condition("(display: grid)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn rejects_unknown_property() {
  let cond = parse_supports_condition("(foo-bar: baz)").unwrap();
  assert!(!evaluate_supports(&cond));
}

#[test]
fn supports_not() {
  let cond = parse_supports_condition("not (foo-bar: baz)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_and() {
  let cond = parse_supports_condition("(display: grid) and (color: red)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_and_fails_if_one_unknown() {
  let cond = parse_supports_condition("(display: grid) and (fake-prop: x)").unwrap();
  assert!(!evaluate_supports(&cond));
}

#[test]
fn supports_or() {
  let cond = parse_supports_condition("(fake-prop: x) or (display: block)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_selector() {
  let cond = parse_supports_condition("selector(.foo > .bar)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_selector_in_parens() {
  let cond = parse_supports_condition("(selector(.foo > .bar))").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_selector_or_property() {
  let cond = parse_supports_condition("selector(.foo) or (display: grid)").unwrap();
  assert!(evaluate_supports(&cond));
}

#[test]
fn supports_not_selector() {
  let cond = parse_supports_condition("not selector(.foo)").unwrap();
  assert!(!evaluate_supports(&cond));
}
