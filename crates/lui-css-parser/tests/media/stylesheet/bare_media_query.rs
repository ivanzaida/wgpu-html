use lui_css_parser::{parse_stylesheet, CssAtRule, CssUnit, CssValue, MediaCondition};

#[test]
fn populates_media_field_for_max_width_feature_with_nested_rule() {
    let sheet = parse_stylesheet(
        "@media (max-width: 800px) { h1 { color: red; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert!(at.media.is_some(), "expected media field to be populated");

    let mql = at.media.as_ref().unwrap();
    assert_eq!(mql.0.len(), 1);
    let query = &mql.0[0];
    assert_eq!(query.conditions.len(), 1);
    let MediaCondition::Feature(feature) = &query.conditions[0] else {
        panic!("expected a feature condition");
    };
    assert_eq!(feature.name, "max-width");
    assert_eq!(feature.value, Some(CssValue::Dimension { value: 800.0, unit: CssUnit::Px }));

    // Also verify the nested rule is intact
    assert_eq!(at.rules.len(), 1);
}
