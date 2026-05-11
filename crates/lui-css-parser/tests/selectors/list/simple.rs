use lui_css_parser::selector::*;

#[test]
fn parses_comma_separated_tag_list() {
    let list = parse_selector_list("h1, h2, h3").unwrap();
    assert_eq!(list.0.len(), 3);
    assert_eq!(list.0[0].compounds[0].tag, Some("h1".to_string()));
    assert_eq!(list.0[1].compounds[0].tag, Some("h2".to_string()));
    assert_eq!(list.0[2].compounds[0].tag, Some("h3".to_string()));
}