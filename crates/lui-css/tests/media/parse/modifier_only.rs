use lui_css::{parse_media_query_list, MediaModifier};

#[test]
fn parses_only_modifier_with_media_type() {
    let result = parse_media_query_list("only print").unwrap();

    let query = &result.0[0];
    assert_eq!(query.modifier, Some(MediaModifier::Only));
    assert_eq!(query.media_type.as_deref(), Some("print"));
    assert!(
        query.conditions.is_empty(),
        "expected no conditions for `only print`"
    );
}
