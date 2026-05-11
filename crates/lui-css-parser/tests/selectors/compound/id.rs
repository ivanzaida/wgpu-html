use lui_css_parser::selector::*;

#[test]
fn parses_id_selector() {
    let list = parse_selector_list("#bar").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, None);
    assert!(sel.classes.is_empty());
    assert_eq!(sel.id, Some("bar".to_string()));
}