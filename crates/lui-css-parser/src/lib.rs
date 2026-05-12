use std::sync::Arc;

/// Cheaply-cloneable string — `Arc<str>`.
pub type ArcStr = Arc<str>;

pub mod color;
pub mod combinator;
pub mod css_at_rule;
pub mod css_function;
pub mod css_property;
pub mod css_pseudo;
pub mod css_type;
pub mod error;
pub mod media;
pub mod parser;
pub mod selector;
pub mod shorthand;
pub mod stylesheet;
pub mod supports;
pub mod tokenizer;
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
pub use error::ParseError;
pub use media::{parse_media_query_list, MediaCondition, MediaFeature, MediaModifier, MediaQuery, MediaQueryList};
pub use parser::{parse_declaration, parse_value};
pub use selector::{parse_selector_list, SelectorList};
pub use shorthand::{expand as expand_shorthand, distribute_values, longhands_of};
pub use supports::{parse_supports_condition, SupportsCondition, SupportsFeature};
pub use stylesheet::{parse_declaration_block, parse_stylesheet, AtRule, Declaration, StyleRule, Stylesheet};
pub use unit::CssUnit;
pub use validate::{validate_value, Validation};
pub use value::CssValue;
