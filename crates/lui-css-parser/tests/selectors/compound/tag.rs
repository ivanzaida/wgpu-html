use lui_css_parser::selector::*;

#[test]
fn parses_div_tag_to_tag_some_div() {
    let list = parse_selector_list("div").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, Some("div".to_string()));
    assert!(sel.classes.is_empty());
    assert_eq!(sel.id, None);
    assert!(sel.attrs.is_empty());
    assert!(sel.pseudos.is_empty());
}

#[test]
fn parses_universal_selector_to_tag_some_star() {
    let list = parse_selector_list("*").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, Some("*".to_string()));
}