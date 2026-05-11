use super::property_group::PropertyGroup;
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
      match &*decl.property {
        "opacity" => style.opacity = decl.value.parse().ok(),
        "visibility" => style.visibility = parse_visibility(&decl.value),
        "z-index" => style.z_index = decl.value.parse().ok(),
        "overflow" => apply_overflow_shorthand(&decl.value, style),
        "overflow-x" => style.overflow_x = parse_overflow(&decl.value),
        "overflow-y" => style.overflow_y = parse_overflow(&decl.value),
        "scrollbar-color" => style.scrollbar_color = parse_scrollbar_color(&decl.value),
        "scrollbar-width" => style.scrollbar_width = parse_scrollbar_width(&decl.value),
        "resize" => style.resize = parse_resize(&decl.value),
        "cursor" => style.cursor = parse_cursor(&decl.value),
        "pointer-events" => style.pointer_events = parse_pointer_events(&decl.value),
        "user-select" => style.user_select = parse_user_select(&decl.value),
        "columns" => apply_columns_shorthand(style, &decl.value),
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
