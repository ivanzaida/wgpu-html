use lui_css::{values::ArcStr, *};

#[test]
fn css_length_formats() {
  assert_eq!(CssLength::Px(12.0).to_string(), "12px");
  assert_eq!(CssLength::Percent(50.0).to_string(), "50%");
  assert_eq!(CssLength::Em(1.5).to_string(), "1.5em");
  assert_eq!(CssLength::Rem(2.0).to_string(), "2rem");
  assert_eq!(CssLength::Vw(100.0).to_string(), "100vw");
  assert_eq!(CssLength::Vh(50.0).to_string(), "50vh");
  assert_eq!(CssLength::Vmin(10.0).to_string(), "10vmin");
  assert_eq!(CssLength::Vmax(20.0).to_string(), "20vmax");
  assert_eq!(CssLength::Auto.to_string(), "auto");
  assert_eq!(CssLength::Zero.to_string(), "0");
}

#[test]
fn css_length_min_max_clamp_format() {
  let min = CssLength::Min(vec![CssLength::Px(100.0), CssLength::Percent(50.0)]);
  assert_eq!(min.to_string(), "min(100px, 50%)");

  let max = CssLength::Max(vec![CssLength::Px(200.0), CssLength::Vw(50.0)]);
  assert_eq!(max.to_string(), "max(200px, 50vw)");

  let clamp = CssLength::Clamp {
    min: Box::new(CssLength::Px(100.0)),
    preferred: Box::new(CssLength::Percent(50.0)),
    max: Box::new(CssLength::Px(600.0)),
  };
  assert_eq!(clamp.to_string(), "clamp(100px, 50%, 600px)");
}

#[test]
fn css_length_raw_format() {
  let raw = CssLength::Raw(ArcStr::from("fit-content(300px)"));
  assert_eq!(raw.to_string(), "fit-content(300px)");
}

#[test]
fn css_color_formats() {
  assert_eq!(CssColor::Rgb(255, 0, 0).to_string(), "rgb(255, 0, 0)");
  assert_eq!(CssColor::Rgba(0, 0, 0, 0.5).to_string(), "rgba(0, 0, 0, 0.5)");
  assert_eq!(CssColor::Hsl(120.0, 50.0, 50.0).to_string(), "hsl(120, 50%, 50%)");
  assert_eq!(
    CssColor::Hsla(240.0, 100.0, 50.0, 0.8).to_string(),
    "hsla(240, 100%, 50%, 0.8)"
  );
  assert_eq!(CssColor::Transparent.to_string(), "transparent");
  assert_eq!(CssColor::CurrentColor.to_string(), "currentColor");
  assert_eq!(CssColor::Named(ArcStr::from("red")).to_string(), "red");
  assert_eq!(CssColor::Hex(ArcStr::from("#ff0000")).to_string(), "#ff0000");
}

#[test]
fn css_image_formats() {
  assert_eq!(CssImage::Url(ArcStr::from("img.png")).to_string(), "url(img.png)");
  assert_eq!(
    CssImage::Function(ArcStr::from("linear-gradient(to right, red, blue)")).to_string(),
    "linear-gradient(to right, red, blue)"
  );
}

#[test]
fn grid_track_size_formats() {
  assert_eq!(GridTrackSize::Auto.to_string(), "auto");
  assert_eq!(GridTrackSize::Fr(1.0).to_string(), "1fr");
  assert_eq!(GridTrackSize::Fr(2.5).to_string(), "2.5fr");
  assert_eq!(GridTrackSize::Length(CssLength::Px(200.0)).to_string(), "200px");
}

#[test]
fn grid_line_formats() {
  assert_eq!(GridLine::Auto.to_string(), "auto");
  assert_eq!(GridLine::Line(3).to_string(), "3");
  assert_eq!(GridLine::Line(-1).to_string(), "-1");
  assert_eq!(GridLine::Span(2).to_string(), "span 2");
}

#[test]
fn css_math_expr_formats() {
  let expr = CssMathExpr::Add(
    Box::new(CssMathExpr::Length(CssLength::Percent(100.0))),
    Box::new(CssMathExpr::Length(CssLength::Px(20.0))),
  );
  assert_eq!(expr.to_string(), "100% + 20px");
}

#[test]
fn css_wide_keyword_from_value() {
  assert_eq!(CssWideKeyword::from_value("inherit"), Some(CssWideKeyword::Inherit));
  assert_eq!(CssWideKeyword::from_value("INITIAL"), Some(CssWideKeyword::Initial));
  assert_eq!(CssWideKeyword::from_value(" unset "), Some(CssWideKeyword::Unset));
  assert_eq!(CssWideKeyword::from_value("red"), None);
  assert_eq!(CssWideKeyword::from_value(""), None);
}

#[test]
fn css_content_values() {
  assert_eq!(CssContent::None, CssContent::None);
  assert_eq!(CssContent::Normal, CssContent::Normal);
  let s = CssContent::String(ArcStr::from("hello"));
  match s {
    CssContent::String(ref v) => assert_eq!(&**v, "hello"),
    _ => panic!("expected string content"),
  }
}
