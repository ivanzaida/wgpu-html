use crate::value::CssValue;

/// A comma-separated list of media queries.
#[derive(Debug, Clone, PartialEq)]
#[repr(transparent)]
pub struct MediaQueryList(pub Vec<MediaQuery>);

/// A single media query: optional modifier, optional type, and conditions.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
  pub modifier: Option<MediaModifier>,
  pub media_type: Option<String>,
  pub conditions: Vec<MediaCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaModifier {
  Not,
  Only,
}

/// A single condition within a media query.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaCondition {
  Feature(MediaFeature),
  And(Box<MediaCondition>),
  Or(Box<MediaCondition>),
  Not(Box<MediaCondition>),
}

/// A single media feature like `(min-width: 600px)` or `(color)`.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaFeature {
  pub name: String,
  pub value: Option<CssValue>,
}
