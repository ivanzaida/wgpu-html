use lui_css_old::{syntax::parse_raw_declarations, Importance};

#[test]
fn parses_normal_declaration() {
  let block = parse_raw_declarations("color: red;");
  assert_eq!(block.len(), 1);
  assert_eq!(&*block.declarations[0].property, "color");
  assert_eq!(&*block.declarations[0].value, "red");
  assert_eq!(block.declarations[0].importance, Importance::Normal);
}

#[test]
fn parses_important_declaration() {
  let block = parse_raw_declarations("color: red !important;");
  assert_eq!(block.declarations[0].importance, Importance::Important);
  assert_eq!(&*block.declarations[0].value, "red");
}

#[test]
fn parses_important_with_extra_whitespace() {
  let block = parse_raw_declarations("color: red !  important;");
  assert_eq!(block.declarations[0].importance, Importance::Important);
}

#[test]
fn parses_multiple_declarations() {
  let block = parse_raw_declarations("color: red; font-size: 12px; margin: 0;");
  assert_eq!(block.len(), 3);
}

#[test]
fn lowercases_property_names() {
  let block = parse_raw_declarations("COLOR: red; Font-Size: 12px;");
  assert_eq!(&*block.declarations[0].property, "color");
  assert_eq!(&*block.declarations[1].property, "font-size");
}

#[test]
fn preserves_custom_property_case() {
  let block = parse_raw_declarations("--MyColor: blue;");
  assert_eq!(&*block.declarations[0].property, "--MyColor");
}

#[test]
fn handles_trailing_semicolons_and_whitespace() {
  let block = parse_raw_declarations("  color: red ;  ;  ");
  assert_eq!(block.len(), 1);
}

#[test]
fn handles_no_semicolons() {
  let block = parse_raw_declarations("color: red");
  assert_eq!(block.len(), 1);
  assert_eq!(&*block.declarations[0].value, "red");
}

#[test]
fn handles_empty_input() {
  let block = parse_raw_declarations("");
  assert!(block.is_empty());
}

#[test]
fn handles_only_semicolons() {
  let block = parse_raw_declarations(";;;");
  assert!(block.is_empty());
}

#[test]
fn preserves_complex_values() {
  let block = parse_raw_declarations("background: linear-gradient(to right, #ff0, rgba(0,0,0,0.5));");
  assert_eq!(block.len(), 1);
  assert!(block.declarations[0].value.contains("linear-gradient"));
}

#[test]
fn preserves_value_with_colons() {
  let block = parse_raw_declarations("background-image: url(http://example.com/img.png);");
  assert_eq!(block.len(), 1);
  assert!(block.declarations[0].value.contains("http://example.com"));
}

#[test]
fn mixed_importance() {
  let block = parse_raw_declarations("color: red !important; font-size: 12px; margin: 0 !important;");
  assert_eq!(block.declarations[0].importance, Importance::Important);
  assert_eq!(block.declarations[1].importance, Importance::Normal);
  assert_eq!(block.declarations[2].importance, Importance::Important);
}

#[test]
fn value_with_var_reference() {
  let block = parse_raw_declarations("color: var(--theme-color, #333);");
  assert_eq!(block.len(), 1);
  assert!(block.declarations[0].value.contains("var(--theme-color"));
}
