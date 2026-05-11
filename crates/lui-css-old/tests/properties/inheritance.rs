use lui_css_old::{properties::is_inherited, PropertyId};

#[test]
fn inherited_text_properties() {
  assert!(is_inherited("color"));
  assert!(is_inherited("font-family"));
  assert!(is_inherited("font-size"));
  assert!(is_inherited("font-weight"));
  assert!(is_inherited("font-style"));
  assert!(is_inherited("line-height"));
  assert!(is_inherited("letter-spacing"));
  assert!(is_inherited("text-align"));
  assert!(is_inherited("text-transform"));
  assert!(is_inherited("white-space"));
  assert!(is_inherited("word-break"));
  assert!(is_inherited("visibility"));
}

#[test]
fn inherited_svg_properties() {
  assert!(is_inherited("fill"));
  assert!(is_inherited("fill-opacity"));
  assert!(is_inherited("fill-rule"));
  assert!(is_inherited("stroke"));
  assert!(is_inherited("stroke-width"));
  assert!(is_inherited("stroke-opacity"));
  assert!(is_inherited("stroke-linecap"));
  assert!(is_inherited("stroke-linejoin"));
  assert!(is_inherited("stroke-dasharray"));
  assert!(is_inherited("stroke-dashoffset"));
}

#[test]
fn inherited_interaction_properties() {
  assert!(is_inherited("cursor"));
  assert!(is_inherited("pointer-events"));
  assert!(is_inherited("user-select"));
}

#[test]
fn non_inherited_layout_properties() {
  assert!(!is_inherited("display"));
  assert!(!is_inherited("position"));
  assert!(!is_inherited("width"));
  assert!(!is_inherited("height"));
  assert!(!is_inherited("margin-top"));
  assert!(!is_inherited("padding-left"));
  assert!(!is_inherited("flex-direction"));
  assert!(!is_inherited("grid-template-columns"));
  assert!(!is_inherited("z-index"));
  assert!(!is_inherited("opacity"));
}

#[test]
fn non_inherited_border_properties() {
  assert!(!is_inherited("border"));
  assert!(!is_inherited("border-top-width"));
  assert!(!is_inherited("border-radius"));
  assert!(!is_inherited("background-color"));
}

#[test]
fn property_id_is_inherited_matches_function() {
  let id = PropertyId::from_name("color").unwrap();
  assert!(id.is_inherited());

  let id = PropertyId::from_name("display").unwrap();
  assert!(!id.is_inherited());
}
