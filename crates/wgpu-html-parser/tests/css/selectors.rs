use wgpu_html_parser::{stylesheet::parse_selector, Combinator, PseudoClass};

#[test]
fn parses_tag_selector() {
    let s = parse_selector("div");
    assert_eq!(s.subject().tag.as_deref(), Some("div"));
}

#[test]
fn parses_id_selector() {
    let s = parse_selector("#hero");
    assert_eq!(s.subject().id.as_deref(), Some("hero"));
}

#[test]
fn parses_class_selector() {
    let s = parse_selector(".card");
    assert_eq!(s.subject().classes, vec!["card"]);
}

#[test]
fn parses_compound_selector() {
    let s = parse_selector("div#hero.card.big");
    assert_eq!(s.subject().tag.as_deref(), Some("div"));
    assert_eq!(s.subject().id.as_deref(), Some("hero"));
    assert!(s.subject().classes.contains(&"card".to_owned()));
    assert!(s.subject().classes.contains(&"big".to_owned()));
}

#[test]
fn universal_keeps_tag_none() {
    let s = parse_selector("*");
    assert!(s.subject().tag.is_none());
    assert!(s.subject().id.is_none());
}

#[test]
fn parses_descendant_combinator() {
    let s = parse_selector("div p");
    let subj = s.subject();
    let anc = s.ancestor_compounds();
    assert_eq!(subj.tag.as_deref(), Some("p"));
    assert_eq!(anc.len(), 1);
    assert_eq!(anc[0].tag.as_deref(), Some("div"));
    assert_eq!(s.combinators.len(), 1);
    assert_eq!(s.combinators[0], Combinator::Descendant);
}

#[test]
fn parses_three_level_descendant_chain() {
    let s = parse_selector(".a .b .c");
    let anc = s.ancestor_compounds();
    assert_eq!(anc.len(), 2);
    assert!(anc[0].classes.contains(&"a".to_owned()));
    assert!(anc[1].classes.contains(&"b".to_owned()));
    assert!(s.subject().classes.contains(&"c".to_owned()));
}

#[test]
fn descendant_specificity_sums_compounds() {
    let two = parse_selector(".a .b").specificity();
    let one = parse_selector(".b").specificity();
    assert!(two > one, "descendant chain should have higher specificity");
}

#[test]
fn all_combinators_now_supported() {
    let gt = parse_selector("div > p");
    assert_eq!(gt.combinators.len(), 1);
    assert_eq!(gt.combinators[0], Combinator::Child);
    let plus = parse_selector("div + p");
    assert_eq!(plus.combinators[0], Combinator::NextSibling);
    let tilde = parse_selector("div ~ p");
    assert_eq!(tilde.combinators[0], Combinator::SubsequentSibling);
}

#[test]
fn nth_child_now_supported() {
    let s = parse_selector("li:nth-child(2)");
    assert_eq!(s.subject().tag.as_deref(), Some("li"));
    assert!(!s.subject().pseudo_classes.is_empty());
}

#[test]
fn parses_hover_pseudo_class() {
    let s = parse_selector("a:hover");
    assert_eq!(s.subject().tag.as_deref(), Some("a"));
    assert!(s.subject().pseudo_classes.iter().any(|pc| matches!(pc, PseudoClass::Hover)));
}

#[test]
fn parses_bare_hover_pseudo_class() {
    let s = parse_selector(":hover");
    assert!(s.subject().pseudo_classes.iter().any(|pc| matches!(pc, PseudoClass::Hover)));
}

#[test]
fn parses_focus_pseudo_class() {
    let focus = parse_selector(":focus");
    assert!(focus.subject().pseudo_classes.iter().any(|pc| matches!(pc, PseudoClass::Focus)));
}

#[test]
fn parses_attribute_presence_selector() {
    let s = parse_selector("abbr[title]");
    assert_eq!(s.subject().tag.as_deref(), Some("abbr"));
    assert!(!s.subject().attrs.is_empty());
    assert_eq!(s.subject().attrs[0].name, "title");
}

#[test]
fn parses_attribute_equality_selector() {
    let s = parse_selector(r#"input[type="submit"]"#);
    assert_eq!(s.subject().tag.as_deref(), Some("input"));
    assert!(!s.subject().attrs.is_empty());
    assert_eq!(s.subject().attrs[0].name, "type");
    assert_eq!(s.subject().attrs[0].value, "submit");
}

#[test]
fn attribute_selector_adds_class_specificity() {
    let plain = parse_selector("input").specificity();
    let attr = parse_selector("input[type=submit]").specificity();
    assert!(attr > plain);
}

#[test]
fn parses_compound_selector_into_hover_active() {
    let s = parse_selector("button#go.primary:hover:active");
    assert_eq!(s.subject().tag.as_deref(), Some("button"));
    assert_eq!(s.subject().id.as_deref(), Some("go"));
    assert!(s.subject().classes.contains(&"primary".to_owned()));
    assert!(s.subject().pseudo_classes.iter().any(|pc| matches!(pc, PseudoClass::Hover)));
    assert!(s.subject().pseudo_classes.iter().any(|pc| matches!(pc, PseudoClass::Active)));
}

#[test]
fn compound_specificity_adds_up() {
    let plain = parse_selector("a").specificity();
    let hover = parse_selector("a:hover").specificity();
    let two_pc = parse_selector("a:hover:active").specificity();
    let two_cls = parse_selector("a.x.y").specificity();
    assert!(hover > plain);
    assert!(two_pc >= hover);
    assert_eq!(two_cls, two_pc);
}

#[test]
fn specificity_ordering() {
    let id = parse_selector("#a").specificity();
    let cls = parse_selector(".a").specificity();
    let tag = parse_selector("a").specificity();
    assert!(id > cls);
    assert!(cls > tag);
}
