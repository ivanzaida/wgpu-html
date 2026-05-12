use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::ArcStr};

pub struct SvgGroup;

impl PropertyGroup for SvgGroup {
  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "fill",
      "fill-opacity",
      "fill-rule",
      "stroke",
      "stroke-width",
      "stroke-opacity",
      "stroke-linecap",
      "stroke-linejoin",
      "stroke-dasharray",
      "stroke-dashoffset",
    ]
  }

  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      match &*decl.property {
        "fill" => style.svg_fill = parse_css_color(&decl.value),
        "fill-opacity" => style.svg_fill_opacity = decl.value.trim().parse().ok(),
        "fill-rule" => {
          let v = decl.value.trim();
          if matches!(v, "nonzero" | "evenodd") {
            style.svg_fill_rule = Some(ArcStr::from(v));
          }
        }
        "stroke" => style.svg_stroke = parse_css_color(&decl.value),
        "stroke-width" => style.svg_stroke_width = parse_css_length(&decl.value),
        "stroke-opacity" => style.svg_stroke_opacity = decl.value.trim().parse().ok(),
        "stroke-linecap" => {
          let v = decl.value.trim();
          if matches!(v, "butt" | "round" | "square") {
            style.svg_stroke_linecap = Some(ArcStr::from(v));
          }
        }
        "stroke-linejoin" => {
          let v = decl.value.trim();
          if matches!(v, "miter" | "round" | "bevel" | "arcs" | "miter-clip") {
            style.svg_stroke_linejoin = Some(ArcStr::from(v));
          }
        }
        "stroke-dasharray" => {
          let v = decl.value.trim();
          if v != "none" {
            style.svg_stroke_dasharray = Some(ArcStr::from(v));
          }
        }
        "stroke-dashoffset" => style.svg_stroke_dashoffset = parse_css_length(&decl.value),
        _ => {}
      }
    }
  }
}
