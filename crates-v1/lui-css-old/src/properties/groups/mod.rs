mod animation;
mod background;
mod border;
mod box_model;
mod flex;
mod grid;
mod position;
mod property_group;
mod svg;
mod typography;
mod visual;

pub use animation::AnimationGroup;
pub use background::BackgroundGroup;
pub use border::BorderGroup;
pub use box_model::BoxModelGroup;
pub use flex::FlexGroup;
pub use grid::GridGroup;
pub use position::PositionGroup;
pub use property_group::PropertyGroup;
pub use svg::SvgGroup;
pub use typography::TypographyGroup;
pub use visual::VisualGroup;

use crate::{
  css_parser::apply_generic_shorthand,
  declaration::DeclarationBlock,
  shorthands::{is_deferred_longhand, shorthand_members},
  style::Style,
  token::{Token, Tokenizer},
  values::ArcStr,
  warn_once,
};

static GROUPS: &[&dyn PropertyGroup] = &[
  &BorderGroup,
  &BackgroundGroup,
  &PositionGroup,
  &BoxModelGroup,
  &TypographyGroup,
  &FlexGroup,
  &GridGroup,
  &VisualGroup,
  &SvgGroup,
  &AnimationGroup,
];

pub fn apply_declarations(declarations: &DeclarationBlock, style: &mut Style) {
  for decl in &declarations.declarations {
    if !decl.property.starts_with("--") {
      diagnose_value(&decl.property, &decl.value);
    }
  }

  for group in GROUPS {
    group.apply(declarations, style);
  }

  for decl in &declarations.declarations {
    let prop = &*decl.property;
    if is_handled_by_groups(prop) {
      continue;
    }
    if shorthand_members(prop).is_some() {
      apply_generic_shorthand(style, prop, &decl.value);
    } else if is_deferred_longhand(prop) {
      style
        .deferred_longhands
        .insert(decl.property.clone(), ArcStr::from(decl.value.as_ref()));
    } else if !prop.starts_with("--") {
      warn_once!("unsupported property: `{prop}`");
    }
  }
}

fn is_handled_by_groups(property: &str) -> bool {
  GROUPS.iter().any(|g| g.handled_properties().contains(&property))
}

pub(crate) fn warn_none<T>(prop: &str, value: &str, result: Option<T>) -> Option<T> {
  if result.is_none() {
    warn_once!("unsupported value for `{prop}`: `{value}`");
  }
  result
}

static KNOWN_FUNCTIONS: &[&str] = &[
  "calc", "min", "max", "clamp", "fit-content",
  "var", "env",
  "rgb", "rgba", "hsl", "hsla", "hwb", "lab", "lch", "oklab", "oklch",
  "color", "color-mix", "light-dark",
  "url",
  "linear-gradient", "radial-gradient", "conic-gradient",
  "repeating-linear-gradient", "repeating-radial-gradient", "repeating-conic-gradient",
  "image-set",
  "cubic-bezier", "steps", "linear",
  "minmax", "repeat",
  "counter", "counters",
  "sin", "cos", "tan", "asin", "acos", "atan", "atan2",
  "pow", "sqrt", "hypot", "log", "exp", "abs", "sign", "mod", "rem", "round",
  "rotate", "rotateX", "rotateY", "rotateZ",
  "scale", "scaleX", "scaleY", "scale3d",
  "translate", "translateX", "translateY", "translateZ", "translate3d",
  "skew", "skewX", "skewY",
  "matrix", "matrix3d", "perspective",
  "blur", "brightness", "contrast", "drop-shadow", "grayscale",
  "hue-rotate", "invert", "opacity", "saturate", "sepia",
  "scroll", "view",
];

static KNOWN_UNITS: &[&str] = &[
  "px", "em", "rem", "%",
  "vw", "vh", "vmin", "vmax",
  "fr",
  "s", "ms",
  "deg", "rad", "grad", "turn",
  "dpi", "dpcm", "dppx",
  "pt", "pc", "in", "cm", "mm", "Q",
];

fn diagnose_value(prop: &str, value: &str) {
  for token in Tokenizer::new(value) {
    match token {
      Token::Function(name) => {
        let lower = name.to_ascii_lowercase();
        if !KNOWN_FUNCTIONS.iter().any(|k| k.eq_ignore_ascii_case(&lower)) {
          warn_once!("unsupported function `{name}()` in `{prop}`");
        }
      }
      Token::Dimension(_, unit) => {
        if !KNOWN_UNITS.iter().any(|k| k.eq_ignore_ascii_case(unit)) {
          warn_once!("unsupported unit `{unit}` in `{prop}`");
        }
      }
      _ => {}
    }
  }
}
