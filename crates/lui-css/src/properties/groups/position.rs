use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style};

pub struct PositionGroup;

impl PropertyGroup for PositionGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      match &*decl.property {
        "display" => style.display = parse_display(&decl.value),
        "position" => style.position = parse_position(&decl.value),
        "top" => style.top = parse_css_length(&decl.value),
        "right" => style.right = parse_css_length(&decl.value),
        "bottom" => style.bottom = parse_css_length(&decl.value),
        "left" => style.left = parse_css_length(&decl.value),
        "inset" => apply_inset_shorthand(&decl.value, style),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &["display", "position", "top", "right", "bottom", "left", "inset"]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_inset_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(style, &["top", "right", "bottom", "left"]);
  let (t, r, b, l) = parse_box_shorthand(value);
  style.top = t;
  style.right = r;
  style.bottom = b;
  style.left = l;
}
