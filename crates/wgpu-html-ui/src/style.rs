//! Builder DSL for constructing CSS stylesheets programmatically.
//!
//! Mirrors the `el::` module pattern — each rule is built with a
//! chainable builder, then serialised to a CSS string via `.to_css()`.
//!
//! # Example
//!
//! ```ignore
//! use wgpu_html_ui::style::{self, px, pct};
//! use wgpu_html_models::common::css_enums::*;
//!
//! let css = style::sheet([
//!     style::rule("body")
//!         .display(Display::Flex)
//!         .flex_direction(FlexDirection::Column)
//!         .overflow(Overflow::Hidden),
//!     style::rule(".card")
//!         .padding(px(16))
//!         .border_radius(px(8))
//!         .background_color("#1a1a1a"),
//!     style::rule(".card:hover")
//!         .background_color("#2a2a2a"),
//! ]).to_css();
//! ```

use std::fmt::Write;

use wgpu_html_models::common::css_enums::{
  AlignContent, AlignItems, AlignSelf, BackgroundClip, BackgroundRepeat, BorderStyle, BoxSizing, Cursor, Display,
  FlexDirection, FlexWrap, FontStyle, FontWeight, GridAutoFlow, JustifyContent, JustifyItems, JustifySelf, Overflow,
  PointerEvents, Position, TextAlign, TextTransform, UserSelect, Visibility, WhiteSpace,
};

// ── Value types ─────────────────────────────────────────────────────────────

/// A CSS length / size value.
#[derive(Debug, Clone)]
pub enum Val {
  Px(f32),
  Pct(f32),
  Em(f32),
  Rem(f32),
  Vw(f32),
  Vh(f32),
  Vmin(f32),
  Vmax(f32),
  Auto,
  Zero,
  Raw(String),
}

impl Val {
  fn to_css_string(&self) -> String {
    match self {
      Val::Px(v) => format!("{v}px"),
      Val::Pct(v) => format!("{v}%"),
      Val::Em(v) => format!("{v}em"),
      Val::Rem(v) => format!("{v}rem"),
      Val::Vw(v) => format!("{v}vw"),
      Val::Vh(v) => format!("{v}vh"),
      Val::Vmin(v) => format!("{v}vmin"),
      Val::Vmax(v) => format!("{v}vmax"),
      Val::Auto => "auto".into(),
      Val::Zero => "0".into(),
      Val::Raw(s) => s.clone(),
    }
  }
}

/// Convenience: allow `px(18)` from integer.
impl From<i32> for Val {
  fn from(v: i32) -> Self {
    Val::Px(v as f32)
  }
}

impl From<u32> for Val {
  fn from(v: u32) -> Self {
    Val::Px(v as f32)
  }
}

/// Convenience: allow `px(18.5)` from float.
impl From<f32> for Val {
  fn from(v: f32) -> Self {
    Val::Px(v)
  }
}

impl From<&str> for Val {
  fn from(v: &str) -> Self {
    match v.trim() {
      "auto" => Val::Auto,
      "0" => Val::Zero,
      other => Val::Raw(other.to_string()),
    }
  }
}

impl From<String> for Val {
  fn from(v: String) -> Self {
    v.as_str().into()
  }
}

// ── Value constructors ──────────────────────────────────────────────────────

/// Helper trait so `px(16)` and `px(16.0)` both work.
pub trait IntoF32 {
  fn into_f32(self) -> f32;
}
impl IntoF32 for f32 {
  fn into_f32(self) -> f32 {
    self
  }
}
impl IntoF32 for i32 {
  fn into_f32(self) -> f32 {
    self as f32
  }
}
impl IntoF32 for u32 {
  fn into_f32(self) -> f32 {
    self as f32
  }
}

pub fn var(v: impl Into<String>) -> String {
  format!("var(--{})", v.into())
}

pub fn px(v: impl IntoF32) -> Val {
  Val::Px(v.into_f32())
}
pub fn pct(v: impl IntoF32) -> Val {
  Val::Pct(v.into_f32())
}
pub fn em(v: impl IntoF32) -> Val {
  Val::Em(v.into_f32())
}
pub fn rem(v: impl IntoF32) -> Val {
  Val::Rem(v.into_f32())
}
pub fn vw(v: impl IntoF32) -> Val {
  Val::Vw(v.into_f32())
}
pub fn vh(v: impl IntoF32) -> Val {
  Val::Vh(v.into_f32())
}
pub fn vmin(v: impl IntoF32) -> Val {
  Val::Vmin(v.into_f32())
}
pub fn vmax(v: impl IntoF32) -> Val {
  Val::Vmax(v.into_f32())
}
pub fn auto() -> Val {
  Val::Auto
}
pub fn raw(s: impl Into<String>) -> Val {
  Val::Raw(s.into())
}

// ── Stylesheet ──────────────────────────────────────────────────────────────

/// A collection of CSS rules, optionally scoped to a component prefix.
pub struct Stylesheet {
  rules: Vec<Rule>,
  scope: &'static str,
}

impl Stylesheet {
  /// An empty stylesheet (no rules, no scope).
  pub fn empty() -> Self {
    Self { rules: Vec::new(), scope: "" }
  }

  /// Returns true if there are no rules.
  pub fn is_empty(&self) -> bool {
    self.rules.is_empty()
  }

  /// The scope prefix, or `""` if unscoped.
  pub fn scope(&self) -> &'static str {
    self.scope
  }

  /// Set the scope prefix. Class selectors are auto-prefixed, and
  /// [`Ctx::scoped`] produces matching names in view().
  pub fn scoped(mut self, scope: &'static str) -> Self {
    self.scope = scope;
    self
  }

  /// Serialize to a CSS string.
  pub fn to_css(&self) -> String {
    let mut out = String::new();
    for rule in &self.rules {
      rule.write_css(&mut out);
      out.push('\n');
    }
    out
  }

  /// Serialize to CSS with all class selectors prefixed.
  ///
  /// Every `.classname` in a selector becomes `.{prefix}-classname`.
  /// Use with [`Ctx::scoped`] in view() to match.
  pub fn to_css_scoped(&self, prefix: &str) -> String {
    let mut out = String::new();
    for rule in &self.rules {
      rule.write_css_scoped(&mut out, prefix);
      out.push('\n');
    }
    out
  }
}

/// Create a stylesheet from an iterator of rules.
pub fn sheet(rules: impl IntoIterator<Item = Rule>) -> Stylesheet {
  Stylesheet {
    rules: rules.into_iter().collect(),
    scope: "",
  }
}

/// Create a scoped stylesheet. Class selectors are auto-prefixed with
/// `scope`, and [`Ctx::scoped`] produces matching class names in view().
pub fn scoped_sheet(scope: &'static str, rules: impl IntoIterator<Item = Rule>) -> Stylesheet {
  Stylesheet {
    rules: rules.into_iter().collect(),
    scope,
  }
}

// ── Rule ────────────────────────────────────────────────────────────────────

/// A single CSS rule: selector + declarations.
pub struct Rule {
  selector: String,
  decls: Vec<(String, String)>,
}

/// Create a rule with the given selector.
pub fn rule(selector: impl Into<String>) -> Rule {
  Rule {
    selector: selector.into(),
    decls: Vec::new(),
  }
}

pub fn rule_class(selector: impl Into<String>) -> Rule {
  rule(format!(".{}", selector.into()))
}

pub fn rule_id(selector: impl Into<String>) -> Rule {
  rule(format!("#{}", selector.into()))
}

impl Rule {
  fn write_css(&self, out: &mut String) {
    let _ = write!(out, "{} {{\n", self.selector);
    for (prop, val) in &self.decls {
      let _ = write!(out, "    {}: {};\n", prop, val);
    }
    out.push_str("}\n");
  }

  fn write_css_scoped(&self, out: &mut String, prefix: &str) {
    let scoped_sel = scope_selector(&self.selector, prefix);
    let _ = write!(out, "{} {{\n", scoped_sel);
    for (prop, val) in &self.decls {
      let _ = write!(out, "    {}: {};\n", prop, val);
    }
    out.push_str("}\n");
  }

  /// Escape hatch: set any property with a raw string value.
  pub fn prop(mut self, property: impl Into<String>, value: impl Into<String>) -> Self {
    self.decls.push((property.into(), value.into()));
    self
  }

  // ── Layout ──────────────────────────────────────────────────────────

  pub fn display(self, v: impl Into<Display>) -> Self {
    self.prop("display", v.into().to_string())
  }
  pub fn position(self, v: impl Into<Position>) -> Self {
    self.prop("position", v.into().to_string())
  }
  pub fn width(self, v: impl Into<Val>) -> Self {
    self.prop("width", v.into().to_css_string())
  }
  pub fn height(self, v: impl Into<Val>) -> Self {
    self.prop("height", v.into().to_css_string())
  }
  pub fn min_width(self, v: impl Into<Val>) -> Self {
    self.prop("min-width", v.into().to_css_string())
  }
  pub fn min_height(self, v: impl Into<Val>) -> Self {
    self.prop("min-height", v.into().to_css_string())
  }
  pub fn max_width(self, v: impl Into<Val>) -> Self {
    self.prop("max-width", v.into().to_css_string())
  }
  pub fn max_height(self, v: impl Into<Val>) -> Self {
    self.prop("max-height", v.into().to_css_string())
  }
  pub fn top(self, v: impl Into<Val>) -> Self {
    self.prop("top", v.into().to_css_string())
  }
  pub fn right(self, v: impl Into<Val>) -> Self {
    self.prop("right", v.into().to_css_string())
  }
  pub fn bottom(self, v: impl Into<Val>) -> Self {
    self.prop("bottom", v.into().to_css_string())
  }
  pub fn left(self, v: impl Into<Val>) -> Self {
    self.prop("left", v.into().to_css_string())
  }

  // ── Spacing ─────────────────────────────────────────────────────────

  pub fn margin(self, v: impl Into<Val>) -> Self {
    self.prop("margin", v.into().to_css_string())
  }
  pub fn margin_top(self, v: impl Into<Val>) -> Self {
    self.prop("margin-top", v.into().to_css_string())
  }
  pub fn margin_right(self, v: impl Into<Val>) -> Self {
    self.prop("margin-right", v.into().to_css_string())
  }
  pub fn margin_bottom(self, v: impl Into<Val>) -> Self {
    self.prop("margin-bottom", v.into().to_css_string())
  }
  pub fn margin_left(self, v: impl Into<Val>) -> Self {
    self.prop("margin-left", v.into().to_css_string())
  }
  pub fn padding(self, v: impl Into<Val>) -> Self {
    self.prop("padding", v.into().to_css_string())
  }
  pub fn padding_top(self, v: impl Into<Val>) -> Self {
    self.prop("padding-top", v.into().to_css_string())
  }
  pub fn padding_right(self, v: impl Into<Val>) -> Self {
    self.prop("padding-right", v.into().to_css_string())
  }
  pub fn padding_bottom(self, v: impl Into<Val>) -> Self {
    self.prop("padding-bottom", v.into().to_css_string())
  }
  pub fn padding_left(self, v: impl Into<Val>) -> Self {
    self.prop("padding-left", v.into().to_css_string())
  }

  /// Shorthand: `padding: top right bottom left;`
  pub fn padding_trbl(self, t: impl Into<Val>, r: impl Into<Val>, l: impl Into<Val>, b: impl Into<Val>) -> Self {
    let (t, r, l, b) = (t.into(), r.into(), l.into(), b.into());
    self.prop(
      "padding",
      format!(
        "{} {} {} {}",
        t.to_css_string(),
        r.to_css_string(),
        b.to_css_string(),
        l.to_css_string()
      ),
    )
  }

  /// Shorthand: `padding: vertical horizontal;`
  pub fn padding_vh(self, vertical: impl Into<Val>, horizontal: impl Into<Val>) -> Self {
    let (vertical, horizontal) = (vertical.into(), horizontal.into());
    self.prop(
      "padding",
      format!("{} {}", vertical.to_css_string(), horizontal.to_css_string()),
    )
  }

  /// Shorthand: `margin: vertical horizontal;`
  pub fn margin_vh(self, vertical: impl Into<Val>, horizontal: impl Into<Val>) -> Self {
    let (vertical, horizontal) = (vertical.into(), horizontal.into());
    self.prop(
      "margin",
      format!("{} {}", vertical.to_css_string(), horizontal.to_css_string()),
    )
  }

  // ── Flex ────────────────────────────────────────────────────────────

  pub fn flex_direction(self, v: impl Into<FlexDirection>) -> Self {
    self.prop("flex-direction", v.into().to_string())
  }
  pub fn flex_wrap(self, v: impl Into<FlexWrap>) -> Self {
    self.prop("flex-wrap", v.into().to_string())
  }
  pub fn flex_grow(self, v: f32) -> Self {
    self.prop("flex-grow", v.to_string())
  }
  pub fn flex_shrink(self, v: f32) -> Self {
    self.prop("flex-shrink", v.to_string())
  }
  pub fn flex_basis(self, v: impl Into<Val>) -> Self {
    self.prop("flex-basis", v.into().to_css_string())
  }
  pub fn justify_content(self, v: impl Into<JustifyContent>) -> Self {
    self.prop("justify-content", v.into().to_string())
  }
  pub fn align_items(self, v: impl Into<AlignItems>) -> Self {
    self.prop("align-items", v.into().to_string())
  }
  pub fn align_content(self, v: impl Into<AlignContent>) -> Self {
    self.prop("align-content", v.into().to_string())
  }
  pub fn align_self(self, v: impl Into<AlignSelf>) -> Self {
    self.prop("align-self", v.into().to_string())
  }
  pub fn gap(self, v: impl Into<Val>) -> Self {
    self.prop("gap", v.into().to_css_string())
  }
  pub fn row_gap(self, v: impl Into<Val>) -> Self {
    self.prop("row-gap", v.into().to_css_string())
  }
  pub fn column_gap(self, v: impl Into<Val>) -> Self {
    self.prop("column-gap", v.into().to_css_string())
  }
  pub fn order(self, v: i32) -> Self {
    self.prop("order", v.to_string())
  }

  // ── Grid ────────────────────────────────────────────────────────────

  pub fn grid_template_columns(self, v: impl Into<String>) -> Self {
    self.prop("grid-template-columns", v.into())
  }
  pub fn grid_template_rows(self, v: impl Into<String>) -> Self {
    self.prop("grid-template-rows", v.into())
  }
  pub fn grid_auto_flow(self, v: impl Into<GridAutoFlow>) -> Self {
    self.prop("grid-auto-flow", v.into().to_string())
  }
  pub fn grid_column(self, v: impl Into<String>) -> Self {
    self.prop("grid-column", v.into())
  }
  pub fn grid_row(self, v: impl Into<String>) -> Self {
    self.prop("grid-row", v.into())
  }
  pub fn justify_items(self, v: impl Into<JustifyItems>) -> Self {
    self.prop("justify-items", v.into().to_string())
  }
  pub fn justify_self(self, v: impl Into<JustifySelf>) -> Self {
    self.prop("justify-self", v.into().to_string())
  }

  // ── Text ────────────────────────────────────────────────────────────

  pub fn font_family(self, v: impl Into<String>) -> Self {
    self.prop("font-family", v.into())
  }
  pub fn font_size(self, v: impl Into<Val>) -> Self {
    self.prop("font-size", v.into().to_css_string())
  }
  pub fn font_weight(self, v: impl Into<FontWeight>) -> Self {
    self.prop("font-weight", v.into().to_string())
  }
  pub fn font_style(self, v: impl Into<FontStyle>) -> Self {
    self.prop("font-style", v.into().to_string())
  }
  pub fn line_height(self, v: impl Into<Val>) -> Self {
    self.prop("line-height", v.into().to_css_string())
  }
  pub fn letter_spacing(self, v: impl Into<Val>) -> Self {
    self.prop("letter-spacing", v.into().to_css_string())
  }
  pub fn text_align(self, v: impl Into<TextAlign>) -> Self {
    self.prop("text-align", v.into().to_string())
  }
  pub fn text_transform(self, v: impl Into<TextTransform>) -> Self {
    self.prop("text-transform", v.into().to_string())
  }
  pub fn text_decoration(self, v: impl Into<String>) -> Self {
    self.prop("text-decoration", v.into())
  }
  pub fn white_space(self, v: impl Into<WhiteSpace>) -> Self {
    self.prop("white-space", v.into().to_string())
  }

  // ── Visual ──────────────────────────────────────────────────────────

  pub fn color(self, v: impl Into<String>) -> Self {
    self.prop("color", v.into())
  }
  pub fn background(self, v: impl Into<String>) -> Self {
    self.prop("background", v.into())
  }
  pub fn background_color(self, v: impl Into<String>) -> Self {
    self.prop("background-color", v.into())
  }
  pub fn background_clip(self, v: impl Into<BackgroundClip>) -> Self {
    self.prop("background-clip", v.into().to_string())
  }
  pub fn background_repeat(self, v: impl Into<BackgroundRepeat>) -> Self {
    self.prop("background-repeat", v.into().to_string())
  }
  pub fn border(self, v: impl Into<String>) -> Self {
    self.prop("border", v.into())
  }
  pub fn border_top(self, v: impl Into<String>) -> Self {
    self.prop("border-top", v.into())
  }
  pub fn border_right(self, v: impl Into<String>) -> Self {
    self.prop("border-right", v.into())
  }
  pub fn border_bottom(self, v: impl Into<String>) -> Self {
    self.prop("border-bottom", v.into())
  }
  pub fn border_left(self, v: impl Into<String>) -> Self {
    self.prop("border-left", v.into())
  }
  pub fn border_style(self, v: impl Into<BorderStyle>) -> Self {
    self.prop("border-style", v.into().to_string())
  }
  pub fn border_radius(self, v: impl Into<Val>) -> Self {
    self.prop("border-radius", v.into().to_css_string())
  }
  pub fn opacity(self, v: f32) -> Self {
    self.prop("opacity", v.to_string())
  }
  pub fn overflow(self, v: impl Into<Overflow>) -> Self {
    self.prop("overflow", v.into().to_string())
  }
  pub fn overflow_x(self, v: impl Into<Overflow>) -> Self {
    self.prop("overflow-x", v.into().to_string())
  }
  pub fn overflow_y(self, v: impl Into<Overflow>) -> Self {
    self.prop("overflow-y", v.into().to_string())
  }
  pub fn box_shadow(self, v: impl Into<String>) -> Self {
    self.prop("box-shadow", v.into())
  }

  // ── Misc ────────────────────────────────────────────────────────────

  pub fn box_sizing(self, v: impl Into<BoxSizing>) -> Self {
    self.prop("box-sizing", v.into().to_string())
  }
  pub fn cursor(self, v: impl Into<Cursor>) -> Self {
    self.prop("cursor", v.into().to_string())
  }
  pub fn pointer_events(self, v: impl Into<PointerEvents>) -> Self {
    self.prop("pointer-events", v.into().to_string())
  }
  pub fn user_select(self, v: impl Into<UserSelect>) -> Self {
    self.prop("user-select", v.into().to_string())
  }
  pub fn visibility(self, v: impl Into<Visibility>) -> Self {
    self.prop("visibility", v.into().to_string())
  }
  pub fn z_index(self, v: i32) -> Self {
    self.prop("z-index", v.to_string())
  }
}

// ── Scoping helpers ─────────────────────────────────────────────────────────

/// Prefix all class selectors in a selector string.
///
/// `.tree-row:hover` → `.{prefix}-tree-row:hover`
/// `.main .item` → `.{prefix}-main .{prefix}-item`
/// `body` → `body` (no dot → no change)
fn scope_selector(selector: &str, prefix: &str) -> String {
  let mut out = String::with_capacity(selector.len() + 32);
  let mut chars = selector.chars().peekable();
  while let Some(ch) = chars.next() {
    if ch == '.' {
      // Check it's a class selector (not part of a decimal number)
      // by looking at what follows: must be a letter, underscore, or hyphen
      if chars
        .peek()
        .is_some_and(|c| c.is_ascii_alphabetic() || *c == '_' || *c == '-')
      {
        out.push('.');
        out.push_str(prefix);
        out.push('-');
      } else {
        out.push('.');
      }
    } else {
      out.push(ch);
    }
  }
  out
}

/// Build the scoped class name for use in `el::div().class(...)`.
///
/// `scoped_class("dt", "tree-row")` → `"dt-tree-row"`
pub fn scoped_class(prefix: &str, class: &str) -> String {
  format!("{prefix}-{class}")
}

/// Build multiple scoped class names separated by spaces.
///
/// `scoped_classes("dt", &["tree-row", "selected"])` → `"dt-tree-row dt-selected"`
pub fn scoped_classes(prefix: &str, classes: &[&str]) -> String {
  classes
    .iter()
    .map(|c| format!("{prefix}-{c}"))
    .collect::<Vec<_>>()
    .join(" ")
}
