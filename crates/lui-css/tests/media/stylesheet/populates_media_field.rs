use lui_css::{parse_stylesheet, CssAtRule, MediaCondition};

#[test]
fn populates_atrule_media_field_when_parsing_media_rule() {
    let sheet = parse_stylesheet(
        "@media (min-width: 1024px) { div { display: none; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert!(at.media.is_some(), "expected media field to be populated");

    let mql = at.media.as_ref().unwrap();
    assert_eq!(mql.0.len(), 1, "expected exactly one media query");
    let query = &mql.0[0];
    assert_eq!(query.conditions.len(), 1);
    assert!(
        matches!(&query.conditions[0], MediaCondition::Feature(_)),
        "expected a feature condition"
    );
}
