use lui_css_parser::selector::*;
use lui_css_parser::CssPseudo;
use lui_css_parser::ArcStr;

#[test]
fn parses_hover_pseudo_class() {
    let list = parse_selector_list(":hover").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.pseudos.len(), 1);
    assert_eq!(sel.pseudos[0].pseudo, CssPseudo::Hover);
    assert_eq!(sel.pseudos[0].arg, None);
}

#[test]
fn parses_before_pseudo_element() {
    let list = parse_selector_list("::before").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.pseudos.len(), 1);
    assert_eq!(sel.pseudos[0].pseudo, CssPseudo::Before);
    assert_eq!(sel.pseudos[0].arg, None);
}

#[test]
fn parses_nth_child_functional_pseudo_with_arg() {
    let list = parse_selector_list(":nth-child(2n+1)").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.pseudos.len(), 1);
    assert_eq!(sel.pseudos[0].arg, Some("2n+1".to_string()));
}