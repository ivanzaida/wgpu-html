//! Extract non-None CSS properties from a `Style`, grouped by category.

use wgpu_html_models::Style;

use crate::css_fmt::*;

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

/// Wraps a function returning `&str` to return `String`.
macro_rules! s {
  ($f:expr) => {
    |v| $f(v).to_string()
  };
}

pub(crate) fn extract_grouped(style: &Style) -> Vec<CssDeclGroup> {
  let mut groups = Vec::new();

  // ── Layout ──────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (display,    "display",    s!(fmt_display)),
    (position,   "position",   s!(fmt_position)),
    (top,        "top",        |v| fmt_length(v)),
    (right,      "right",      |v| fmt_length(v)),
    (bottom,     "bottom",     |v| fmt_length(v)),
    (left,       "left",       |v| fmt_length(v)),
    (width,      "width",      |v| fmt_length(v)),
    (height,     "height",     |v| fmt_length(v)),
    (min_width,  "min-width",  |v| fmt_length(v)),
    (min_height, "min-height", |v| fmt_length(v)),
    (max_width,  "max-width",  |v| fmt_length(v)),
    (max_height, "max-height", |v| fmt_length(v)),
    (overflow,   "overflow",   s!(fmt_overflow)),
    (overflow_x, "overflow-x", s!(fmt_overflow)),
    (overflow_y, "overflow-y", s!(fmt_overflow)),
    (box_sizing, "box-sizing", s!(fmt_box_sizing)),
  );
  if let Some(v) = style.z_index {
    g.push(CssDecl { property: "z-index".into(), value: v.to_string() });
  }
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Layout", decls: g }); }

  // ── Spacing ─────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (margin,        "margin",         |v| fmt_length(v)),
    (margin_top,    "margin-top",     |v| fmt_length(v)),
    (margin_right,  "margin-right",   |v| fmt_length(v)),
    (margin_bottom, "margin-bottom",  |v| fmt_length(v)),
    (margin_left,   "margin-left",    |v| fmt_length(v)),
    (padding,        "padding",        |v| fmt_length(v)),
    (padding_top,    "padding-top",    |v| fmt_length(v)),
    (padding_right,  "padding-right",  |v| fmt_length(v)),
    (padding_bottom, "padding-bottom", |v| fmt_length(v)),
    (padding_left,   "padding-left",   |v| fmt_length(v)),
  );
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Spacing", decls: g }); }

  // ── Typography ──────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (color,          "color",           |v| fmt_color(v)),
    (font_family,    "font-family",     |v: &String| v.clone()),
    (font_size,      "font-size",       |v| fmt_length(v)),
    (font_weight,    "font-weight",     |v| fmt_font_weight(v)),
    (font_style,     "font-style",      s!(fmt_font_style)),
    (line_height,    "line-height",     |v| fmt_length(v)),
    (letter_spacing, "letter-spacing",  |v| fmt_length(v)),
    (text_align,     "text-align",      s!(fmt_text_align)),
    (text_decoration,"text-decoration", |v: &String| v.clone()),
    (text_transform, "text-transform",  s!(fmt_text_transform)),
    (white_space,    "white-space",     s!(fmt_white_space)),
  );
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Typography", decls: g }); }

  // ── Visual ──────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (background_color,  "background-color",  |v| fmt_color(v)),
    (background_image,  "background-image",  |v| fmt_image(v)),
    (background_size,   "background-size",   |v: &String| v.clone()),
    (background_position,"background-position",|v: &String| v.clone()),
    (background_repeat, "background-repeat", s!(fmt_background_repeat)),
    (background_clip,   "background-clip",   s!(fmt_background_clip)),
    (border_top_width,    "border-top-width",    |v| fmt_length(v)),
    (border_right_width,  "border-right-width",  |v| fmt_length(v)),
    (border_bottom_width, "border-bottom-width", |v| fmt_length(v)),
    (border_left_width,   "border-left-width",   |v| fmt_length(v)),
    (border_top_style,    "border-top-style",    s!(fmt_border_style)),
    (border_right_style,  "border-right-style",  s!(fmt_border_style)),
    (border_bottom_style, "border-bottom-style", s!(fmt_border_style)),
    (border_left_style,   "border-left-style",   s!(fmt_border_style)),
    (border_top_color,    "border-top-color",    |v| fmt_color(v)),
    (border_right_color,  "border-right-color",  |v| fmt_color(v)),
    (border_bottom_color, "border-bottom-color", |v| fmt_color(v)),
    (border_left_color,   "border-left-color",   |v| fmt_color(v)),
    (border_top_left_radius,     "border-top-left-radius",     |v| fmt_length(v)),
    (border_top_right_radius,    "border-top-right-radius",    |v| fmt_length(v)),
    (border_bottom_right_radius, "border-bottom-right-radius", |v| fmt_length(v)),
    (border_bottom_left_radius,  "border-bottom-left-radius",  |v| fmt_length(v)),
    (visibility, "visibility", s!(fmt_visibility)),
    (box_shadow, "box-shadow", |v: &String| v.clone()),
  );
  if let Some(v) = style.opacity {
    g.push(CssDecl { property: "opacity".into(), value: v.to_string() });
  }
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Visual", decls: g }); }

  // ── Flex ────────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (flex_direction,  "flex-direction",  s!(fmt_flex_direction)),
    (flex_wrap,       "flex-wrap",       s!(fmt_flex_wrap)),
    (justify_content, "justify-content", s!(fmt_justify_content)),
    (align_items,     "align-items",     s!(fmt_align_items)),
    (align_content,   "align-content",   s!(fmt_align_content)),
    (align_self,      "align-self",      s!(fmt_align_self)),
    (gap,             "gap",             |v| fmt_length(v)),
    (row_gap,         "row-gap",         |v| fmt_length(v)),
    (column_gap,      "column-gap",      |v| fmt_length(v)),
    (flex,            "flex",            |v: &String| v.clone()),
    (flex_basis,      "flex-basis",      |v| fmt_length(v)),
  );
  if let Some(v) = style.order {
    g.push(CssDecl { property: "order".into(), value: v.to_string() });
  }
  if let Some(v) = style.flex_grow {
    g.push(CssDecl { property: "flex-grow".into(), value: v.to_string() });
  }
  if let Some(v) = style.flex_shrink {
    g.push(CssDecl { property: "flex-shrink".into(), value: v.to_string() });
  }
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Flex", decls: g }); }

  // ── Grid ────────────────────────────────────────────────────────
  let mut g = Vec::new();
  if let Some(ref v) = style.grid_template_columns {
    g.push(CssDecl { property: "grid-template-columns".into(), value: fmt_grid_tracks(v) });
  }
  if let Some(ref v) = style.grid_template_rows {
    g.push(CssDecl { property: "grid-template-rows".into(), value: fmt_grid_tracks(v) });
  }
  extract!(g, style,
    (grid_auto_columns, "grid-auto-columns", |v| fmt_grid_track(v)),
    (grid_auto_rows,    "grid-auto-rows",    |v| fmt_grid_track(v)),
    (grid_auto_flow,    "grid-auto-flow",    s!(fmt_grid_auto_flow)),
    (grid_column_start, "grid-column-start", |v| fmt_grid_line(v)),
    (grid_column_end,   "grid-column-end",   |v| fmt_grid_line(v)),
    (grid_row_start,    "grid-row-start",    |v| fmt_grid_line(v)),
    (grid_row_end,      "grid-row-end",      |v| fmt_grid_line(v)),
    (justify_items,     "justify-items",     s!(fmt_justify_items)),
    (justify_self,      "justify-self",      s!(fmt_justify_self)),
  );
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Grid", decls: g }); }

  // ── Transform & Animation ──────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (transform,        "transform",        |v: &String| v.clone()),
    (transform_origin, "transform-origin", |v: &String| v.clone()),
    (transition,       "transition",       |v: &String| v.clone()),
    (animation,        "animation",        |v: &String| v.clone()),
  );
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Transform", decls: g }); }

  // ── Interaction ─────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (cursor,         "cursor",         |v| fmt_cursor(v)),
    (pointer_events, "pointer-events", s!(fmt_pointer_events)),
    (user_select,    "user-select",    s!(fmt_user_select)),
  );
  if !g.is_empty() { groups.push(CssDeclGroup { label: "Interaction", decls: g }); }

  // ── SVG ─────────────────────────────────────────────────────────
  let mut g = Vec::new();
  extract!(g, style,
    (svg_fill,             "fill",             |v| fmt_color(v)),
    (svg_stroke,           "stroke",           |v| fmt_color(v)),
    (svg_stroke_width,     "stroke-width",     |v| fmt_length(v)),
    (svg_stroke_linecap,   "stroke-linecap",   |v: &String| v.clone()),
    (svg_stroke_linejoin,  "stroke-linejoin",  |v: &String| v.clone()),
    (svg_stroke_dasharray, "stroke-dasharray", |v: &String| v.clone()),
    (svg_stroke_dashoffset,"stroke-dashoffset",|v| fmt_length(v)),
    (svg_fill_rule,        "fill-rule",        |v: &String| v.clone()),
  );
  if let Some(v) = style.svg_fill_opacity {
    g.push(CssDecl { property: "fill-opacity".into(), value: v.to_string() });
  }
  if let Some(v) = style.svg_stroke_opacity {
    g.push(CssDecl { property: "stroke-opacity".into(), value: v.to_string() });
  }
  if !g.is_empty() { groups.push(CssDeclGroup { label: "SVG", decls: g }); }

  // ── Custom Properties ───────────────────────────────────────────
  if !style.custom_properties.is_empty() {
    let mut decls: Vec<CssDecl> = style
      .custom_properties
      .iter()
      .map(|(k, v)| CssDecl { property: k.clone(), value: v.clone() })
      .collect();
    decls.sort_by(|a, b| a.property.cmp(&b.property));
    groups.push(CssDeclGroup { label: "Custom Properties", decls });
  }

  groups
}
