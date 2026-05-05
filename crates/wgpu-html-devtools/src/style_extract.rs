//! Extract non-None CSS properties from a `Style`, grouped by category.

use std::fmt::Display;

use wgpu_html_models::common::css_enums::GridTrackSize;
use wgpu_html_models::Style;

pub(crate) struct CssDecl {
  pub property: String,
  pub value: String,
}

pub(crate) struct CssDeclGroup {
  pub label: &'static str,
  pub decls: Vec<CssDecl>,
}

macro_rules! extract {
  ($group:ident, $style:expr, $( ($field:ident, $name:literal, $fmt:expr) ),* $(,)?) => {
    $(
      if let Some(ref v) = $style.$field {
        $group.push(CssDecl { property: $name.into(), value: $fmt(v) });
      }
    )*
  };
}

fn css(value: &impl Display) -> String {
  value.to_string()
}

fn css_grid_tracks(values: &[GridTrackSize]) -> String {
  values.iter().map(ToString::to_string).collect::<Vec<_>>().join(" ")
}

pub(crate) fn extract_grouped(style: &Style) -> Vec<CssDeclGroup> {
  let mut groups = Vec::new();

  // ── Layout ──────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (display, "display", css),
    (position, "position", css),
    (top, "top", css),
    (right, "right", css),
    (bottom, "bottom", css),
    (left, "left", css),
    (width, "width", css),
    (height, "height", css),
    (min_width, "min-width", css),
    (min_height, "min-height", css),
    (max_width, "max-width", css),
    (max_height, "max-height", css),
    (overflow, "overflow", css),
    (overflow_x, "overflow-x", css),
    (overflow_y, "overflow-y", css),
    (box_sizing, "box-sizing", css),
  );
  if let Some(v) = style.z_index {
    g.push(CssDecl {
      property: "z-index".into(),
      value: v.to_string(),
    });
  }
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Layout",
      decls: g,
    });
  }

  // ── Spacing ─────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (margin, "margin", css),
    (margin_top, "margin-top", css),
    (margin_right, "margin-right", css),
    (margin_bottom, "margin-bottom", css),
    (margin_left, "margin-left", css),
    (padding, "padding", css),
    (padding_top, "padding-top", css),
    (padding_right, "padding-right", css),
    (padding_bottom, "padding-bottom", css),
    (padding_left, "padding-left", css),
  );
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Spacing",
      decls: g,
    });
  }

  // ── Typography ──────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (color, "color", css),
    (font_family, "font-family", |v: &String| v.clone()),
    (font_size, "font-size", css),
    (font_weight, "font-weight", css),
    (font_style, "font-style", css),
    (line_height, "line-height", css),
    (letter_spacing, "letter-spacing", css),
    (text_align, "text-align", css),
    (text_decoration, "text-decoration", |v: &String| v.clone()),
    (text_transform, "text-transform", css),
    (white_space, "white-space", css),
  );
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Typography",
      decls: g,
    });
  }

  // ── Visual ──────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (background_color, "background-color", css),
    (background_image, "background-image", css),
    (background_size, "background-size", |v: &String| v.clone()),
    (background_position, "background-position", |v: &String| v.clone()),
    (background_repeat, "background-repeat", css),
    (background_clip, "background-clip", css),
    (border_top_width, "border-top-width", css),
    (border_right_width, "border-right-width", css),
    (border_bottom_width, "border-bottom-width", css),
    (border_left_width, "border-left-width", css),
    (border_top_style, "border-top-style", css),
    (border_right_style, "border-right-style", css),
    (border_bottom_style, "border-bottom-style", css),
    (border_left_style, "border-left-style", css),
    (border_top_color, "border-top-color", css),
    (border_right_color, "border-right-color", css),
    (border_bottom_color, "border-bottom-color", css),
    (border_left_color, "border-left-color", css),
    (border_top_left_radius, "border-top-left-radius", css),
    (border_top_right_radius, "border-top-right-radius", css),
    (border_bottom_right_radius, "border-bottom-right-radius", css),
    (border_bottom_left_radius, "border-bottom-left-radius", css),
    (visibility, "visibility", css),
    (box_shadow, "box-shadow", |v: &String| v.clone()),
  );
  if let Some(v) = style.opacity {
    g.push(CssDecl {
      property: "opacity".into(),
      value: v.to_string(),
    });
  }
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Visual",
      decls: g,
    });
  }

  // ── Flex ────────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (flex_direction, "flex-direction", css),
    (flex_wrap, "flex-wrap", css),
    (justify_content, "justify-content", css),
    (align_items, "align-items", css),
    (align_content, "align-content", css),
    (align_self, "align-self", css),
    (gap, "gap", css),
    (row_gap, "row-gap", css),
    (column_gap, "column-gap", css),
    (flex, "flex", |v: &String| v.clone()),
    (flex_basis, "flex-basis", css),
  );
  if let Some(v) = style.order {
    g.push(CssDecl {
      property: "order".into(),
      value: v.to_string(),
    });
  }
  if let Some(v) = style.flex_grow {
    g.push(CssDecl {
      property: "flex-grow".into(),
      value: v.to_string(),
    });
  }
  if let Some(v) = style.flex_shrink {
    g.push(CssDecl {
      property: "flex-shrink".into(),
      value: v.to_string(),
    });
  }
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Flex",
      decls: g,
    });
  }

  // ── Grid ────────────────────────────────────────────────────────
  let mut g = Vec::new();
  if let Some(ref v) = style.grid_template_columns {
    g.push(CssDecl {
      property: "grid-template-columns".into(),
      value: css_grid_tracks(v),
    });
  }
  if let Some(ref v) = style.grid_template_rows {
    g.push(CssDecl {
      property: "grid-template-rows".into(),
      value: css_grid_tracks(v),
    });
  }
  extract!(
    g,
    style,
    (grid_auto_columns, "grid-auto-columns", css),
    (grid_auto_rows, "grid-auto-rows", css),
    (grid_auto_flow, "grid-auto-flow", css),
    (grid_column_start, "grid-column-start", css),
    (grid_column_end, "grid-column-end", css),
    (grid_row_start, "grid-row-start", css),
    (grid_row_end, "grid-row-end", css),
    (justify_items, "justify-items", css),
    (justify_self, "justify-self", css),
  );
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Grid",
      decls: g,
    });
  }

  // ── Transform & Animation ──────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (transform, "transform", |v: &String| v.clone()),
    (transform_origin, "transform-origin", |v: &String| v.clone()),
    (transition, "transition", |v: &String| v.clone()),
    (animation, "animation", |v: &String| v.clone()),
  );
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Transform",
      decls: g,
    });
  }

  // ── Interaction ─────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (cursor, "cursor", css),
    (pointer_events, "pointer-events", css),
    (user_select, "user-select", css),
  );
  if !g.is_empty() {
    groups.push(CssDeclGroup {
      label: "Interaction",
      decls: g,
    });
  }

  // ── SVG ─────────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(
    g,
    style,
    (svg_fill, "fill", css),
    (svg_stroke, "stroke", css),
    (svg_stroke_width, "stroke-width", css),
    (svg_stroke_linecap, "stroke-linecap", |v: &String| v.clone()),
    (svg_stroke_linejoin, "stroke-linejoin", |v: &String| v.clone()),
    (svg_stroke_dasharray, "stroke-dasharray", |v: &String| v.clone()),
    (svg_stroke_dashoffset, "stroke-dashoffset", css),
    (svg_fill_rule, "fill-rule", |v: &String| v.clone()),
  );
  if let Some(v) = style.svg_fill_opacity {
    g.push(CssDecl {
      property: "fill-opacity".into(),
      value: v.to_string(),
    });
  }
  if let Some(v) = style.svg_stroke_opacity {
    g.push(CssDecl {
      property: "stroke-opacity".into(),
      value: v.to_string(),
    });
  }
  if !g.is_empty() {
    groups.push(CssDeclGroup { label: "SVG", decls: g });
  }

  // ── Custom Properties ───────────────────────────────────────────
  if !style.custom_properties.is_empty() {
    let mut decls: Vec<CssDecl> = style
      .custom_properties
      .iter()
      .map(|(k, v)| CssDecl {
        property: k.clone(),
        value: v.clone(),
      })
      .collect();
    decls.sort_by(|a, b| a.property.cmp(&b.property));
    groups.push(CssDeclGroup {
      label: "Custom Properties",
      decls,
    });
  }

  groups
}
