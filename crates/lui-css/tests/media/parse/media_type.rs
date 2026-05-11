use lui_css::parse_media_query_list;

#[test]
fn parses_media_type_without_features_or_modifier() {
    let result = parse_media_query_list("screen").unwrap();

    assert_eq!(result.0.len(), 1, "expected exactly one media query");
    let query = &result.0[0];
    assert_eq!(query.media_type.as_deref(), Some("screen"));
    assert!(
        query.conditions.is_empty(),
        "expected no conditions for media-type-only query"
    );
    assert!(
        query.modifier.is_none(),
        "expected no modifier for media-type-only query"
    );
}
