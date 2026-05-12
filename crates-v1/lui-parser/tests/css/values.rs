use lui_models::common::css_enums::{CssColor, CssLength};
use lui_parser::{
  css_parser::{parse_css_color, parse_css_length},
  parse_inline_style, parse_inline_style_decls,
};

#[test]
fn parse_css_length_values() {
  assert!(matches!(parse_css_length("auto"), Some(CssLength::Auto)));
  assert!(matches!(parse_css_length("0"), Some(CssLength::Zero)));
  assert!(matches!(parse_css_length("10px"), Some(CssLength::Px(v)) if (v - 10.0).abs() < 0.01));
  assert!(matches!(parse_css_length("50%"), Some(CssLength::Percent(v)) if (v - 50.0).abs() < 0.01));
  assert!(matches!(parse_css_length("1.5em"), Some(CssLength::Em(v)) if (v - 1.5).abs() < 0.01));
  assert!(matches!(parse_css_length("2rem"), Some(CssLength::Rem(v)) if (v - 2.0).abs() < 0.01));
}

#[test]
fn parse_css_color_hex() {
  assert!(matches!(parse_css_color("#ff0000"), Some(CssColor::Hex(ref s)) if &**s == "#ff0000"));
}

#[test]
fn parse_css_color_rgb() {
  let c = parse_css_color("rgb(255, 128, 0)");
  assert!(matches!(c, Some(CssColor::Rgb(255, 128, 0))));
}

#[test]
fn parse_css_color_rgba() {
  let c = parse_css_color("rgba(255, 128, 0, 0.5)");
  assert!(matches!(c, Some(CssColor::Rgba(255, 128, 0, a)) if (a - 0.5).abs() < 0.01));
}

#[test]
fn parse_css_color_transparent() {
  assert!(matches!(parse_css_color("transparent"), Some(CssColor::Transparent)));
}

#[test]
fn parse_font_weight_via_declarations() {
  use lui_models::common::css_enums::FontWeight;
  let s = parse_inline_style("font-weight: 700;");
  assert!(matches!(s.font_weight, Some(FontWeight::Weight(700))));
  let s = parse_inline_style("font-weight: bold;");
  assert!(matches!(s.font_weight, Some(FontWeight::Bold)));
}

#[test]
fn padding_shorthand_one_value() {
  let s = parse_inline_style("padding: 10px;");
  assert!(matches!(s.padding_top, Some(CssLength::Px(10.0))));
  assert!(matches!(s.padding_right, Some(CssLength::Px(10.0))));
  assert!(matches!(s.padding_bottom, Some(CssLength::Px(10.0))));
  assert!(matches!(s.padding_left, Some(CssLength::Px(10.0))));
  assert!(s.padding.is_some());
}

#[test]
fn padding_shorthand_two_values() {
  let s = parse_inline_style("padding: 6px 10px;");
  assert!(matches!(s.padding_top, Some(CssLength::Px(6.0))));
  assert!(matches!(s.padding_bottom, Some(CssLength::Px(6.0))));
  assert!(matches!(s.padding_left, Some(CssLength::Px(10.0))));
  assert!(matches!(s.padding_right, Some(CssLength::Px(10.0))));
}

#[test]
fn padding_shorthand_three_values() {
  let s = parse_inline_style("padding: 1px 2px 3px;");
  assert!(matches!(s.padding_top, Some(CssLength::Px(1.0))));
  assert!(matches!(s.padding_right, Some(CssLength::Px(2.0))));
  assert!(matches!(s.padding_left, Some(CssLength::Px(2.0))));
  assert!(matches!(s.padding_bottom, Some(CssLength::Px(3.0))));
}

#[test]
fn padding_shorthand_four_values() {
  let s = parse_inline_style("padding: 1px 2px 3px 4px;");
  assert!(matches!(s.padding_top, Some(CssLength::Px(1.0))));
  assert!(matches!(s.padding_right, Some(CssLength::Px(2.0))));
  assert!(matches!(s.padding_bottom, Some(CssLength::Px(3.0))));
  assert!(matches!(s.padding_left, Some(CssLength::Px(4.0))));
}

#[test]
fn margin_shorthand_two_values_mixed_units() {
  let s = parse_inline_style("margin: 1em 20px;");
  assert!(matches!(s.margin_top, Some(CssLength::Em(v)) if (v - 1.0).abs() < 0.01));
  assert!(matches!(s.margin_bottom, Some(CssLength::Em(v)) if (v - 1.0).abs() < 0.01));
  assert!(matches!(s.margin_left, Some(CssLength::Px(20.0))));
  assert!(matches!(s.margin_right, Some(CssLength::Px(20.0))));
}

#[test]
fn padding_shorthand_zero_and_auto() {
  let s = parse_inline_style("margin: 0 auto;");
  assert!(matches!(s.margin_top, Some(CssLength::Zero)));
  assert!(matches!(s.margin_bottom, Some(CssLength::Zero)));
  assert!(matches!(s.margin_left, Some(CssLength::Auto)));
  assert!(matches!(s.margin_right, Some(CssLength::Auto)));
}

#[test]
fn padding_shorthand_too_many_tokens_is_invalid() {
  let s = parse_inline_style("padding: 1px 2px 3px 4px 5px;");
  assert!(s.padding_top.is_none());
  assert!(s.padding_right.is_none());
  assert!(s.padding_bottom.is_none());
  assert!(s.padding_left.is_none());
  assert!(s.padding.is_none());
}

#[test]
fn flex_shorthand_extracts_flex_grow() {
  let s = parse_inline_style("flex: 1;");
  assert_eq!(s.flex_grow, Some(1.0));
  assert!(matches!(s.flex_basis, Some(CssLength::Percent(v)) if (v - 0.0).abs() < 0.01));
  assert_eq!(s.flex.as_deref(), Some("1"));

  let s = parse_inline_style("flex: 2.5;");
  assert_eq!(s.flex_grow, Some(2.5));

  let s = parse_inline_style("flex: auto;");
  assert_eq!(s.flex_grow, Some(1.0));

  let s = parse_inline_style("flex: none;");
  assert_eq!(s.flex_grow, Some(0.0));

  let s = parse_inline_style("flex: 3 1 0%;");
  assert_eq!(s.flex_grow, Some(3.0));
}

#[test]
fn parse_box_shorthand_direct() {
  use lui_parser::css_parser::parse_box_shorthand;
  let (t, r, b, l) = parse_box_shorthand("1px 2px 3px 4px");
  assert!(matches!(t, Some(CssLength::Px(1.0))));
  assert!(matches!(r, Some(CssLength::Px(2.0))));
  assert!(matches!(b, Some(CssLength::Px(3.0))));
  assert!(matches!(l, Some(CssLength::Px(4.0))));
  let (t, r, b, l) = parse_box_shorthand("1px 2px 3px 4px 5px");
  assert!(t.is_none() && r.is_none() && b.is_none() && l.is_none());
}

#[test]
fn border_shorthand_with_rgb_color_keeps_function_intact() {
  let style = parse_inline_style("border: 2px solid rgb(212, 175, 55);");
  let top_color = style.border_top_color.as_ref().expect("border-top-color should be set");
  match top_color {
    CssColor::Rgb(212, 175, 55) => {}
    other => panic!("expected Rgb(212, 175, 55), got {:?}", other),
  }
}

#[test]
fn important_routes_to_important_bucket() {
  let decls = parse_inline_style_decls("color: red !important;");
  assert!(decls.normal.color.is_none());
  assert!(decls.important.color.is_some());
}

#[test]
fn normal_and_important_in_same_block_split_by_property() {
  let decls = parse_inline_style_decls("color: red !important; background-color: blue;");
  assert!(decls.normal.color.is_none());
  assert!(decls.normal.background_color.is_some());
  assert!(decls.important.color.is_some());
  assert!(decls.important.background_color.is_none());
}

#[test]
fn important_value_parses_as_if_it_were_normal() {
  let decls = parse_inline_style_decls("width: 100px !important;");
  assert!(matches!(decls.important.width, Some(CssLength::Px(v)) if v == 100.0));
}

#[test]
fn important_is_case_insensitive_and_whitespace_tolerant() {
  for css in [
    "color: red !important;",
    "color: red ! important;",
    "color: red  !  IMPORTANT  ;",
    "color: red !IMPORTANT;",
  ] {
    let decls = parse_inline_style_decls(css);
    assert!(
      decls.important.color.is_some(),
      "expected `{css}` to be marked important"
    );
    assert!(decls.normal.color.is_none(), "`{css}` leaked into normal");
  }
}

#[test]
fn parse_inline_style_folds_important_back_in() {
  let style = parse_inline_style("color: red; color: blue !important;");
  let c = style.color.expect("color set");
  assert!(matches!(c, CssColor::Named(ref s) if &**s == "blue"));
}

#[test]
fn bare_word_important_without_bang_is_not_important() {
  let decls = parse_inline_style_decls("color: red important;");
  assert!(decls.important.color.is_none());
}
