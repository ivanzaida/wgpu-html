use lui_cascade::index::{PreparedStylesheet, candidate_rules};
use lui_parse::parse_stylesheet;
use std::collections::HashMap;

#[test]
fn collects_matching_buckets() {
    let sheet = parse_stylesheet(
        "#x { a: b; } .c { a: b; } div { a: b; } * { a: b; }"
    ).unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    let candidates = candidate_rules(&prepared.index, "div", Some("x"), &["c"], &HashMap::new(), &HashMap::new(), &HashMap::new());
    assert_eq!(candidates.len(), 4);
}

#[test]
fn deduplicates() {
    let sheet = parse_stylesheet("div.c { a: b; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    let candidates = candidate_rules(&prepared.index, "div", None, &["c"], &HashMap::new(), &HashMap::new(), &HashMap::new());
    assert_eq!(candidates.len(), 1);
}

#[test]
fn no_match_returns_only_universal() {
    let sheet = parse_stylesheet("#x { a: b; } .c { a: b; } * { a: b; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    let candidates = candidate_rules(&prepared.index, "div", None, &[], &HashMap::new(), &HashMap::new(), &HashMap::new());
    assert_eq!(candidates.len(), 1); // only universal
}

#[test]
fn empty_sheet_returns_empty() {
    let sheet = parse_stylesheet("").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    let candidates = candidate_rules(&prepared.index, "div", None, &[], &HashMap::new(), &HashMap::new(), &HashMap::new());
    assert!(candidates.is_empty());
}

#[test]
fn multiple_classes_collect_from_each_bucket() {
    let sheet = parse_stylesheet(".a { x: y; } .b { x: y; } .c { x: y; }").unwrap();
    let prepared = PreparedStylesheet::new(sheet);
    let candidates = candidate_rules(&prepared.index, "div", None, &["a", "c"], &HashMap::new(), &HashMap::new(), &HashMap::new());
    assert_eq!(candidates.len(), 2);
}
