use lui_live::builder::style::{self, px};

#[test]
fn style_builder_builds_scoped_core_stylesheet() {
  let stylesheet = style::sheet([style::rule(".card:hover .title").padding(px(12))])
    .try_to_core_stylesheet_scoped("demo")
    .unwrap();

  assert_eq!(stylesheet.rules.len(), 1);

  let selector = &stylesheet.rules[0].selector.0[0];
  assert_eq!(selector.compounds[0].classes, ["demo-card"]);
  assert_eq!(selector.compounds[1].classes, ["demo-title"]);
  assert_eq!(stylesheet.rules[0].specificity, (0, 3, 0));
}
