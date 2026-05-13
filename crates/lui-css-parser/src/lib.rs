// Re-export all types from lui-core so downstream crates see the same API.
pub use lui_core::*;

// Parsing modules (local to this crate).
pub mod parser;
pub mod tokenizer;

// The mixed modules still live here — they contain both types (re-exported
// from lui-core above) and parsing functions. We declare them so parsing
// functions are accessible, but their type re-exports come from lui-core.
mod selector_parse;
mod stylesheet_parse;
mod media_parse;
mod supports_parse;

pub use parser::{parse_declaration, parse_value};
pub use selector_parse::{complex_specificity, parse_selector_list};
pub use stylesheet_parse::{parse_declaration_block, parse_stylesheet};
pub use media_parse::parse_media_query_list;
pub use supports_parse::parse_supports_condition;
