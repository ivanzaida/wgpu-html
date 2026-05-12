use super::{property_group::PropertyGroup, warn_none};
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style};

pub struct VisualGroup;

impl PropertyGroup for VisualGroup {
  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "opacity",
      "visibility",
      "z-index",
      "overflow",
      "overflow-x",
      "overflow-y",
      "scrollbar-color",
      "scrollbar-width",
      "resize",
      "cursor",
      "pointer-events",
      "user-select",
      "columns",
    ]
  }

  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      let (p, v) = (&*decl.property, &*decl.value);
      match p {
        "opacity" => style.opacity = v.parse().ok(),
        "visibility" => style.visibility = warn_none(p, v, parse_visibility(v)),
        "z-index" => style.z_index = v.parse().ok(),
        "overflow" => apply_overflow_shorthand(v, style),
        "overflow-x" => style.overflow_x = warn_none(p, v, parse_overflow(v)),
        "overflow-y" => style.overflow_y = warn_none(p, v, parse_overflow(v)),
        "scrollbar-color" => style.scrollbar_color = parse_scrollbar_color(v),
        "scrollbar-width" => style.scrollbar_width = warn_none(p, v, parse_scrollbar_width(v)),
        "resize" => style.resize = warn_none(p, v, parse_resize(v)),
        "cursor" => style.cursor = warn_none(p, v, parse_cursor(v)),
        "pointer-events" => style.pointer_events = warn_none(p, v, parse_pointer_events(v)),
        "user-select" => style.user_select = warn_none(p, v, parse_user_select(v)),
        "columns" => apply_columns_shorthand(style, v),
        _ => {}
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_overflow_shorthand(value: &str, style: &mut Style) {
  let mut parts = value.split_whitespace();
  let Some(first) = parts.next().and_then(parse_overflow) else {
    return;
  };
  let second = match parts.next() {
    Some(value) => match parse_overflow(value) {
      Some(parsed) => parsed,
      None => return,
    },
    None => first,
  };
  if parts.next().is_some() {
    return;
  }

  style.overflow = Some(first);
  style.overflow_x = Some(first);
  style.overflow_y = Some(second);
}

fn apply_columns_shorthand(style: &mut Style, value: &str) {
  mark_property_resets(style, &["column-width", "column-count"]);
  for token in split_top_level_whitespace(value) {
    if parse_css_length(token).is_some() && !matches!(token.trim(), "auto") {
      set_deferred(style, "column-width", token);
    } else {
      set_deferred(style, "column-count", token);
    }
  }
}
