use lui_cascade::ComputedStyle;
use lui_core::{CssUnit, CssValue};
use lui_glyph::text_style_from_cascade;

#[test]
fn extracts_font_size_px() {
  let val = CssValue::Dimension {
    value: 24.0,
    unit: CssUnit::Px,
  };
  let style = ComputedStyle {
    font_size: Some(&val),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_size, 24.0);
}

#[test]
fn extracts_font_size_from_unitless_number() {
  let val = CssValue::Number(20.0);
  let style = ComputedStyle {
    font_size: Some(&val),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_size, 20.0);
}

#[test]
fn defaults_font_size_to_16_when_missing() {
  let style = ComputedStyle::default();
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_size, 16.0);
}

#[test]
fn extracts_line_height_px() {
  let lh = CssValue::Dimension {
    value: 28.0,
    unit: CssUnit::Px,
  };
  let style = ComputedStyle {
    line_height: Some(&lh),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.line_height, 28.0);
}

#[test]
fn extracts_line_height_as_factor_of_font_size() {
  let fs = CssValue::Dimension {
    value: 20.0,
    unit: CssUnit::Px,
  };
  let lh = CssValue::Number(1.5);
  let style = ComputedStyle {
    font_size: Some(&fs),
    line_height: Some(&lh),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_size, 20.0);
  assert_eq!(ts.line_height, 30.0); // 20 * 1.5
}

#[test]
fn defaults_line_height_to_1_2_factor_of_font_size() {
  let fs = CssValue::Dimension {
    value: 20.0,
    unit: CssUnit::Px,
  };
  let style = ComputedStyle {
    font_size: Some(&fs),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.line_height, 24.0); // 20 * 1.2
}

#[test]
fn defaults_line_height_to_19_2_when_no_font_size() {
  let style = ComputedStyle::default();
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.line_height, 19.2); // 16 * 1.2
}

#[test]
fn extracts_font_weight_number() {
  let w = CssValue::Number(700.0);
  let style = ComputedStyle {
    font_weight: Some(&w),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.weight, 700);
}

#[test]
fn clamps_font_weight_to_1000() {
  let w = CssValue::Number(1500.0);
  let style = ComputedStyle {
    font_weight: Some(&w),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.weight, 1000);
}

#[test]
fn defaults_font_weight_to_400_when_missing() {
  let style = ComputedStyle::default();
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.weight, 400);
}

#[test]
fn extracts_font_family_from_string_value() {
  let fam = CssValue::String("Roboto".into());
  let style = ComputedStyle {
    font_family: Some(&fam),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_family, "Roboto");
}

#[test]
fn extracts_font_family_from_unknown_value() {
  let fam = CssValue::Unknown("Inter".into());
  let style = ComputedStyle {
    font_family: Some(&fam),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_family, "Inter");
}

#[test]
fn defaults_font_family_to_sans_serif_when_missing() {
  let style = ComputedStyle::default();
  let ts = text_style_from_cascade(&style);
  assert_eq!(ts.font_family, "sans-serif");
}

#[test]
fn extracts_all_properties_together() {
  let fs = CssValue::Dimension {
    value: 18.0,
    unit: CssUnit::Px,
  };
  let lh = CssValue::Dimension {
    value: 24.0,
    unit: CssUnit::Px,
  };
  let fw = CssValue::Number(600.0);
  let ff = CssValue::String("Inter".into());

  let style = ComputedStyle {
    font_size: Some(&fs),
    line_height: Some(&lh),
    font_weight: Some(&fw),
    font_family: Some(&ff),
    ..ComputedStyle::default()
  };
  let ts = text_style_from_cascade(&style);

  assert_eq!(ts.font_size, 18.0);
  assert_eq!(ts.line_height, 24.0);
  assert_eq!(ts.weight, 600);
  assert_eq!(ts.font_family, "Inter");
}
