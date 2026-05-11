use lui_css_old::*;

#[test]
fn display_round_trips() {
  let parsed: Display = "inline-block".parse().unwrap();
  assert_eq!(parsed, Display::InlineBlock);
  assert_eq!(parsed.to_string(), "inline-block");
}

#[test]
fn display_case_insensitive() {
  let parsed: Display = "INLINE-GRID".parse().unwrap();
  assert_eq!(parsed, Display::InlineGrid);
}

#[test]
fn display_trims_whitespace() {
  let parsed: Display = " flex ".parse().unwrap();
  assert_eq!(parsed, Display::Flex);
}

#[test]
fn position_round_trips() {
  for (s, expected) in [
    ("static", Position::Static),
    ("relative", Position::Relative),
    ("absolute", Position::Absolute),
    ("fixed", Position::Fixed),
    ("sticky", Position::Sticky),
  ] {
    let parsed: Position = s.parse().unwrap();
    assert_eq!(parsed, expected);
    assert_eq!(parsed.to_string(), s);
  }
}

#[test]
fn flex_direction_round_trips() {
  let parsed: FlexDirection = "row-reverse".parse().unwrap();
  assert_eq!(parsed, FlexDirection::RowReverse);
  assert_eq!(parsed.to_string(), "row-reverse");

  let parsed: FlexDirection = "COLUMN".parse().unwrap();
  assert_eq!(parsed, FlexDirection::Column);
}

#[test]
fn justify_content_round_trips() {
  let parsed: JustifyContent = "space-between".parse().unwrap();
  assert_eq!(parsed, JustifyContent::SpaceBetween);
  assert_eq!(parsed.to_string(), "space-between");
}

#[test]
fn align_items_round_trips() {
  for s in [
    "normal",
    "stretch",
    "center",
    "start",
    "end",
    "flex-start",
    "flex-end",
    "baseline",
  ] {
    let parsed: AlignItems = s.parse().unwrap();
    assert_eq!(parsed.to_string(), s);
  }
}

#[test]
fn font_weight_numeric() {
  let parsed: FontWeight = "700".parse().unwrap();
  assert_eq!(parsed, FontWeight::Weight(700));
  assert_eq!(parsed.to_string(), "700");
}

#[test]
fn font_weight_keywords() {
  let parsed: FontWeight = "bold".parse().unwrap();
  assert_eq!(parsed, FontWeight::Bold);
  assert_eq!(parsed.to_string(), "bold");

  let parsed: FontWeight = "LIGHTER".parse().unwrap();
  assert_eq!(parsed, FontWeight::Lighter);
}

#[test]
fn grid_auto_flow_dense_variants() {
  assert_eq!("dense".parse::<GridAutoFlow>().unwrap(), GridAutoFlow::RowDense);
  assert_eq!(
    "dense column".parse::<GridAutoFlow>().unwrap(),
    GridAutoFlow::ColumnDense
  );
  assert_eq!(
    "column dense".parse::<GridAutoFlow>().unwrap(),
    GridAutoFlow::ColumnDense
  );
  assert_eq!(GridAutoFlow::ColumnDense.to_string(), "column dense");
}

#[test]
fn grid_line_variants() {
  assert_eq!("auto".parse::<GridLine>().unwrap(), GridLine::Auto);
  assert_eq!("3".parse::<GridLine>().unwrap(), GridLine::Line(3));
  assert_eq!("-1".parse::<GridLine>().unwrap(), GridLine::Line(-1));
  assert_eq!("span 2".parse::<GridLine>().unwrap(), GridLine::Span(2));
  assert!("0".parse::<GridLine>().is_err());
  assert!("span 0".parse::<GridLine>().is_err());
}

#[test]
fn scrollbar_width_variants() {
  let parsed: ScrollbarWidth = "thin".parse().unwrap();
  assert_eq!(parsed, ScrollbarWidth::Thin);

  let parsed: ScrollbarWidth = "4px".parse().unwrap();
  assert_eq!(parsed, ScrollbarWidth::Px(4.0));
  assert_eq!(parsed.to_string(), "4px");
}

#[test]
fn cursor_accepts_any_value_as_raw() {
  let parsed: Cursor = "pointer".parse().unwrap();
  assert_eq!(parsed.to_string(), "pointer");

  let parsed: Cursor = "nw-resize".parse().unwrap();
  assert_eq!(parsed.to_string(), "nw-resize");
}

#[test]
fn border_style_round_trips() {
  for s in [
    "none", "solid", "dashed", "dotted", "double", "groove", "ridge", "inset", "outset",
  ] {
    let parsed: BorderStyle = s.parse().unwrap();
    assert_eq!(parsed.to_string(), s);
  }
}

#[test]
fn overflow_round_trips() {
  for s in ["visible", "hidden", "clip", "scroll", "auto"] {
    let parsed: Overflow = s.parse().unwrap();
    assert_eq!(parsed.to_string(), s);
  }
}

#[test]
fn box_sizing_round_trips() {
  let parsed: BoxSizing = "border-box".parse().unwrap();
  assert_eq!(parsed, BoxSizing::BorderBox);
  assert_eq!(parsed.to_string(), "border-box");

  let parsed: BoxSizing = "content-box".parse().unwrap();
  assert_eq!(parsed, BoxSizing::ContentBox);
}

#[test]
fn all_display_variants_parseable() {
  let keywords = [
    "none",
    "block",
    "inline",
    "inline-block",
    "list-item",
    "flex",
    "inline-flex",
    "grid",
    "inline-grid",
    "table",
    "table-caption",
    "table-header-group",
    "table-row-group",
    "table-footer-group",
    "table-row",
    "table-cell",
    "table-column",
    "table-column-group",
    "ruby",
    "ruby-text",
    "contents",
  ];
  for kw in keywords {
    let parsed: Display = kw.parse().unwrap_or_else(|_| panic!("failed to parse display: {kw}"));
    assert_eq!(parsed.to_string(), kw);
  }
}

#[test]
fn from_str_and_from_trait_agree() {
  let a: FlexDirection = "column-reverse".parse().unwrap();
  let b: FlexDirection = "column-reverse".into();
  assert_eq!(a, b);

  let a: JustifyContent = "space-evenly".parse().unwrap();
  let b: JustifyContent = String::from("space-evenly").into();
  assert_eq!(a, b);
}
