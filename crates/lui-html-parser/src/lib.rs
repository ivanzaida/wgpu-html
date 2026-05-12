use std::sync::Arc;

/// Cheaply-cloneable string — `Arc<str>`.
pub type ArcStr = Arc<str>;

pub mod element;
pub mod entities;
pub mod parser;
pub mod tokenizer;

pub use element::{HtmlElement, SVG_ELEMENTS, should_auto_close};
pub use parser::{HtmlDocument, HtmlNode, parse};
pub use tokenizer::{Token, tokenize};
