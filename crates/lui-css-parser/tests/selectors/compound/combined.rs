use lui_css_parser::selector::*;
use lui_css_parser::CssPseudo;
use lui_css_parser::ArcStr;

#[test]
fn parses_combined_tag_class_id_pseudo_in_one_compound() {
    let list = parse_selector_list("div.foo#bar:hover").unwrap();
    let sel = &list.0[0].compounds[0];
    assert_eq!(sel.tag, Some("div".to_string()));
    assert_eq!(sel.classes, vec!["foo".to_string()]);
    assert_eq!(sel.id, Some("bar".to_string()));
    assert_eq!(sel.pseudos.len(), 1);
    assert_eq!(sel.pseudos[0].pseudo, CssPseudo::Hover);
    assert!(sel.attrs.is_empty());
}