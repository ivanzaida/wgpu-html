use lui_cascade::index::{PreparedStylesheet, RuleCondition};
use lui_css_parser::parse_stylesheet;

#[test]
fn indexes_by_id() {
    let sheet = parse_stylesheet("#main { color: red; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.by_id.get("main").unwrap().len(), 1);
    assert!(prepared.index.by_class.is_empty());
    assert!(prepared.index.by_tag.is_empty());
    assert!(prepared.index.universal.is_empty());
}

#[test]
fn indexes_by_class() {
    let sheet = parse_stylesheet(".card { padding: 8px; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.by_class.get("card").unwrap().len(), 1);
}

#[test]
fn indexes_by_tag() {
    let sheet = parse_stylesheet("div { display: block; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.by_tag.get("div").unwrap().len(), 1);
}

#[test]
fn universal_goes_to_universal() {
    let sheet = parse_stylesheet("* { margin: 0; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.universal.len(), 1);
}

#[test]
fn id_takes_priority_over_class_and_tag() {
    let sheet = parse_stylesheet("div.card#hero { color: red; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.by_id.get("hero").unwrap().len(), 1);
    assert!(prepared.index.by_class.is_empty());
    assert!(prepared.index.by_tag.is_empty());
}

#[test]
fn multiple_selectors_indexed_separately() {
    let sheet = parse_stylesheet(".a, .b, #c { color: red; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.index.by_class.get("a").unwrap().len(), 1);
    assert_eq!(prepared.index.by_class.get("b").unwrap().len(), 1);
    assert_eq!(prepared.index.by_id.get("c").unwrap().len(), 1);
}

#[test]
fn media_rules_flattened_into_conditional() {
    let sheet = parse_stylesheet("@media (min-width: 768px) { .card { padding: 16px; } }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert!(prepared.rules.is_empty());
    assert_eq!(prepared.conditional_rules.len(), 1);
    assert_eq!(prepared.conditions.len(), 1);
    assert!(matches!(prepared.conditions[0], RuleCondition::Media(_)));
    assert_eq!(prepared.conditional_index.by_class.get("card").unwrap().len(), 1);
}

#[test]
fn supports_rules_flattened_into_conditional() {
    let sheet = parse_stylesheet("@supports (display: grid) { .grid { display: grid; } }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert!(prepared.rules.is_empty());
    assert_eq!(prepared.conditional_rules.len(), 1);
    assert!(matches!(prepared.conditions[0], RuleCondition::Supports(_)));
}

#[test]
fn multiple_rules_indexed() {
    let sheet = parse_stylesheet("div { a: b; } .x { a: b; } #y { a: b; } span { a: b; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    assert_eq!(prepared.rules.len(), 4);
    assert_eq!(prepared.index.by_tag.get("div").unwrap().len(), 1);
    assert_eq!(prepared.index.by_class.get("x").unwrap().len(), 1);
    assert_eq!(prepared.index.by_id.get("y").unwrap().len(), 1);
    assert_eq!(prepared.index.by_tag.get("span").unwrap().len(), 1);
}
