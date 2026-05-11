use lui_css_parser::{parse_media_query_list, MediaModifier};

#[test]
fn parses_not_modifier_with_media_type() {
    let result = parse_media_query_list("not screen").unwrap();

    let query = &result.0[0];
    assert_eq!(query.modifier, Some(MediaModifier::Not));
    assert_eq!(query.media_type.as_deref(), Some("screen"));
    assert!(
        query.conditions.is_empty(),
        "expected no conditions for `not screen`"
    );
}
