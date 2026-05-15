use std::sync::Arc;

/// Cheaply-cloneable string — `Arc<str>`.
pub type ArcStr = Arc<str>;

/// A 2D rectangle: position + size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
  pub x: f32,
  pub y: f32,
  pub width: f32,
  pub height: f32,
}

impl Rect {
  pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
    Self { x, y, width, height }
  }
}

// ── Display list IR ──────────────────────────────────────────────────

pub mod display_list;
pub use display_list::*;

// ── CSS types ─────────────────────────────────────────────────────────

pub mod events;

pub mod color;
pub mod combinator;
pub mod css_at_rule;
pub mod css_function;
pub mod css_property;
pub mod css_pseudo;
pub mod css_type;
pub mod element;
pub mod error;
pub mod form_state;

pub mod media;
pub mod node;
pub mod scrollbar;
pub mod selector;
pub mod selector_match;
pub mod selector_parse;
pub mod shorthand;
pub mod stylesheet;
pub mod supports;
pub mod text_edit;
pub mod text_selection;
pub mod transform;
pub mod type_keywords;
pub mod unit;
pub mod validate;
pub mod value;

pub use color::CssColor;
pub use combinator::CssCombinator;
pub use css_at_rule::{AtRuleKind, CssAtRule};
pub use css_function::CssFunction;
pub use css_property::CssProperty;
pub use css_pseudo::CssPseudo;
pub use css_type::CssType;
pub use element::{should_auto_close, HtmlElement, SVG_ELEMENTS};
pub use error::ParseError;
pub use media::{MediaCondition, MediaFeature, MediaModifier, MediaQuery, MediaQueryList};
pub use node::{
  compute_node_hash, hash_kv, hash_tag, ClassList, EventHandler, EventListenerOptions, EventPhase, HtmlDocument,
  HtmlNode, DIRTY_ALL, DIRTY_ATTRS, DIRTY_CHILDREN, DIRTY_TEXT,
};
pub use scrollbar::{
  resolve_pseudo_scrollbar_width, resolve_scrollbar_inset, resolve_scrollbar_min_thumb_size, resolve_scrollbar_width, ScrollbarMode,
  ScrollbarPartStyle, ScrollbarStyles, DEFAULT_SCROLLBAR_WIDTH, THIN_SCROLLBAR_WIDTH,
};
pub use selector::{AttrOp, AttributeSelector, ComplexSelector, CompoundSelector, PseudoSelector, SelectorList};
pub use selector_match::{is_pseudo_element, matches_compound, matches_selector, AncestorEntry, Dir, MatchContext};
pub use selector_parse::{complex_specificity, parse_selector_list};
pub use shorthand::{distribute_values, expand as expand_shorthand, longhands_of};
pub use stylesheet::{AtRule, Declaration, StyleRule, Stylesheet};
pub use supports::{SupportsCondition, SupportsFeature};
pub use text_selection::{cursor_less, SelectionColors, TextCursor, TextSelection};
pub use unit::CssUnit;
pub use validate::{validate_value, Validation};
pub use value::CssValue;
