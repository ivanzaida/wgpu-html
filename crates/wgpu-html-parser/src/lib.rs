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
pub mod stylesheet;
pub mod tokenizer;
pub mod tree_builder;

pub use css_parser::parse_inline_style;
pub use stylesheet::{Rule, Selector, Stylesheet, parse_stylesheet};
pub use tokenizer::Token;

use wgpu_html_tree::Tree;

/// Parse an HTML string into a tree.
pub fn parse(html: &str) -> Tree {
    tree_builder::build(tokenizer::tokenize(html))
}
