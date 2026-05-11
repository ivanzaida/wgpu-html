use super::property_group::PropertyGroup;
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::*};

pub struct BoxModelGroup;

impl PropertyGroup for BoxModelGroup {
  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      match &*decl.property {
        "margin" => {
          let (t, r, b, l) = parse_box_shorthand(&decl.value);
          if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
            mark_property_resets(style, &["margin-top", "margin-right", "margin-bottom", "margin-left"]);
            style.margin = t.clone();
            style.margin_top = t;
            style.margin_right = r;
            style.margin_bottom = b;
            style.margin_left = l;
          }
        }
        "margin-top" => style.margin_top = parse_css_length(&decl.value),
        "margin-right" => style.margin_right = parse_css_length(&decl.value),
        "margin-bottom" => style.margin_bottom = parse_css_length(&decl.value),
        "margin-left" => style.margin_left = parse_css_length(&decl.value),
        "padding" => {
          let (t, r, b, l) = parse_box_shorthand(&decl.value);
          if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
            mark_property_resets(
              style,
              &["padding-top", "padding-right", "padding-bottom", "padding-left"],
            );
            style.padding = t.clone();
            style.padding_top = t;
            style.padding_right = r;
            style.padding_bottom = b;
            style.padding_left = l;
          }
        }
        "padding-top" => style.padding_top = parse_css_length(&decl.value),
        "padding-right" => style.padding_right = parse_css_length(&decl.value),
        "padding-bottom" => style.padding_bottom = parse_css_length(&decl.value),
        "padding-left" => style.padding_left = parse_css_length(&decl.value),
        "width" => style.width = parse_css_length(&decl.value),
        "height" => style.height = parse_css_length(&decl.value),
        "min-width" => style.min_width = parse_css_length(&decl.value),
        "min-height" => style.min_height = parse_css_length(&decl.value),
        "max-width" => style.max_width = parse_css_length(&decl.value),
        "max-height" => style.max_height = parse_css_length(&decl.value),
        "box-sizing" => style.box_sizing = parse_box_sizing(&decl.value),
        "box-shadow" => style.box_shadow = Some(ArcStr::from(decl.value.as_ref())),
        "content" => style.content = parse_css_content(&decl.value),
        "list-style-type" => style.list_style_type = parse_list_style_type(&decl.value),
        "list-style-position" => style.list_style_position = parse_list_style_position(&decl.value),
        "list-style" => apply_list_style_shorthand(style, &decl.value),
        "color" => style.color = parse_css_color(&decl.value),
        "accent-color" => style.accent_color = parse_css_color(&decl.value),
        _ => {}
      }
    }
  }

  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "margin",
      "margin-top",
      "margin-right",
      "margin-bottom",
      "margin-left",
      "padding",
      "padding-top",
      "padding-right",
      "padding-bottom",
      "padding-left",
      "width",
      "height",
      "min-width",
      "min-height",
      "max-width",
      "max-height",
      "box-sizing",
      "box-shadow",
      "content",
      "list-style-type",
      "list-style-position",
      "list-style",
      "color",
      "accent-color",
    ]
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_list_style_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "list-style");
  for token in split_top_level_whitespace(value) {
    match token.to_ascii_lowercase().as_str() {
      "inside" | "outside" => style.list_style_position = parse_list_style_position(token),
      _ if parse_list_style_type(token).is_some() => style.list_style_type = parse_list_style_type(token),
      _ if parse_css_image(token).is_some() => set_deferred(style, "list-style-image", token),
      "none" => style.list_style_type = Some(ListStyleType::None),
      _ => set_deferred(style, "list-style-type", token),
    }
  }
}
