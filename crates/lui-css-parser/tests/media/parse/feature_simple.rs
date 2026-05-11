use lui_css_parser::{parse_media_query_list, MediaCondition};

#[test]
fn parses_single_feature_as_one_query_with_one_condition() {
    let result = parse_media_query_list("(min-width: 600px)").unwrap();

    assert_eq!(result.0.len(), 1, "expected exactly one media query");
    let query = &result.0[0];
    assert_eq!(query.conditions.len(), 1, "expected exactly one condition");
    assert!(
        matches!(&query.conditions[0], MediaCondition::Feature(_)),
        "expected a feature condition"
    );
    assert!(
        query.media_type.is_none(),
        "expected no media type for bare feature query"
    );
    assert!(
        query.modifier.is_none(),
        "expected no modifier for bare feature query"
    );
}
