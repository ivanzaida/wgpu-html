use lui_css_parser::selector::*;
use lui_css_parser::CssCombinator;
use lui_css_parser::ArcStr;

#[test]
fn parses_mixed_child_and_next_sibling_combinators() {
    let list = parse_selector_list("div.foo > span.bar + a").unwrap();
    let sel = &list.0[0];
    assert_eq!(sel.compounds.len(), 3);
    assert_eq!(sel.compounds[0].tag, Some("div".to_string()));
    assert_eq!(sel.compounds[0].classes, vec!["foo".to_string()]);
    assert_eq!(sel.compounds[1].tag, Some("span".to_string()));
    assert_eq!(sel.compounds[1].classes, vec!["bar".to_string()]);
    assert_eq!(sel.compounds[2].tag, Some("a".to_string()));
    assert_eq!(sel.combinators.len(), 2);
    assert_eq!(sel.combinators[0], CssCombinator::Child);
    assert_eq!(sel.combinators[1], CssCombinator::NextSibling);
}