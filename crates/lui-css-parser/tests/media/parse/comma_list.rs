use lui_css_parser::parse_media_query_list;

#[test]
fn parses_comma_separated_queries_into_two_separate_queries() {
    let result = parse_media_query_list("screen, print").unwrap();

    assert_eq!(result.0.len(), 2, "expected exactly two media queries");
    assert_eq!(result.0[0].media_type.as_deref(), Some("screen"));
    assert_eq!(result.0[1].media_type.as_deref(), Some("print"));
}
