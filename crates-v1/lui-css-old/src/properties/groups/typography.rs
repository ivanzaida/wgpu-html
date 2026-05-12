use super::{property_group::PropertyGroup, warn_none};
use crate::{css_parser::*, declaration::DeclarationBlock, style::Style, values::*};

pub struct TypographyGroup;

impl PropertyGroup for TypographyGroup {
  fn handled_properties(&self) -> &'static [&'static str] {
    &[
      "font-family",
      "font-size",
      "font-weight",
      "font-style",
      "line-height",
      "letter-spacing",
      "text-align",
      "text-decoration",
      "text-transform",
      "text-overflow",
      "white-space",
      "word-break",
      "vertical-align",
      "font",
      "font-variant",
      "font-variant-ligatures",
      "font-synthesis",
    ]
  }

  fn apply(&self, declarations: &DeclarationBlock, style: &mut Style) {
    for decl in &declarations.declarations {
      let (p, v) = (&*decl.property, &*decl.value);
      match p {
        "font-family" => style.font_family = Some(ArcStr::from(v)),
        "font-size" => style.font_size = parse_css_length(v),
        "font-weight" => style.font_weight = warn_none(p, v, parse_font_weight(v)),
        "font-style" => style.font_style = warn_none(p, v, parse_font_style(v)),
        "line-height" => style.line_height = parse_css_length(v),
        "letter-spacing" => style.letter_spacing = parse_css_length(v),
        "text-align" => style.text_align = warn_none(p, v, parse_text_align(v)),
        "text-decoration" => apply_text_decoration_shorthand(v, style),
        "text-transform" => style.text_transform = warn_none(p, v, parse_text_transform(v)),
        "text-overflow" => style.text_overflow = warn_none(p, v, parse_text_overflow(v)),
        "white-space" => apply_white_space_property(v, style),
        "word-break" => style.word_break = warn_none(p, v, parse_word_break(v)),
        "vertical-align" => style.vertical_align = parse_vertical_align(v),
        "font" => apply_font_shorthand(v, style),
        "font-variant" => apply_font_variant_shorthand(style, v),
        "font-variant-ligatures" => apply_font_variant_ligatures_shorthand(style, v),
        "font-synthesis" => apply_font_synthesis_shorthand(style, v),
        _ => {}
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn apply_text_decoration_shorthand(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "text-decoration",
      "text-decoration-line",
      "text-decoration-style",
      "text-decoration-color",
      "text-decoration-thickness",
    ],
  );
  style.text_decoration = Some(ArcStr::from(value));
  let mut lines = Vec::new();
  for token in split_top_level_whitespace(value) {
    match token.to_ascii_lowercase().as_str() {
      "underline" | "overline" | "line-through" | "none" => lines.push(token),
      "solid" | "double" | "dotted" | "dashed" | "wavy" => set_deferred(style, "text-decoration-style", token),
      "auto" | "from-font" => set_deferred(style, "text-decoration-thickness", token),
      _ if parse_css_color(token).is_some() => set_deferred(style, "text-decoration-color", token),
      _ if parse_css_length(token).is_some() => set_deferred(style, "text-decoration-thickness", token),
      _ => {}
    }
  }
  if !lines.is_empty() {
    set_deferred(style, "text-decoration-line", lines.join(" "));
  }
}

fn apply_white_space_property(value: &str, style: &mut Style) {
  mark_property_resets(
    style,
    &[
      "white-space",
      "white-space-collapse",
      "text-wrap-mode",
      "white-space-trim",
    ],
  );
  style.white_space = parse_white_space(value);
  let lower = value.trim().to_ascii_lowercase();
  let (collapse, wrap, trim) = match lower.as_str() {
    "normal" => ("collapse", "wrap", "none"),
    "nowrap" => ("collapse", "nowrap", "none"),
    "pre" => ("preserve", "nowrap", "none"),
    "pre-wrap" => ("preserve", "wrap", "none"),
    "pre-line" => ("preserve-breaks", "wrap", "none"),
    "break-spaces" => ("break-spaces", "wrap", "none"),
    other => (other, other, "none"),
  };
  set_deferred(style, "white-space-collapse", collapse);
  set_deferred(style, "text-wrap-mode", wrap);
  set_deferred(style, "white-space-trim", trim);
}

fn apply_font_shorthand(value: &str, style: &mut Style) {
  mark_shorthand_reset(style, "font");
  set_deferred(style, "font-variant", "normal");
  set_deferred(style, "font-stretch", "normal");
  style.font_style = Some(FontStyle::Normal);
  style.font_weight = Some(FontWeight::Normal);
  style.line_height = Some(CssLength::Raw(ArcStr::from("normal")));

  let tokens = split_top_level_whitespace(value);
  let mut size_idx = None;
  for (idx, token) in tokens.iter().enumerate() {
    if token.contains('/') || is_font_size_token(token) {
      size_idx = Some(idx);
      break;
    }
    match token.to_ascii_lowercase().as_str() {
      "italic" | "oblique" | "normal" => style.font_style = parse_font_style(token),
      "bold" | "bolder" | "lighter" => style.font_weight = parse_font_weight(token),
      "small-caps" => set_deferred(style, "font-variant", *token),
      _ => {
        if let Some(weight) = parse_font_weight(token) {
          style.font_weight = Some(weight);
        } else if matches!(
          token.to_ascii_lowercase().as_str(),
          "ultra-condensed"
            | "extra-condensed"
            | "condensed"
            | "semi-condensed"
            | "semi-expanded"
            | "expanded"
            | "extra-expanded"
            | "ultra-expanded"
        ) {
          set_deferred(style, "font-stretch", *token);
        }
      }
    }
  }
  if let Some(size_idx) = size_idx {
    let size_token = tokens[size_idx];
    if let Some((size_part, line_part)) = size_token.split_once('/') {
      style.font_size = parse_css_length(size_part);
      style.line_height = parse_css_length(line_part);
    } else {
      style.font_size = parse_css_length(size_token);
      if let Some(next) = tokens.get(size_idx + 1) {
        if let Some(line) = next.strip_prefix('/') {
          style.line_height = parse_css_length(line);
        }
      }
    }
    let family_start = if tokens.get(size_idx + 1).is_some_and(|t| t.starts_with('/')) {
      size_idx + 2
    } else {
      size_idx + 1
    };
    if family_start < tokens.len() {
      style.font_family = Some(ArcStr::from(tokens[family_start..].join(" ").as_str()));
    }
  }
}

fn is_font_size_token(token: &str) -> bool {
  matches!(
    parse_css_length(token),
    Some(
      CssLength::Px(_)
        | CssLength::Percent(_)
        | CssLength::Em(_)
        | CssLength::Rem(_)
        | CssLength::Vw(_)
        | CssLength::Vh(_)
        | CssLength::Vmin(_)
        | CssLength::Vmax(_)
        | CssLength::Zero
        | CssLength::Calc(_)
        | CssLength::Min(_)
        | CssLength::Max(_)
        | CssLength::Clamp { .. }
    )
  )
}

fn apply_font_variant_shorthand(style: &mut Style, value: &str) {
  mark_shorthand_reset(style, "font-variant");
  set_deferred(style, "font-variant-ligatures", value);
  set_deferred(style, "font-variant-caps", value);
  set_deferred(style, "font-variant-numeric", value);
  set_deferred(style, "font-variant-east-asian", value);
  set_deferred(style, "font-variant-alternates", value);
  set_deferred(style, "font-variant-position", value);
  set_deferred(style, "font-variant-emoji", value);
}

fn apply_font_variant_ligatures_shorthand(style: &mut Style, value: &str) {
  let tokens = split_top_level_whitespace(value);
  let props = &[
    "font-variant-ligatures-common",
    "font-variant-ligatures-discretionary",
    "font-variant-ligatures-historical",
    "font-variant-ligatures-contextual",
  ];
  mark_property_resets(style, props);
  for (idx, prop) in props.iter().enumerate() {
    let resolved = tokens
      .get(idx)
      .copied()
      .or_else(|| tokens.last().copied())
      .unwrap_or("");
    if !resolved.is_empty() {
      set_deferred(style, prop, resolved);
    }
  }
}

fn apply_font_synthesis_shorthand(style: &mut Style, value: &str) {
  let tokens = split_top_level_whitespace(value);
  let props = &[
    "font-synthesis-weight",
    "font-synthesis-style",
    "font-synthesis-small-caps",
    "font-synthesis-position",
  ];
  mark_property_resets(style, props);
  for (idx, prop) in props.iter().enumerate() {
    let resolved = tokens
      .get(idx)
      .copied()
      .or_else(|| tokens.last().copied())
      .unwrap_or("");
    if !resolved.is_empty() {
      set_deferred(style, prop, resolved);
    }
  }
}
