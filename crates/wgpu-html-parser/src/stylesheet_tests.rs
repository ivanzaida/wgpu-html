use super::*;

#[test]
fn parses_tag_selector() {
  let s = parse_selector("div").unwrap();
  assert_eq!(s.tag.as_deref(), Some("div"));
  assert!(s.id.is_none());
  assert!(s.classes.is_empty());
}

#[test]
fn parses_id_selector() {
  let s = parse_selector("#hero").unwrap();
  assert_eq!(s.id.as_deref(), Some("hero"));
  assert!(s.tag.is_none());
}

#[test]
fn parses_class_selector() {
  let s = parse_selector(".card").unwrap();
  assert_eq!(s.classes, vec!["card"]);
}

#[test]
fn parses_compound_selector() {
  let s = parse_selector("div#hero.card.big").unwrap();
  assert_eq!(s.tag.as_deref(), Some("div"));
  assert_eq!(s.id.as_deref(), Some("hero"));
  assert_eq!(s.classes, vec!["card", "big"]);
}

#[test]
fn universal_keeps_tag_none() {
  let s = parse_selector("*").unwrap();
  assert!(s.tag.is_none());
}

#[test]
fn parses_descendant_combinator() {
  // `div p` → subject `p` with required ancestor `div`.
  let s = parse_selector("div p").unwrap();
  assert_eq!(s.tag.as_deref(), Some("p"));
  assert_eq!(s.ancestors.len(), 1);
  assert_eq!(s.ancestors[0].tag.as_deref(), Some("div"));
}

#[test]
fn parses_three_level_descendant_chain() {
  // Subject `.c`, immediate ancestor `.b`, further `.a`.
  let s = parse_selector(".a .b .c").unwrap();
  assert_eq!(s.classes, vec!["c"]);
  assert_eq!(s.ancestors.len(), 2);
  assert_eq!(s.ancestors[0].classes, vec!["b"]);
  assert_eq!(s.ancestors[1].classes, vec!["a"]);
}

#[test]
fn descendant_specificity_sums_compounds() {
  // `.a .b` → 2 classes worth of specificity, not 1.
  let two = parse_selector(".a .b").unwrap().specificity();
  let one = parse_selector(".b").unwrap().specificity();
  assert!(two > one);
}

#[test]
fn rejects_unsupported_combinators() {
  // `>`, `+`, `~` still drop the rule.
  assert!(parse_selector("div > p").is_none());
  assert!(parse_selector("div + p").is_none());
  assert!(parse_selector("div ~ p").is_none());
}

#[test]
fn rejects_unknown_pseudo_classes() {
  // We accept dynamic state pseudo-classes only; structural
  // pseudo-classes and pseudo-elements still drop.
  assert!(parse_selector("p::before").is_none());
  assert!(parse_selector("li:nth-child").is_none());
}

#[test]
fn parses_hover_pseudo_class() {
  let s = parse_selector("a:hover").unwrap();
  assert_eq!(s.tag.as_deref(), Some("a"));
  assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover]);
}

#[test]
fn parses_bare_hover_pseudo_class() {
  // `:hover { ... }` matches every hovered element.
  let s = parse_selector(":hover").unwrap();
  assert!(s.tag.is_none());
  assert!(s.id.is_none());
  assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover]);
}

#[test]
fn parses_focus_and_visited_pseudo_classes() {
  let focus = parse_selector(":focus").unwrap();
  assert_eq!(focus.pseudo_classes, vec![PseudoClass::Focus]);
  let visited = parse_selector("a:visited").unwrap();
  assert_eq!(visited.tag.as_deref(), Some("a"));
  assert_eq!(visited.pseudo_classes, vec![PseudoClass::Visited]);
}

#[test]
fn parses_attribute_presence_selector() {
  let s = parse_selector("abbr[title]").unwrap();
  assert_eq!(s.tag.as_deref(), Some("abbr"));
  assert_eq!(s.attributes.len(), 1);
  assert_eq!(s.attributes[0].name, "title");
  assert_eq!(s.attributes[0].value, None);
}

#[test]
fn parses_attribute_equality_selector() {
  let s = parse_selector(r#"input[type="submit"]"#).unwrap();
  assert_eq!(s.tag.as_deref(), Some("input"));
  assert_eq!(s.attributes.len(), 1);
  assert_eq!(s.attributes[0].name, "type");
  assert_eq!(s.attributes[0].value.as_deref(), Some("submit"));
}

#[test]
fn attribute_selector_adds_class_specificity() {
  let plain = parse_selector("input").unwrap().specificity();
  let attr = parse_selector("input[type=submit]").unwrap().specificity();
  assert!(attr > plain);
}

#[test]
fn parses_pseudo_class_after_id_and_class() {
  let s = parse_selector("button#go.primary:hover:active").unwrap();
  assert_eq!(s.tag.as_deref(), Some("button"));
  assert_eq!(s.id.as_deref(), Some("go"));
  assert_eq!(s.classes, vec!["primary"]);
  assert_eq!(s.pseudo_classes, vec![PseudoClass::Hover, PseudoClass::Active]);
}

#[test]
fn pseudo_class_adds_class_specificity() {
  // `a:hover` should beat plain `a` on specificity (1 class +
  // 1 tag vs 1 tag).
  let plain = parse_selector("a").unwrap().specificity();
  let hover = parse_selector("a:hover").unwrap().specificity();
  assert!(hover > plain);
  // Two pseudo-classes match a `.x.y` for specificity.
  let two_pc = parse_selector("a:hover:active").unwrap().specificity();
  let two_cls = parse_selector("a.x.y").unwrap().specificity();
  assert_eq!(two_pc, two_cls);
}

#[test]
fn parses_simple_stylesheet() {
  let css = r#"
          #parent { width: 100px; padding: 10px; }
          .child { width: 30px; height: 30px; }
          #c1 { background-color: red; }
      "#;
  let sheet = parse_stylesheet(css);
  assert_eq!(sheet.rules.len(), 3);
  assert_eq!(sheet.rules[0].selectors[0].id.as_deref(), Some("parent"));
  assert_eq!(sheet.rules[1].selectors[0].classes, vec!["child"]);
  assert!(sheet.rules[2].declarations.background_color.is_some());
}

#[test]
fn handles_comma_lists() {
  let sheet = parse_stylesheet("h1, h2, .big { color: red; }");
  assert_eq!(sheet.rules.len(), 1);
  assert_eq!(sheet.rules[0].selectors.len(), 3);
}

#[test]
fn strips_comments() {
  let sheet = parse_stylesheet("/* hi */ .x { /* ok */ color: red; }");
  assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn parses_media_block_rules() {
  let sheet = parse_stylesheet(
    r#"
          .base { width: 10px; }
          @media screen and (max-width: 600px) {
              .base { width: 20px; }
          }
          "#,
  );
  assert_eq!(sheet.rules.len(), 2);
  assert!(sheet.rules[0].media.is_empty());
  assert_eq!(sheet.rules[1].media.len(), 1);
  assert_eq!(sheet.rules[1].media[0].queries.len(), 1);
  assert_eq!(
    sheet.rules[1].media[0].queries[0].features,
    vec![MediaFeature::MaxWidth(600.0)]
  );
}

#[test]
fn nested_media_blocks_are_anded_on_rules() {
  let sheet = parse_stylesheet(
    r#"
          @media screen {
              @media (orientation: landscape) {
                  .wide { height: 10px; }
              }
          }
          "#,
  );
  assert_eq!(sheet.rules.len(), 1);
  assert_eq!(sheet.rules[0].media.len(), 2);
}

#[test]
fn parses_not_media_modifier() {
  let list = parse_media_query_list("not print and (min-width: 300px)").unwrap();
  assert_eq!(list.queries.len(), 1);
  assert!(list.queries[0].not);
  assert_eq!(list.queries[0].media_type, MediaType::Print);
  assert_eq!(list.queries[0].features, vec![MediaFeature::MinWidth(300.0)]);
}

#[test]
fn specificity_ordering() {
  let id = parse_selector("#a").unwrap().specificity();
  let cls = parse_selector(".a").unwrap().specificity();
  let tag = parse_selector("a").unwrap().specificity();
  assert!(id > cls);
  assert!(cls > tag);
}
