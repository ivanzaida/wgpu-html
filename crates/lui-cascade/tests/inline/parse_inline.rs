use lui_cascade::inline::parse_inline_style;
use lui_css_parser::CssProperty;

#[test]
fn parses_single_declaration() {
    let decls = parse_inline_style("color: red");
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].property, CssProperty::Color);
    assert!(!decls[0].important);
}

#[test]
fn parses_multiple_declarations() {
    let decls = parse_inline_style("color: red; display: block; margin: 0");
    assert_eq!(decls.len(), 3);
}

#[test]
fn handles_important() {
    let decls = parse_inline_style("color: red !important");
    assert_eq!(decls.len(), 1);
    assert!(decls[0].important);
}

#[test]
fn handles_trailing_semicolon() {
    let decls = parse_inline_style("color: red;");
    assert_eq!(decls.len(), 1);
}

#[test]
fn skips_empty_declarations() {
    let decls = parse_inline_style(";;color: red;;;");
    assert_eq!(decls.len(), 1);
}

#[test]
fn returns_empty_for_empty_string() {
    let decls = parse_inline_style("");
    assert!(decls.is_empty());
}

#[test]
fn handles_whitespace() {
    let decls = parse_inline_style("  color :  red  ;  display :  block  ");
    assert_eq!(decls.len(), 2);
}

#[test]
fn separates_normal_and_important() {
    let decls = parse_inline_style("color: red; display: block !important; margin: 0");
    let normal: Vec<_> = decls.iter().filter(|d| !d.important).collect();
    let important: Vec<_> = decls.iter().filter(|d| d.important).collect();
    assert_eq!(normal.len(), 2);
    assert_eq!(important.len(), 1);
    assert_eq!(important[0].property, CssProperty::Display);
}
