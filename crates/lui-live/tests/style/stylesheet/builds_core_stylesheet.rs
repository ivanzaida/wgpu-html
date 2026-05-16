use lui_core::{CssProperty, CssUnit, CssValue};
use lui_live::builder::style::{self, px};

#[test]
fn style_builder_builds_typed_core_stylesheet() {
  let stylesheet = style::sheet([style::rule(".card")
    .display("grid")
    .gap(px(8))
    .prop("color", "red !important")])
  .try_to_core_stylesheet()
  .unwrap();

  assert_eq!(stylesheet.rules.len(), 1);

  let declarations = &stylesheet.rules[0].declarations;
  assert_eq!(declarations.len(), 3);

  assert_eq!(declarations[0].property, CssProperty::Display);
  assert_eq!(declarations[0].value, CssValue::String("grid".into()));
  assert!(!declarations[0].important);

  assert_eq!(declarations[1].property, CssProperty::Gap);
  assert_eq!(
    declarations[1].value,
    CssValue::Dimension {
      value: 8.0,
      unit: CssUnit::Px,
    }
  );

  assert_eq!(declarations[2].property, CssProperty::Color);
  assert!(declarations[2].important);
}
