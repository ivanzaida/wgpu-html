use lui_core::{
  CssProperty, CssUnit, CssValue, MediaCondition, MediaFeature, MediaModifier, MediaQuery, MediaQueryList,
  SupportsCondition, SupportsFeature,
};

/// Viewport and device context for evaluating @media conditions.
#[derive(Debug, Clone, Copy)]
pub struct MediaContext {
  pub viewport_width: f32,
  pub viewport_height: f32,
  pub dpi: f32,
  pub is_screen: bool,
}

impl Default for MediaContext {
  fn default() -> Self {
    Self {
      viewport_width: 1920.0,
      viewport_height: 1080.0,
      dpi: 96.0,
      is_screen: true,
    }
  }
}

/// Evaluate a @media query list. Returns true if any query matches (OR semantics).
pub fn evaluate_media(queries: &MediaQueryList, ctx: &MediaContext) -> bool {
  queries.0.iter().any(|q| evaluate_query(q, ctx))
}

fn evaluate_query(query: &MediaQuery, ctx: &MediaContext) -> bool {
  let type_matches = match &query.media_type {
    Some(t) => match t.to_ascii_lowercase().as_str() {
      "all" => true,
      "screen" => ctx.is_screen,
      "print" => !ctx.is_screen,
      _ => false,
    },
    None => true,
  };

  let conditions_match = query.conditions.iter().all(|c| evaluate_condition(c, ctx));

  let result = type_matches && conditions_match;
  match query.modifier {
    Some(MediaModifier::Not) => !result,
    Some(MediaModifier::Only) => result,
    None => result,
  }
}

fn evaluate_condition(cond: &MediaCondition, ctx: &MediaContext) -> bool {
  match cond {
    MediaCondition::Feature(f) => evaluate_feature(f, ctx),
    MediaCondition::And(inner) => evaluate_condition(inner, ctx),
    MediaCondition::Or(inner) => evaluate_condition(inner, ctx),
    MediaCondition::Not(inner) => !evaluate_condition(inner, ctx),
  }
}

fn evaluate_feature(feature: &MediaFeature, ctx: &MediaContext) -> bool {
  let name = feature.name.to_ascii_lowercase();
  match (name.as_str(), &feature.value) {
    // ── Width ──
    ("width", Some(val)) => approx_eq(ctx.viewport_width, resolve_px(val)),
    ("min-width", Some(val)) => ctx.viewport_width >= resolve_px(val),
    ("max-width", Some(val)) => ctx.viewport_width <= resolve_px(val),

    // ── Height ──
    ("height", Some(val)) => approx_eq(ctx.viewport_height, resolve_px(val)),
    ("min-height", Some(val)) => ctx.viewport_height >= resolve_px(val),
    ("max-height", Some(val)) => ctx.viewport_height <= resolve_px(val),

    // ── Orientation ──
    ("orientation", Some(val)) => match val_as_str(val) {
      Some("portrait") => ctx.viewport_height >= ctx.viewport_width,
      Some("landscape") => ctx.viewport_width > ctx.viewport_height,
      _ => false,
    },

    // ── Display quality ──
    ("color", None) => true,
    ("color", Some(val)) => resolve_number(val) > 0.0,
    ("color-gamut", Some(val)) => match val_as_str(val) {
      Some("srgb") => true,
      _ => false,
    },
    ("monochrome", None) => false,

    // ── Resolution ──
    ("resolution", Some(val)) => approx_eq(ctx.dpi, resolve_dpi(val)),
    ("min-resolution", Some(val)) => ctx.dpi >= resolve_dpi(val),
    ("max-resolution", Some(val)) => ctx.dpi <= resolve_dpi(val),

    // ── Interaction ──
    ("hover", Some(val)) => val_as_str(val) == Some("hover"),
    ("hover", None) => true,
    ("any-hover", Some(val)) => val_as_str(val) == Some("hover"),
    ("pointer", Some(val)) => val_as_str(val) == Some("fine"),
    ("pointer", None) => true,
    ("any-pointer", Some(val)) => val_as_str(val) == Some("fine"),

    // ── Preferences ──
    ("prefers-color-scheme", Some(val)) => val_as_str(val) == Some("light"),
    ("prefers-reduced-motion", Some(val)) => val_as_str(val) == Some("no-preference"),
    ("prefers-contrast", Some(val)) => val_as_str(val) == Some("no-preference"),

    // ── Display mode ──
    ("display-mode", Some(val)) => val_as_str(val) == Some("browser"),

    // ── Boolean features (no value = test support) ──
    ("color-index", None) => true,
    ("grid", Some(val)) => resolve_number(val) == 0.0,
    ("grid", None) => false,

    // Unknown features: permissive
    _ => true,
  }
}

fn resolve_px(val: &CssValue) -> f32 {
  match val {
    CssValue::Number(n) => *n as f32,
    CssValue::Dimension { value, unit } => {
      let v = *value as f32;
      match unit {
        CssUnit::Px => v,
        CssUnit::Em | CssUnit::Rem => v * 16.0,
        CssUnit::Cm => v * 37.7953,
        CssUnit::Mm => v * 3.77953,
        CssUnit::In => v * 96.0,
        CssUnit::Pt => v * 1.3333,
        CssUnit::Pc => v * 16.0,
        _ => v,
      }
    }
    _ => 0.0,
  }
}

fn resolve_dpi(val: &CssValue) -> f32 {
  match val {
    CssValue::Number(n) => *n as f32,
    CssValue::Dimension { value, unit } => {
      let v = *value as f32;
      match unit {
        CssUnit::Dpi => v,
        CssUnit::Dpcm => v * 2.54,
        CssUnit::Dppx => v * 96.0,
        _ => v,
      }
    }
    _ => 96.0,
  }
}

fn resolve_number(val: &CssValue) -> f64 {
  match val {
    CssValue::Number(n) => *n,
    CssValue::Dimension { value, .. } => *value,
    _ => 0.0,
  }
}

fn val_as_str(val: &CssValue) -> Option<&str> {
  match val {
    CssValue::String(s) => Some(s.as_ref()),
    CssValue::Unknown(s) => Some(s.as_ref()),
    _ => None,
  }
}

fn approx_eq(a: f32, b: f32) -> bool {
  (a - b).abs() < 0.01
}

// ---------------------------------------------------------------------------
// @supports evaluation
// ---------------------------------------------------------------------------

/// Evaluate a @supports condition. Returns true if the feature is supported.
pub fn evaluate_supports(cond: &SupportsCondition) -> bool {
  match cond {
    SupportsCondition::Feature(f) => evaluate_supports_feature(f),
    SupportsCondition::Not(inner) => !evaluate_supports(inner),
    SupportsCondition::And(terms) => terms.iter().all(evaluate_supports),
    SupportsCondition::Or(terms) => terms.iter().any(evaluate_supports),
  }
}

fn evaluate_supports_feature(feature: &SupportsFeature) -> bool {
  if feature.is_selector {
    return true;
  }
  let prop = CssProperty::from_name(&feature.name);
  !matches!(prop, CssProperty::Unknown(_))
}
