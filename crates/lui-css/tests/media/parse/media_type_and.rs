use lui_css::{parse_media_query_list, MediaCondition};

#[test]
fn parses_media_type_with_and_feature() {
    let result = parse_media_query_list("screen and (color)").unwrap();

    let query = &result.0[0];
    assert_eq!(query.media_type.as_deref(), Some("screen"));
    assert_eq!(query.conditions.len(), 1);
    let MediaCondition::Feature(feature) = &query.conditions[0] else {
        panic!("expected a feature condition");
    };
    assert_eq!(feature.name, "color");
    assert!(feature.value.is_none());
}
