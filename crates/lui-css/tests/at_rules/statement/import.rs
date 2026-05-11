use lui_css::{parse_stylesheet, CssAtRule};

#[test]
fn parses_import_with_url_function_and_trailing_semicolon() {
    let sheet = parse_stylesheet(
        "@import url(\"fonts.css\");"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Import);
    assert_eq!(at.prelude, "url(\"fonts.css\")");
    assert_eq!(at.rules.len(), 0);
    assert_eq!(at.at_rules.len(), 0);
}

#[test]
fn parses_import_with_string_url() {
    let sheet = parse_stylesheet(
        "@import \"styles.css\";"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Import);
    assert_eq!(at.prelude, "\"styles.css\"");
    assert_eq!(at.rules.len(), 0);
}

#[test]
fn parses_import_followed_by_style_rule() {
    let sheet = parse_stylesheet(
        "@import url(\"fonts.css\"); h1 { color: red; }"
    ).unwrap();

    assert_eq!(sheet.at_rules.len(), 1);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Import);
    assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn parses_import_with_media_query() {
    let sheet = parse_stylesheet(
        "@import url(\"print.css\") print;"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Import);
    assert_eq!(at.prelude, "url(\"print.css\") print");
}

#[test]
fn parses_import_with_layer() {
    let sheet = parse_stylesheet(
        "@import url(\"theme.css\") layer(base);"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Import);
    assert_eq!(at.prelude, "url(\"theme.css\") layer(base)");
}

#[test]
fn parses_multiple_import_statements() {
    let sheet = parse_stylesheet(
        "@import url(\"a.css\"); @import url(\"b.css\");"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 2);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Import);
    assert_eq!(sheet.at_rules[1].at_rule, CssAtRule::Import);
}
