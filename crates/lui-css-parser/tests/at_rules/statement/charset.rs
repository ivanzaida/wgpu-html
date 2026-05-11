use lui_css_parser::{parse_stylesheet, CssAtRule};
use lui_css_parser::ArcStr;

#[test]
fn parses_charset_with_utf8_string() {
    let sheet = parse_stylesheet(
        "@charset \"UTF-8\";"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Charset);
    assert_eq!(at.prelude, "\"UTF-8\"");
    assert_eq!(at.rules.len(), 0);
    assert_eq!(at.at_rules.len(), 0);
}

#[test]
fn parses_charset_with_lowercase_encoding() {
    let sheet = parse_stylesheet(
        "@charset \"iso-8859-1\";"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Charset);
    assert_eq!(at.prelude, "\"iso-8859-1\"");
}

#[test]
fn parses_charset_without_trailing_semicolon_as_statement() {
    // The parser is forgiving: it treats a missing semicolon as end-of-prelude
    let sheet = parse_stylesheet(
        "@charset \"UTF-8\""
    ).unwrap();

    assert_eq!(sheet.at_rules.len(), 1);
    assert_eq!(sheet.at_rules[0].at_rule, CssAtRule::Charset);
    assert_eq!(sheet.at_rules[0].prelude, "\"UTF-8\"");
}

#[test]
fn parses_charset_with_empty_encoding_string() {
    let sheet = parse_stylesheet(
        "@charset \"\";"
    ).unwrap();

    let at = &sheet.at_rules[0];
    assert_eq!(at.at_rule, CssAtRule::Charset);
    assert_eq!(at.prelude, "\"\"");
}

#[test]
fn parses_unknown_at_rule_as_unknown_variant() {
    let sheet = parse_stylesheet(
        "@custom-rule \"value\";"
    ).unwrap();

    assert_eq!(sheet.rules.len(), 0);
    assert_eq!(sheet.at_rules.len(), 1);
    assert_eq!(
        sheet.at_rules[0].at_rule,
        CssAtRule::Unknown("@custom-rule".into())
    );
    assert_eq!(sheet.at_rules[0].prelude, "\"value\"");
    assert_eq!(sheet.at_rules[0].rules.len(), 0);
}