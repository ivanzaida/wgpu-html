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
pub mod media;
pub mod node;
pub mod selector;
pub mod shorthand;
pub mod stylesheet;
pub mod supports;
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
pub use element::{HtmlElement, SVG_ELEMENTS, should_auto_close};
pub use node::{EventHandler, EventListenerOptions, HtmlDocument, HtmlNode, compute_node_hash, hash_kv, hash_tag};
pub use error::ParseError;
pub use media::{MediaCondition, MediaFeature, MediaModifier, MediaQuery, MediaQueryList};
pub use selector::{AttrOp, AttributeSelector, CompoundSelector, ComplexSelector, PseudoSelector, SelectorList};
pub use shorthand::{distribute_values, expand as expand_shorthand, longhands_of};
pub use stylesheet::{AtRule, Declaration, StyleRule, Stylesheet};
pub use supports::{SupportsCondition, SupportsFeature};
pub use unit::CssUnit;
pub use validate::{Validation, validate_value};
pub use value::CssValue;
