use lui_css_parser::selector::*;
use lui_css_parser::CssPseudo;
use lui_css_parser::ArcStr;

#[test]
fn parses_nesting_selector_ampersand_with_class() {
    let list = parse_selector_list("&.active").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, None);
    assert_eq!(sel.classes, vec!["active".to_string()]);
    assert_eq!(sel.pseudos.len(), 1);
    assert_eq!(sel.pseudos[0].pseudo, CssPseudo::Ampersand);
    assert_eq!(sel.pseudos[0].arg, None);
}