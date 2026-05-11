use lui_css_parser::selector::*;
use lui_css_parser::CssCombinator;
use lui_css_parser::ArcStr;

#[test]
fn parses_child_combinator() {
    let list = parse_selector_list("div > span").unwrap();
    let sel = &list.0[0];
    assert_eq!(sel.compounds.len(), 2);
    assert_eq!(sel.compounds[0].tag, Some("div".to_string()));
    assert_eq!(sel.compounds[1].tag, Some("span".to_string()));
    assert_eq!(sel.combinators.len(), 1);
    assert_eq!(sel.combinators[0], CssCombinator::Child);
}