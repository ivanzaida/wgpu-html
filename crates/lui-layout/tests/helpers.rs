use lui_cascade::cascade::CascadeContext;
use lui_core::{ArcStr, CssUnit, CssValue};
use lui_layout::LayoutBox;
use lui_parse::parse;

/// Create a `CssValue::Dimension` in px (borrowed, for arena-allocated styles).
pub fn px(v: f32) -> CssValue {
  CssValue::Dimension {
    value: v as f64,
    unit: CssUnit::Px,
  }
}

/// Create a `CssValue::Number`.
pub fn num(v: f32) -> CssValue {
  CssValue::Number(v as f64)
}

/// Create a `CssValue::Percentage`.
pub fn pct(v: f32) -> CssValue {
  CssValue::Percentage(v as f64)
}

/// Create a `CssValue::String("auto")`.
pub fn auto() -> CssValue {
  CssValue::String(ArcStr::from("auto"))
}

pub fn find_by_tag<'a>(b: &'a LayoutBox<'a>, tag: &str) -> Option<&'a LayoutBox<'a>> {
  if b.node.element.tag_name() == tag {
    return Some(b);
  }
  for child in &b.children {
    if let Some(found) = find_by_tag(child, tag) {
      return Some(found);
    }
  }
  None
}

pub fn flex_lt(html: &str, _vw: f32) -> (lui_parse::HtmlDocument, CascadeContext) {
  let full = format!("<html><body>{}</body></html>", html);
  let doc = parse(&full);
  let mut ctx = CascadeContext::new();
  let sheet = lui_parse::parse_stylesheet("* { margin: 0; padding: 0; border-width: 0; }").unwrap();
  ctx.set_stylesheets(&[sheet]);
  (doc, ctx)
}
