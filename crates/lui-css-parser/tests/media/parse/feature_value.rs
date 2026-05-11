use lui_css_parser::{parse_media_query_list, CssUnit, CssValue, MediaCondition};

#[test]
fn parses_feature_name_and_value_correctly() {
    let result = parse_media_query_list("(min-width: 600px)").unwrap();

    let query = &result.0[0];
    let MediaCondition::Feature(feature) = &query.conditions[0] else {
        panic!("expected a feature condition");
    };
    assert_eq!(feature.name, "min-width");
    assert_eq!(feature.value, Some(CssValue::Dimension { value: 600.0, unit: CssUnit::Px }));
}
