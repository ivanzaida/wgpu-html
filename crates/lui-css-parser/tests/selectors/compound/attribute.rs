use lui_css_parser::selector::*;

#[test]
fn parses_attribute_presence_only() {
    let list = parse_selector_list("[attr]").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.attrs.len(), 1);
    assert_eq!(sel.attrs[0].name, "attr");
    assert_eq!(sel.attrs[0].op, None);
    assert_eq!(sel.attrs[0].value, None);
    assert_eq!(sel.attrs[0].modifier, None);
}

#[test]
fn parses_attribute_equals_value() {
    let list = parse_selector_list("[attr=val]").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.attrs.len(), 1);
    assert_eq!(sel.attrs[0].name, "attr");
    assert_eq!(sel.attrs[0].op, Some(AttrOp::Eq));
    assert_eq!(sel.attrs[0].value, Some("val".to_string()));
    assert_eq!(sel.attrs[0].modifier, None);
}

#[test]
fn parses_attribute_starts_with() {
    let list = parse_selector_list("[attr^=val]").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.attrs.len(), 1);
    assert_eq!(sel.attrs[0].name, "attr");
    assert_eq!(sel.attrs[0].op, Some(AttrOp::StartsWith));
    assert_eq!(sel.attrs[0].value, Some("val".to_string()));
}

#[test]
fn parses_attribute_includes_with_case_modifier() {
    let list = parse_selector_list("[attr~=val i]").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.attrs.len(), 1);
    assert_eq!(sel.attrs[0].name, "attr");
    assert_eq!(sel.attrs[0].op, Some(AttrOp::Includes));
    assert_eq!(sel.attrs[0].value, Some("val".to_string()));
    assert_eq!(sel.attrs[0].modifier, Some('i'));
}
