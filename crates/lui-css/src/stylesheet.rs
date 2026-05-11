use crate::{declaration::DeclarationBlock, values::ArcStr};

#[derive(Debug, Clone, Default)]
pub struct Stylesheet {
  pub rules: Vec<CssRule>,
}

impl Stylesheet {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn append(&mut self, other: Stylesheet) {
    self.rules.extend(other.rules);
  }
}

#[derive(Debug, Clone)]
pub enum CssRule {
  Style(StyleRule),
  Media(MediaRule),
  Import(ImportRule),
  Keyframes(KeyframesRule),
  FontFace(FontFaceRule),
  Supports(SupportsRule),
  Unknown(UnknownAtRule),
}

#[derive(Debug, Clone)]
pub struct StyleRule {
  pub selector_text: ArcStr,
  pub declarations: DeclarationBlock,
}

#[derive(Debug, Clone)]
pub struct MediaRule {
  pub query: MediaQueryList,
  pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone)]
pub struct ImportRule {
  pub url: ArcStr,
  pub media: Option<ArcStr>,
}

#[derive(Debug, Clone)]
pub struct KeyframesRule {
  pub name: ArcStr,
  pub keyframes: Vec<Keyframe>,
}

#[derive(Debug, Clone)]
pub struct Keyframe {
  pub selectors: Vec<KeyframeSelector>,
  pub declarations: DeclarationBlock,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyframeSelector {
  Percentage(f32),
  From,
  To,
}

#[derive(Debug, Clone)]
pub struct FontFaceRule {
  pub descriptors: Vec<FontFaceDescriptor>,
}

#[derive(Debug, Clone)]
pub struct FontFaceDescriptor {
  pub name: ArcStr,
  pub value: ArcStr,
}

#[derive(Debug, Clone)]
pub struct SupportsRule {
  pub condition: ArcStr,
  pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone)]
pub struct UnknownAtRule {
  pub name: ArcStr,
  pub prelude: ArcStr,
  pub block: Option<ArcStr>,
}

// ── Media query types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaType {
  All,
  Screen,
  Print,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MediaFeature {
  Width(f32),
  MinWidth(f32),
  MaxWidth(f32),
  Height(f32),
  MinHeight(f32),
  MaxHeight(f32),
  OrientationPortrait,
  OrientationLandscape,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MediaQuery {
  pub not: bool,
  pub media_type: MediaType,
  pub features: Vec<MediaFeature>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MediaQueryList {
  pub queries: Vec<MediaQuery>,
}
