use super::{property_group::PropertyGroup, warn_none};
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style};

pub struct PositionGroup;

impl PropertyGroup for PositionGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      let (p, v) = (&*decl.property, &*decl.value);
      match p {
        "display" => style.display = warn_none(p, v, parse_display(v)),
        "position" => style.position = warn_none(p, v, parse_position(v)),
        "top" => style.top = parse_css_length(v),
        "right" => style.right = parse_css_length(v),
        "bottom" => style.bottom = parse_css_length(v),
        "left" => style.left = parse_css_length(v),
        "inset" => apply_inset_shorthand(v, style),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &["display", "position", "top", "right", "bottom", "left", "inset"]
  }
}

fn apply_inset_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["top", "right", "bottom", "left"]);
  let (t, r, b, l) = parse_box_shorthand(value);
  style.top = t;
  style.right = r;
  style.bottom = b;
  style.left = l;
}
