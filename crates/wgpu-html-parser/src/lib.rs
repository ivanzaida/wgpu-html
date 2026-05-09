//! HTML/CSS parser.
//!
//! Reads an HTML string into a [`wgpu_html_tree::Tree`] of typed elements,
//! and CSS declaration strings (e.g. the value of an inline `style="..."`
//! attribute) into a [`wgpu_html_models::Style`].
//!
//! Comments, doctypes, unknown tags, and whitespace-only text between tags
//! are dropped — this is a renderer's parser, not a browser's.

pub mod attr_parser;
pub mod css_parser;
pub mod shorthands;
pub mod style_props;
pub mod stylesheet;
pub mod tokenizer;
pub mod tree_builder;

pub use css_parser::{
  CssWideKeyword, StyleDecls, apply_css_property, parse_css_color, parse_inline_style, parse_inline_style_decls,
  resolve_lui_color_picker_style, resolve_lui_popup_style, resolve_var_references,
};
pub use style_props::{apply_keyword, clear_value_for, is_inherited, merge_values_clearing_keywords};
// Re-export query-engine types through the stylesheet module.
pub use stylesheet::{
  AttrFilter, AttrOp, Combinator, ComplexSelector, CompoundSelector, MatchContext, PseudoClass, PseudoElement,
  SelectorList,
};
pub use stylesheet::{
  MediaFeature, MediaQuery, MediaQueryList, MediaType, Rule, Stylesheet, parse_import_directive,
  parse_media_query_list, parse_stylesheet,
};
pub use tokenizer::Token;
use wgpu_html_tree::Tree;

/// Parse an HTML string into a tree.
pub fn parse(html: &str) -> Tree {
  tree_builder::build(tokenizer::tokenize(html))
}
