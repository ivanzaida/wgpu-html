//! Integration tests for the CSS parser.
//!
//! - `declarations` — `parse_inline_style`: per-property / per-value-type
//!   coverage (lengths, colors, enums, shorthands).
//! - `stylesheet`   — `parse_stylesheet`: full rule grammar, selectors,
//!   comma-lists, comments, malformed input.

#[path = "css/declarations.rs"]
mod declarations;

#[path = "css/stylesheet.rs"]
mod stylesheet;
