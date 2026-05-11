use lui_css_parser::{parse_media_query_list, MediaCondition};
use lui_css_parser::ArcStr;

#[test]
fn parses_boolean_feature_with_name_and_no_value() {
    let result = parse_media_query_list("(color)").unwrap();

    let query = &result.0[0];
    let MediaCondition::Feature(feature) = &query.conditions[0] else {
        panic!("expected a feature condition");
    };
    assert_eq!(feature.name, "color");
    assert!(
        feature.value.is_none(),
        "expected no value for boolean feature (color)"
    );
}