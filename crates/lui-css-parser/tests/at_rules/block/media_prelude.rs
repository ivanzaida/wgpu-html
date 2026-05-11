use lui_css_parser::{parse_stylesheet, CssAtRule};

#[test]
fn parses_media_with_simple_feature_prelude() {
    let sheet = parse_stylesheet(
        "@media (min-width: 1024px) { div { display: none; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "(min-width: 1024px)");
}

#[test]
fn parses_media_with_compound_and_prelude() {
    let sheet = parse_stylesheet(
        "@media screen and (min-width: 768px) and (max-width: 1024px) { p { color: green; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(
        at.prelude,
        "screen and (min-width: 768px) and (max-width: 1024px)"
    );
}

#[test]
fn parses_media_with_not_negation_prelude() {
    let sheet = parse_stylesheet(
        "@media not all and (color) { span { opacity: 0.5; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "not all and (color)");
}

#[test]
fn parses_media_with_only_keyword_prelude() {
    let sheet = parse_stylesheet(
        "@media only screen and (max-width: 600px) { a { text-decoration: none; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "only screen and (max-width: 600px)");
}

#[test]
fn parses_media_with_comma_separated_queries() {
    let sheet = parse_stylesheet(
        "@media (min-width: 600px), print { body { font-size: 14px; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Media);
    assert_eq!(at.prelude, "(min-width: 600px), print");
}

#[test]
fn trims_whitespace_in_media_prelude() {
    let sheet = parse_stylesheet(
        "@media   (max-width: 300px)   { div { color: red; } }"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.prelude, "(max-width: 300px)");
}
