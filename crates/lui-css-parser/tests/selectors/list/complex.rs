use lui_css_parser::selector::*;
use lui_css_parser::CssCombinator;
use lui_css_parser::ArcStr;

#[test]
fn parses_complex_selectors_in_comma_separated_list() {
    let list = parse_selector_list("h1 > a, h2 > a").unwrap();
    assert_eq!(list.0.len(), 2);
    assert_eq!(list.0[0].compounds.len(), 2);
    assert_eq!(list.0[0].compounds[0].tag, Some("h1".to_string()));
    assert_eq!(list.0[0].compounds[1].tag, Some("a".to_string()));
    assert_eq!(list.0[0].combinators[0], CssCombinator::Child);
    assert_eq!(list.0[1].compounds.len(), 2);
    assert_eq!(list.0[1].compounds[0].tag, Some("h2".to_string()));
    assert_eq!(list.0[1].compounds[1].tag, Some("a".to_string()));
    assert_eq!(list.0[1].combinators[0], CssCombinator::Child);
}