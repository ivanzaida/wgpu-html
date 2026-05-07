//! User-agent stylesheet — browser-default CSS applied before author
//! rules.
//!
//! The CSS source lives in `ua.css` next to this file and is embedded
//! at compile time via `include_str!`. Origin-based cascade priority
//! (author > UA) is enforced by the cascade sort key in `lib.rs`.

use std::sync::OnceLock;

use wgpu_html_parser::{Stylesheet, parse_stylesheet};

const UA_CSS: &str = include_str!("ua.css");

/// The lazily-parsed UA stylesheet.
pub fn ua_stylesheet() -> &'static Stylesheet {
  static SHEET: OnceLock<Stylesheet> = OnceLock::new();
  SHEET.get_or_init(|| parse_stylesheet(UA_CSS))
}
