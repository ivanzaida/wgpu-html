use lui_css_parser::selector::*;

#[test]
fn parses_single_class() {
    let list = parse_selector_list(".foo").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, None);
    assert_eq!(sel.classes, vec!["foo".to_string()]);
}

#[test]
fn parses_two_classes_chained() {
    let list = parse_selector_list(".a.b").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, None);
    assert_eq!(sel.classes, vec!["a".to_string(), "b".to_string()]);
}