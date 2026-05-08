//! parse_stylesheet: rule grammar, selectors, comments, malformed input.

use wgpu_html_models::common::css_enums::{CssColor, CssLength};
use wgpu_html_parser::{ComplexSelector, Stylesheet, parse_stylesheet};

fn one_rule_selector(sheet: &Stylesheet) -> &ComplexSelector {
  &sheet.rules[0].selectors[0]
}

fn first_compound(sel: &ComplexSelector) -> &wgpu_html_parser::CompoundSelector {
  &sel.compounds[0]
}

// --------------------------------------------------------------------------
// Empty / trivial
// --------------------------------------------------------------------------

#[test]
fn empty_input_yields_no_rules() {
  let sheet = parse_stylesheet("");
  assert!(sheet.rules.is_empty());
}

#[test]
fn whitespace_only_input_yields_no_rules() {
  let sheet = parse_stylesheet("\n  \t  \n");
  assert!(sheet.rules.is_empty());
}

#[test]
fn single_rule_single_decl() {
  let sheet = parse_stylesheet("div { color: red; }");
  assert_eq!(sheet.rules.len(), 1);
  assert_eq!(sheet.rules[0].selectors.len(), 1);
  assert!(sheet.rules[0].declarations.color.is_some());
}

#[test]
fn rule_without_trailing_semicolon() {
  let sheet = parse_stylesheet("div { color: red }");
  assert!(sheet.rules[0].declarations.color.is_some());
}

#[test]
fn empty_rule_body_keeps_rule() {
  let sheet = parse_stylesheet(".empty {}");
  assert_eq!(sheet.rules.len(), 1);
  assert_eq!(first_compound(&sheet.rules[0].selectors[0]).classes, vec!["empty"]);
}

// --------------------------------------------------------------------------
// Selector grammar
// --------------------------------------------------------------------------

#[test]
fn type_selector() {
  let sheet = parse_stylesheet("div { color: red; }");
  let sel = one_rule_selector(&sheet);
  let c = first_compound(sel);
  assert_eq!(c.tag.as_deref(), Some("div"));
  assert!(c.id.is_none());
  assert!(c.classes.is_empty());
}

#[test]
fn id_selector() {
  let sheet = parse_stylesheet("#hero { color: red; }");
  let sel = one_rule_selector(&sheet);
  assert_eq!(first_compound(sel).id.as_deref(), Some("hero"));
}

#[test]
fn class_selector() {
  let sheet = parse_stylesheet(".card { color: red; }");
  let sel = one_rule_selector(&sheet);
  assert_eq!(first_compound(sel).classes, vec!["card"]);
}

#[test]
fn multi_class_selector() {
  let sheet = parse_stylesheet(".card.big.featured { color: red; }");
  let sel = one_rule_selector(&sheet);
  assert_eq!(first_compound(sel).classes, vec!["card", "big", "featured"]);
}

#[test]
fn compound_tag_id_class() {
  let sheet = parse_stylesheet("div#hero.card.big { color: red; }");
  let sel = one_rule_selector(&sheet);
  let c = first_compound(sel);
  assert_eq!(c.tag.as_deref(), Some("div"));
  assert_eq!(c.id.as_deref(), Some("hero"));
  assert_eq!(c.classes, vec!["card", "big"]);
}

#[test]
fn universal_selector() {
  let sheet = parse_stylesheet("* { color: red; }");
  let sel = one_rule_selector(&sheet);
  let c = first_compound(sel);
  assert!(c.tag.is_none() && c.id.is_none() && c.classes.is_empty() && !c.never_matches);
}

#[test]
fn comma_separated_selector_list() {
  let sheet = parse_stylesheet("h1, h2, .big { color: red; }");
  assert_eq!(sheet.rules.len(), 1);
  let sels = &sheet.rules[0].selectors;
  assert_eq!(sels.len(), 3);
  assert_eq!(first_compound(&sels[0]).tag.as_deref(), Some("h1"));
  assert_eq!(first_compound(&sels[1]).tag.as_deref(), Some("h2"));
  assert_eq!(first_compound(&sels[2]).classes, vec!["big"]);
}

#[test]
fn descendant_combinator_keeps_rule() {
  let sheet = parse_stylesheet("div p { color: red; }");
  assert_eq!(sheet.rules.len(), 1);
  let sel = &sheet.rules[0].selectors[0];
  // "div p" → compounds=[div, p], combinators=[Descendant]
  assert_eq!(sel.compounds.len(), 2);
  assert_eq!(sel.compounds[1].tag.as_deref(), Some("p"));
  assert_eq!(sel.compounds[0].tag.as_deref(), Some("div"));
}

#[test]
fn child_combinator_now_supported() {
  // The engine now supports all four combinators; "div > p"
  // no longer drops the selector. Verify that both selectors
  // survive in the comma list.
  let sheet = parse_stylesheet("div > p, .ok { color: red; }");
  let sels = &sheet.rules[0].selectors;
  assert_eq!(sels.len(), 2);
  // "div > p" → compounds=[div, p], combinators=[Child]
  assert_eq!(sels[0].compounds.len(), 2);
  assert_eq!(sels[0].combinators.len(), 1);
  assert!(matches!(sels[0].combinators[0], wgpu_html_parser::Combinator::Child));
  assert_eq!(first_compound(&sels[1]).classes, vec!["ok"]);
}

// --------------------------------------------------------------------------
// Specificity
// --------------------------------------------------------------------------

#[test]
fn specificity_ranking() {
  let id = parse_stylesheet("#a {}").rules[0].selectors[0].specificity();
  let class = parse_stylesheet(".a {}").rules[0].selectors[0].specificity();
  let tag = parse_stylesheet("a {}").rules[0].selectors[0].specificity();
  let universal = parse_stylesheet("* {}").rules[0].selectors[0].specificity();
  assert!(id > class);
  assert!(class > tag);
  assert!(tag > universal);
}

#[test]
fn compound_specificity_adds_up() {
  let s = parse_stylesheet("div#hero.card.big {}").rules[0].selectors[0].specificity();
  let id = 1u32 << 16;
  let cls = 2u32 << 8;
  let tag = 1u32;
  assert_eq!(s, id | cls | tag);
}

// --------------------------------------------------------------------------
// Multi-rule documents
// --------------------------------------------------------------------------

#[test]
fn multiple_rules_preserved_in_order() {
  let sheet = parse_stylesheet(
    "
        body { padding: 10px; }
        #hero { background-color: red; }
        .card { width: 200px; }
        ",
  );
  assert_eq!(sheet.rules.len(), 3);
  assert_eq!(
    first_compound(&sheet.rules[0].selectors[0]).tag.as_deref(),
    Some("body")
  );
  assert_eq!(first_compound(&sheet.rules[1].selectors[0]).id.as_deref(), Some("hero"));
  assert_eq!(first_compound(&sheet.rules[2].selectors[0]).classes, vec!["card"]);
}

#[test]
fn declarations_inside_rules_parse_normally() {
  let sheet = parse_stylesheet(
    ".card {
            width: 200px;
            background-color: #ff8000;
            padding: 8px;
        }",
  );
  let d = &sheet.rules[0].declarations;
  assert!(matches!(d.width, Some(CssLength::Px(v)) if v == 200.0));
  assert!(matches!(d.background_color, Some(CssColor::Hex(_))));
  assert!(matches!(d.padding_top, Some(CssLength::Px(v)) if v == 8.0));
}

// --------------------------------------------------------------------------
// Comments
// --------------------------------------------------------------------------

#[test]
fn block_comments_stripped_at_top_level() {
  let sheet = parse_stylesheet("/* hi */ .x { color: red; }");
  assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn block_comments_inside_rule_body_stripped() {
  let sheet = parse_stylesheet(".x { /* before */ color: red; /* after */ }");
  assert!(sheet.rules[0].declarations.color.is_some());
}

#[test]
fn unterminated_comment_consumed_to_eof() {
  let sheet = parse_stylesheet("/* never closed");
  assert!(sheet.rules.is_empty());
}

#[test]
fn multiline_comment() {
  let sheet = parse_stylesheet(
    "/*
            multi
            line
        */
        .x { color: red; }",
  );
  assert_eq!(sheet.rules.len(), 1);
}

// --------------------------------------------------------------------------
// Robustness / malformed input
// --------------------------------------------------------------------------

#[test]
fn missing_closing_brace_terminates_parsing() {
  let sheet = parse_stylesheet(".a { color: red; .b { color: blue; }");
  assert!(sheet.rules.len() <= 1);
}

#[test]
fn empty_selector_list_drops_rule() {
  let sheet = parse_stylesheet(", , { color: red; }");
  assert!(sheet.rules.is_empty());
}

#[test]
fn extra_braces_do_not_crash() {
  let _ = parse_stylesheet("} } } div { color: red; }");
}

#[test]
fn declarations_with_unknown_props_still_parse_known_ones() {
  let sheet = parse_stylesheet(".x { frobnicate: 7; color: red; nonsense: foo; }");
  assert!(sheet.rules[0].declarations.color.is_some());
}

// --------------------------------------------------------------------------
// @charset
// --------------------------------------------------------------------------

#[test]
fn charset_at_rule_is_skipped() {
  let sheet = parse_stylesheet(r#"@charset "UTF-8"; body { color: red; }"#);
  assert_eq!(sheet.rules.len(), 1);
  assert!(sheet.rules[0].declarations.color.is_some());
}

#[test]
fn charset_before_rules_does_not_eat_following_rules() {
  let sheet = parse_stylesheet(
    r#"@charset "UTF-8";
    .a { color: red; }
    .b { color: blue; }"#,
  );
  assert_eq!(sheet.rules.len(), 2);
}

// --------------------------------------------------------------------------
// @import (skipped by parser — resolved at source-collection level)
// --------------------------------------------------------------------------

#[test]
fn import_at_rule_is_skipped_by_parser() {
  let sheet = parse_stylesheet(r#"@import "reset.css"; body { color: red; }"#);
  assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn import_url_form_is_skipped() {
  let sheet = parse_stylesheet(r#"@import url("theme.css"); .x { color: blue; }"#);
  assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn import_with_media_query_is_skipped() {
  let sheet = parse_stylesheet(r#"@import "print.css" print; body { margin: 0; }"#);
  assert_eq!(sheet.rules.len(), 1);
}

#[test]
fn multiple_imports_then_rules() {
  let sheet = parse_stylesheet(
    r#"@import "a.css";
    @import url("b.css");
    .x { color: red; }
    .y { color: blue; }"#,
  );
  assert_eq!(sheet.rules.len(), 2);
}

// --------------------------------------------------------------------------
// parse_import_directive
// --------------------------------------------------------------------------

#[test]
fn parse_import_directive_double_quoted_url() {
  use wgpu_html_parser::parse_import_directive;
  let result = parse_import_directive(r#""reset.css""#);
  assert_eq!(result, Some(("reset.css", None)));
}

#[test]
fn parse_import_directive_single_quoted_url() {
  use wgpu_html_parser::parse_import_directive;
  let result = parse_import_directive("'theme.css'");
  assert_eq!(result, Some(("theme.css", None)));
}

#[test]
fn parse_import_directive_url_function() {
  use wgpu_html_parser::parse_import_directive;
  let result = parse_import_directive(r#"url("base.css")"#);
  assert_eq!(result, Some(("base.css", None)));
}

#[test]
fn parse_import_directive_with_media_query() {
  use wgpu_html_parser::parse_import_directive;
  let result = parse_import_directive(r#""print.css" print"#);
  assert_eq!(result, Some(("print.css", Some("print"))));
}

#[test]
fn parse_import_directive_url_unquoted() {
  use wgpu_html_parser::parse_import_directive;
  let result = parse_import_directive("url(plain.css)");
  assert_eq!(result, Some(("plain.css", None)));
}
