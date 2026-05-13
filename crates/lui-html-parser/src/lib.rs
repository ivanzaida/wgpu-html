// Re-export shared types from lui-core.
pub use lui_core::{ArcStr, Rect, HtmlElement, SVG_ELEMENTS, should_auto_close};

pub mod entities;
pub mod parser;
pub mod tokenizer;

pub use parser::{HtmlDocument, HtmlNode, parse};
pub use tokenizer::{Token, tokenize};
