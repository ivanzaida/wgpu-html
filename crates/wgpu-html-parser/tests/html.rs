//! Integration tests for the HTML parser.
//!
//! - `single` — one tag at a time, asserts the parsed `Element` variant.
//! - `tree`   — multi-element trees, attributes, text, nesting, auto-close.

#[path = "html/single.rs"]
mod single;

#[path = "html/tree.rs"]
mod tree;
